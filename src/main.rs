#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::peripherals::Peripherals;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

mod devices;
use devices::controller;
use devices::display::{Display, DisplaySsd1306};

mod ui;
use ui::views::DummyView;
use ui::views::ListView;

mod services;
use embassy_sync::channel;
use services::controller_service;
use services::service_router;

use esp_hal::i2c::master::I2c;
use esp_hal::{i2c::master::Config as I2cConfig, time::Rate};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    // let timg0 = TimerGroup::new(peripherals.TIMG0);
    // esp_rtos::start(timg0.timer0);

    // let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    // let (mut _wifi_controller, _interfaces) =
    //     esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
    //         .expect("Failed to initialize Wi-Fi controller");

    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(400));
    let i2c = I2c::new(p.I2C0, i2c_config)
        .unwrap()
        .with_scl(p.GPIO32)
        .with_sda(p.GPIO33)
        .into_async();
    let interface = I2CDisplayInterface::new(i2c);
    let mut target = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    target.init().unwrap();

    let mut display = DisplaySsd1306::new(target);

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let text = "Palle";
    let t1 = Text::with_alignment(text, Point::new(64, 15), character_style, Alignment::Center);

    let text = "Nere";
    let t2 = Text::with_alignment(text, Point::new(64, 30), character_style, Alignment::Center);

    let drawable = [t1, t2];

    display.clear();
    display.draw_all(drawable.iter());
    display.flush();

    //static COM_CHANNEL: channel::Channel<
    //    CriticalSectionRawMutex,
    //    service_router::ServiceRouterCommand,
    //    1,
    //> = channel::Channel::new();
    //static EV_CHANNEL: channel::Channel<
    //    CriticalSectionRawMutex,
    //    service_router::ServiceRouterEvent,
    //    1,
    //> = channel::Channel::new();
    //static CONTR_COMMANDS: channel::Channel<
    //    CriticalSectionRawMutex,
    //    controller_service::ControllerCommand,
    //    1,
    //> = channel::Channel::new();

    #[embassy_executor::task]
    async fn command_router_task() -> ! {
        let controller_commands = controller_service::ControllerService::<
            Output<'static>,
            Input<'static>,
        >::commands_sender();

        //let IR_commands = IR_service::IRService::<
        //    Output<'static>,
        //    Input<'static>,
        //>::commands_sender();

        let mut com_router =
            service_router::CommandRouter::new(controller_commands /* , IR_commands)*/);
        com_router.run().await
    } //  ui -> services

    let cont = controller::Controller::new(
        (
            Output::new(p.GPIO12, Level::Low, OutputConfig::default()),
            Output::new(p.GPIO14, Level::Low, OutputConfig::default()),
            Output::new(p.GPIO27, Level::Low, OutputConfig::default()),
        ),
        Input::new(p.GPIO13, InputConfig::default()),
    );

    #[embassy_executor::task]
    async fn controller_service_task(
        controller: controller::Controller<Output<'static>, Input<'static>>,
    ) -> ! {
        let events = service_router::EventRouter::events_sender(); // services -> ui

        let mut controller_service = controller_service::ControllerService::new(events, controller);
        controller_service.run().await
    }

    spawner.spawn(command_router_task()).unwrap();

    spawner.spawn(controller_service_task(cont)).unwrap();
    //#[embassy_executor::task]
    //async fn command_router_task(mut com_router: service_router::CommandRouter<'_>) -> ! {
    //    com_router.run().await
    //}

    //spawner.spawn(command_router_task(com_router)).unwrap();

    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
