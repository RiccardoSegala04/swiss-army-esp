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
use defmt::{error, info, unwrap};
use embassy_executor::{Spawner, task};
use embassy_time::Delay;
use embassy_time::Duration;
use embassy_time::Timer;
use embedded_time::rate::Hertz;
use esp_hal::gpio::{DriveMode, Input, InputConfig, Level, Output, OutputConfig, Pull};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_hal::timer::timg::TimerGroup;

mod devices;
use devices::Controller;
use devices::display::DisplaySsd1306;
use devices::ir::Infrared;

mod ui;
use ui::App;
use ui::Style;
use ui::views::view::{ViewContext, ViewType, Viewable};

mod services;
use services::controller::ControllerService;
use services::infrared::InfraredService;
use services::router::{RouterEvent, RouterService};

use esp_hal::ledc::channel::{self, ChannelIFace};
use esp_hal::ledc::timer::{self, TimerIFace};
use esp_hal::ledc::{self, LSGlobalClkSource, Ledc, LowSpeed};

use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

use core::{net::Ipv4Addr, str::FromStr};

use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};

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

use esp_hal::spi::{
    BitOrder as SpiBitOrder, Mode as SpiMode,
    master::{Config as SpiConfig, Spi},
};

use embedded_hal_bus::spi::ExclusiveDevice;

use crate::devices::cc1101::Cc1101;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, RouterEvent, 8> = Channel::new();

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

    let mut service = InfraredService::new(EVENT_CHANNEL.dyn_sender(), ir);
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

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default();
    let mut peripherals = esp_hal::init(config);

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

    let ap_config =
        ModeConfig::AccessPoint(AccessPointConfig::default().with_ssid("esp-radio".to_string()));

    controller.set_config(&ap_config).unwrap();
    controller.start_async().await.unwrap();

    spawner.spawn(net_task(runner)).ok();
    spawner.spawn(run_dhcp(stack, gw_ip_addr_str)).ok();

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
            peripherals.GPIO4,
            InputConfig::default().with_pull(Pull::Up),
        ),
        Input::new(
            peripherals.GPIO5,
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

    let spi_config = SpiConfig::default()
        .with_frequency(Rate::from_mhz(5))
        .with_mode(SpiMode::_0)
        .with_read_bit_order(SpiBitOrder::MsbFirst)
        .with_write_bit_order(SpiBitOrder::MsbFirst);

    let _spi = Spi::new(peripherals.SPI2, spi_config)
        .unwrap()
        .with_sck(peripherals.GPIO12)
        .with_mosi(peripherals.GPIO10)
        .with_miso(peripherals.GPIO11)
        .into_async();

    let mut cc1101_csn = Output::new(peripherals.GPIO9, Level::High, OutputConfig::default());

    let spi_cc1101 = ExclusiveDevice::new(_spi, cc1101_csn, Delay).unwrap();

    use devices::cc1101_driver::CC1101Driver;
    let cc1101_driver = CC1101Driver::new(spi_cc1101, Hertz::new(26000000)).await;
    let mut cc1101 = Cc1101::new(cc1101_driver).await.unwrap();

    info!("cc1101 initialized");
    info!(
        "Cc1101 version: {}",
        cc1101
            .chip
            .read_status(devices::cc1101_driver::StatusReg::VERSION)
            .await
            .unwrap()
    );

    /*
        let led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());
        let ir_rx = Input::new(peripherals.GPIO3, InputConfig::default());

        let mut ledc = Ledc::new(peripherals.LEDC);

    >>>>>>> Stashed changes
        let controller_service = ControllerService::new(EVENT_CHANNEL.dyn_sender(), controller);

        // Infrared Instantiation
        let led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());
        let ir_rx = Input::new(peripherals.GPIO3, InputConfig::default());
        let mut ledc = Ledc::new(peripherals.LEDC);


        // Router Service
        let router_service = RouterService::new(
            ControllerService::<Input>::command_sender(),
            InfraredService::<ledc::channel::Channel<'static, LowSpeed>, Input>::command_sender(),
        );

        // Context Instantiation
        let receiver = EVENT_CHANNEL.dyn_receiver();
        let mut ctx = ViewContext::new(&mut display, receiver, RouterService::command_sender());

        // Starting Tasks
        info!("Starting Router Task");
        spawner.spawn(router_task(router_service)).unwrap();
        info!("Starting Controller Task");
        spawner.spawn(controller_task(controller_service)).unwrap();
        info!("Starting Infrared Task");
        spawner.spawn(infrared_task(led, ledc, ir_rx)).unwrap();


        let style = Style::normal();

        let mut app = App::new(&style, ctx);

        let mut ir_rx_view = IrRxView::with_style(&style);

        info!("Starting ListView");

        // listview.run(&mut ctx).await.unwrap();
        */
    cc1101.set_frequency(Hertz::new(433000000)).await.unwrap();
    cc1101
        .set_modulation(devices::cc1101_driver::Modulation::ModOok)
        .await
        .unwrap();
    cc1101
        .send_packet(&[
            40, 34, 6, 8, 2, 3, 5, 7, 3, 2, 2, 34, 6, 8, 2, 3, 5, 7, 3, 2, 2, 34, 6, 8, 2, 3, 5, 7,
            3, 2, 2, 34, 6, 8, 2, 3, 5, 7, 3, 2, 2,
        ])
        .await
        .unwrap();

    loop {
        //    ir_rx_view.run(&mut ctx).await.unwrap();

        let state = cc1101.get_state().await.unwrap();
        info!("Cc1101 state: {:?}", state);

        Timer::after(Duration::from_millis(1000)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
