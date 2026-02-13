use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

use embassy_futures::select::*;
use embassy_time::{Duration, Timer};

pub enum ControllerEvent {
    ConfirmPressed,
    BackPressed,
    NavNextPressed,
    NavPrevPressed,
}

pub enum ControllerCommand {
    LedColor { red: u8, green: u8, blue: u8 },
}

pub struct Controller<InPin> {
    confirm: InPin,
    back: InPin,

    nav_next: InPin,
    nav_prev: InPin,
}

impl<InPin> Controller<InPin>
where
    InPin: InputPin + Wait,
{
    pub fn new(confirm: InPin, back: InPin, nav_prev: InPin, nav_next: InPin) -> Self {
        Self {
            confirm: confirm,
            back: back,
            nav_next: nav_next,
            nav_prev: nav_prev,
        }
    }

    // pub fn set_led(&mut self, red: u8, green: u8, blue: u8) -> Result<(), PWMpin::Error> {
    //     self.rgb_led.0.set_duty_cycle_percent(red)?;
    //     self.rgb_led.1.set_duty_cycle_percent(green)?;
    //     self.rgb_led.2.set_duty_cycle_percent(blue)?;
    //     Ok(())
    // }

    pub fn is_confirm_pressed(&mut self) -> Result<bool, InPin::Error> {
        self.confirm.is_low()
    }

    pub fn is_back_pressed(&mut self) -> Result<bool, InPin::Error> {
        self.back.is_low()
    }

    pub fn is_nav_next_pressed(&mut self) -> Result<bool, InPin::Error> {
        self.nav_next.is_low()
    }

    pub fn is_nav_prev_pressed(&mut self) -> Result<bool, InPin::Error> {
        self.nav_prev.is_low()
    }

    pub async fn poll_events(&mut self) -> ControllerEvent {
        let confirm = wait_for_falling_edge_debounced(&mut self.confirm, 100);
        let back = wait_for_falling_edge_debounced(&mut self.back, 100);
        let nav_next = wait_for_falling_edge_debounced(&mut self.nav_next, 100);
        let nav_prev = wait_for_falling_edge_debounced(&mut self.nav_prev, 100);

        match select4(confirm, back, nav_next, nav_prev).await {
            Either4::First(_) => ControllerEvent::ConfirmPressed,
            Either4::Second(_) => ControllerEvent::BackPressed,
            Either4::Third(_) => ControllerEvent::NavNextPressed,
            Either4::Fourth(_) => ControllerEvent::NavPrevPressed,
        }
    }
}

async fn wait_for_falling_edge_debounced<B>(button: &mut B, debounce_ms: u64)
where
    B: Wait + InputPin,
{
    loop {
        let _ = button.wait_for_falling_edge().await;

        Timer::after(Duration::from_millis(debounce_ms)).await;

        if button.is_low().unwrap() {
            return;
        }
    }
}
