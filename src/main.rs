// Temporary
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use crate::alloc::string::ToString;
use crate::services::cc1101::RadioService;
use defmt::info;
use embassy_executor::{Spawner, task};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::gpio::{DriveMode, Input, InputConfig, Level, Output, OutputConfig, Pull};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embedded_time::rate::Hertz;
use esp_hal::timer::timg::TimerGroup;

mod devices;
use devices::Controller;
use devices::cc1101::Cc1101;
use devices::cc1101_driver::CC1101Driver;
use devices::display::DisplaySsd1306;
use devices::ir::Infrared;

mod ui;
use ui::Style;
use ui::views::view::{ViewContext, ViewType, Viewable};
use ui::{App, app};

mod services;
use services::cli::CliService;
use services::controller::ControllerService;
use services::infrared::InfraredService;
use services::router::{RouterEvent, RouterService};

use esp_hal::ledc::channel::{self, ChannelIFace};
use esp_hal::ledc::timer::{self, TimerIFace};
use esp_hal::ledc::{self, LSGlobalClkSource, Ledc, LowSpeed};

use esp_hal::i2c::master::I2c;
use esp_hal::{i2c::master::Config as I2cConfig, time::Rate};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

use esp_hal::spi::{
    BitOrder as SpiBitOrder, Mode as SpiMode,
    master::{Config as SpiConfig, Spi},
};

use core::{net::Ipv4Addr, str::FromStr};

use embassy_time::{Duration, Timer};

use embassy_net::{
    IpListenEndpoint, Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4, tcp::TcpSocket,
};

use esp_hal::rng::Rng;
use esp_radio::wifi::{AccessPointConfig, ModeConfig, WifiDevice};

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[task]
pub async fn router_task(mut service: RouterService<'static>) {
    loop {
        service.run().await;
    }
}

#[task]
pub async fn controller_task(mut service: ControllerService<Input<'static>>) {
    loop {
        service.run().await;
    }
}

#[task]
pub async fn cli_task(mut service: CliService<'static>) {
    loop {
        service.run().await;
    }
}

#[task]
pub async fn infrared_task(led: Output<'static>, mut ledc: Ledc<'static>, rx: Input<'static>) {
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0.configure(timer::config::Config {
        duty: timer::config::Duty::Duty5Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: Rate::from_khz(38),
    });

    let mut channel0 = ledc.channel(channel::Number::Channel0, led);
    channel0.configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 10,
        drive_mode: DriveMode::PushPull,
    });

    let ir = Infrared::new(channel0, rx);

    let mut service = InfraredService::new(RouterService::event_sender(), ir);

    loop {
        service.run().await;
    }
}

#[embassy_executor::task]
async fn run_dhcp(stack: Stack<'static>, gw_ip_addr: &'static str) {
    use core::net::{Ipv4Addr, SocketAddrV4};

    use edge_dhcp::{
        io::{self, DEFAULT_SERVER_PORT},
        server::{Server, ServerOptions},
    };
    use edge_nal::UdpBind;
    use edge_nal_embassy::{Udp, UdpBuffers};

    let ip = Ipv4Addr::from_str(gw_ip_addr).expect("dhcp task failed to parse gw ip");

    let mut buf = [0u8; 1500];

    let mut gw_buf = [Ipv4Addr::UNSPECIFIED];

    let buffers = UdpBuffers::<3, 1024, 1024, 10>::new();
    let unbound_socket = Udp::new(stack, &buffers);
    let mut bound_socket = unbound_socket
        .bind(core::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            DEFAULT_SERVER_PORT,
        )))
        .await
        .unwrap();

    loop {
        _ = io::server::run(
            &mut Server::<_, 64>::new_with_et(ip),
            &ServerOptions::new(ip, Some(&mut gw_buf)),
            &mut bound_socket,
            &mut buf,
        )
        .await
        .inspect_err(|e| info!("DHCP server error"));
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[task]
pub async fn radio_task(
    mut service: RadioService<
        ExclusiveDevice<Spi<'static, esp_hal::Async>, Output<'static>, embassy_time::Delay>,
        Output<'static>,
        Input<'static>,
    >,
) {
    loop {
        service.run().await;
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default();
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    // Wifi configuration
    let radio_init = mk_static!(
        esp_radio::Controller,
        esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller")
    );
    // let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let (mut controller, interfaces) =
        esp_radio::wifi::new(radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let device = interfaces.ap;

    let gw_ip_addr_str = "192.168.2.1";
    let gw_ip_addr = Ipv4Addr::from_str(gw_ip_addr_str).unwrap();

    let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(gw_ip_addr, 24),
        gateway: Some(gw_ip_addr),
        dns_servers: Default::default(),
    });

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let (stack, runner) = embassy_net::new(
        device,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    let ap_config = ModeConfig::AccessPoint(
        AccessPointConfig::default().with_ssid("swiss-army-esp".to_string()),
    );

    controller.set_config(&ap_config).unwrap();
    controller.start_async().await.unwrap();

    spawner.spawn(net_task(runner)).ok();
    spawner.spawn(run_dhcp(stack, gw_ip_addr_str)).ok();

    stack.wait_config_up().await;

    stack.config_v4().unwrap();

    // Display Instantiation
    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(400));
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .unwrap()
        .with_scl(peripherals.GPIO15)
        .with_sda(peripherals.GPIO16)
        .into_async();
    let interface = I2CDisplayInterface::new(i2c);
    let mut target = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    target.init().unwrap();

    let mut display = DisplaySsd1306::new(target);

    // Controller Instantiation
    let mut controller = Controller::new(
        Input::new(
            peripherals.GPIO5,
            InputConfig::default().with_pull(Pull::Up),
        ),
        Input::new(
            peripherals.GPIO4,
            InputConfig::default().with_pull(Pull::Up),
        ),
        Input::new(
            peripherals.GPIO6,
            InputConfig::default().with_pull(Pull::Up),
        ),
        Input::new(
            peripherals.GPIO7,
            InputConfig::default().with_pull(Pull::Up),
        ),
    );

    let controller_service = ControllerService::new(RouterService::event_sender(), controller);

    // Infrared Instantiation
    let led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());
    let ir_rx = Input::new(peripherals.GPIO3, InputConfig::default());
    let mut ledc = Ledc::new(peripherals.LEDC);

    // cc1101 instantiation
    let cc1101_csn = Output::new(peripherals.GPIO9, Level::High, OutputConfig::default());
    let cc1101_tx = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
    let cc1101_rx = Input::new(peripherals.GPIO11, InputConfig::default());

    let spi_config = SpiConfig::default()
        .with_frequency(Rate::from_mhz(5))
        .with_mode(SpiMode::_0)
        .with_read_bit_order(SpiBitOrder::MsbFirst)
        .with_write_bit_order(SpiBitOrder::MsbFirst);

    let spi = Spi::new(peripherals.SPI2, spi_config)
        .unwrap()
        .with_sck(peripherals.GPIO12)
        .with_mosi(peripherals.GPIO13)
        .with_miso(peripherals.GPIO14)
        .into_async();

    let spi_cc1101 = ExclusiveDevice::new(spi, cc1101_csn, embassy_time::Delay).unwrap();

    use devices::cc1101_driver::CC1101Driver;
    let cc1101_driver = CC1101Driver::new(spi_cc1101, Hertz::new(26000000)).await;
    let mut cc1101 = Cc1101::new(cc1101_driver, cc1101_tx, cc1101_rx)
        .await
        .unwrap();
    let radio_service = RadioService::new(RouterService::event_sender(), cc1101);

    // Cli Service
    let cli_service = CliService::new(RouterService::command_sender(), stack);

    // Router Service
    let router_service = RouterService::new(
        app::event_sender(),
        CliService::event_sender(),
        ControllerService::<Input>::command_sender(),
        InfraredService::<ledc::channel::Channel<'static, LowSpeed>, Input>::command_sender(),
        RadioService::<
            ExclusiveDevice<Spi<'_, esp_hal::Async>, Output<'_>, embassy_time::Delay>,
            Output,
            Input,
        >::command_sender(),
    );

    // Starting Tasks
    info!("Starting Router Task");
    spawner.spawn(router_task(router_service)).unwrap();
    info!("Starting Controller Task");
    spawner.spawn(controller_task(controller_service)).unwrap();
    info!("Starting Infrared Task");
    spawner.spawn(infrared_task(led, ledc, ir_rx)).unwrap();
    info!("Starting CC1101 Task");
    spawner.spawn(radio_task(radio_service)).unwrap();
    info!("Starting Cli Task");
    spawner.spawn(cli_task(cli_service)).unwrap();

    let style = Style::normal();

    let mut app = App::new(&style, &mut display);

    app.start(ViewType::MainMenuView).await;

    loop {
        Timer::after(Duration::from_millis(2000)).await;
    }
}
