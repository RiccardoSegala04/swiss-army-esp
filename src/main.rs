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
use esp_hal::gpio::{Input, InputConfig, Pull};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_hal::timer::timg::TimerGroup;

mod devices;
use devices::Controller;
use devices::display::DisplaySsd1306;

mod ui;
use ui::Style;
use ui::views::DummyView;
use ui::views::IrRxView;
use ui::views::ListView;
use ui::views::view::{Viewable, ViewContext};

mod services;
use services::controller::ControllerService;
use services::router::RouterEvent;

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
pub async fn controller_task(mut service: ControllerService<Input<'static>>) {
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

    let controller_service = ControllerService::new(EVENT_CHANNEL.dyn_sender(), controller);

    info!("Starting Controller Task");
    spawner.spawn(controller_task(controller_service)).unwrap();

    // let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    // let (mut _wifi_controller, _interfaces) =
    //     esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
    //         .expect("Failed to initialize Wi-Fi controller");

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
    let style = Style::normal();

    let mut views = [
        DummyView::new("View1").into(),
        DummyView::new("View2").into(),
        DummyView::new("View3").into(),
        DummyView::new("View4").into(),
    ];
    let mut listview = ListView::new(&style, "TITLE", &mut views);

    let mut ir_rx_view = IrRxView::with_style(&style);

    let receiver = EVENT_CHANNEL.dyn_receiver();

    info!("Starting ListView");

    let mut ctx = ViewContext::new(&mut display, receiver);

    listview.run(&mut ctx).await.unwrap();

    loop {}

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
