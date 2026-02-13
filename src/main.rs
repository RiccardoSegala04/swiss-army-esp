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

use defmt::info;
use embassy_executor::{Spawner, task};
use esp_hal::gpio::{Input, InputConfig, Pull, DriveMode, Output, OutputConfig, Level};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_hal::timer::timg::TimerGroup;

mod devices;
use devices::Controller;
use devices::ir::Infrared;
use devices::display::DisplaySsd1306;

mod ui;
use ui::Style;
use ui::views::DummyView;
use ui::views::IrRxView;
use ui::views::ListView;
use ui::views::view::{Viewable, ViewContext};

mod services;
use services::controller::ControllerService;
use services::router::{RouterEvent, RouterService};
use services::infrared::InfraredService;


use esp_hal::ledc::{self, Ledc, LSGlobalClkSource, LowSpeed}; 
use esp_hal::ledc::channel::{self, ChannelIFace};
use esp_hal::ledc::timer::{self, TimerIFace};

use esp_hal::i2c::master::I2c;
use esp_hal::{i2c::master::Config as I2cConfig, time::Rate};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

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
pub async fn infrared_task(led: Output<'static>, mut ledc: Ledc<'static>) {

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

    let ir = Infrared::new(channel0);
    let mut service = InfraredService::new(EVENT_CHANNEL.dyn_sender(), ir);
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

    let led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());

    let mut ledc = Ledc::new(peripherals.LEDC);

    let controller_service = ControllerService::new(EVENT_CHANNEL.dyn_sender(), controller);

    let router_service = RouterService::new(ControllerService::<Input>::command_sender(), InfraredService::<ledc::channel::Channel<'static, LowSpeed>>::command_sender());

    let receiver = EVENT_CHANNEL.dyn_receiver();
    let mut ctx = ViewContext::new(&mut display, receiver, RouterService::command_sender());

    info!("Starting Router Task");
    spawner.spawn(router_task(router_service)).unwrap();


    info!("Starting Controller Task");
    spawner.spawn(controller_task(controller_service)).unwrap();
    info!("Starting Infrared Task");
    spawner.spawn(infrared_task(led, ledc)).unwrap();


    // let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    // let (mut _wifi_controller, _interfaces) =
    //     esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
    //         .expect("Failed to initialize Wi-Fi controller");

    let style = Style::normal();

    let mut views = [
        DummyView::new("View1").into(),
        DummyView::new("View2").into(),
        DummyView::new("View3").into(),
        DummyView::new("View4").into(),
    ];
    let mut listview = ListView::new(&style, "TITLE", &mut views);

    let mut ir_rx_view = IrRxView::with_style(&style);


    info!("Starting ListView");


    // listview.run(&mut ctx).await.unwrap();

    loop {
        ir_rx_view.run(&mut ctx).await.unwrap();
        
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
