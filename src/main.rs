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
use esp_hal::gpio::Output;
use esp_hal::timer::timg::TimerGroup;

mod devices;
use devices::Controller;
use devices::Display;

mod services;
use services::controller_service;

use esp_hal::{i2c::master::Config as I2cConfig, time::Rate};
use esp_hal::i2c::master::I2c;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use embedded_graphics::{
    Drawable,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
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

    let mut display = Display::new(target);

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let text = "Palle";
    let t1 = Text::with_alignment(
        text,
        Point::new(64, 15),
        character_style,
        Alignment::Center,
    );

    let text = "Nere";
    let t2 = Text::with_alignment(
        text,
        Point::new(64, 30),
        character_style,
        Alignment::Center,
    );

    let drawable = [t1, t2];

    display.draw_all(drawable.iter());
    
    
    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
