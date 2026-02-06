use core::convert::Infallible;
use std::sync::{Arc, Mutex};

use embassy_time::{Duration, Timer};

use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
use embedded_hal_async::digital::Wait;

/// Host-side simulated GPIO pin (Embassy-compatible)
#[derive(Clone)]
pub struct HostPin {
    state: Arc<Mutex<bool>>,
}

impl HostPin {
    pub fn new(initial_high: bool) -> Self {
        Self {
            state: Arc::new(Mutex::new(initial_high)),
        }
    }

    /// Host-only helper: force pin state
    pub fn set_state(&self, high: bool) {
        let mut s = self.state.lock().unwrap();
        *s = high;
    }

    fn read(&self) -> bool {
        *self.state.lock().unwrap()
    }

    fn write(&self, high: bool) {
        let mut s = self.state.lock().unwrap();
        *s = high;
    }
}

/* -------------------------------------------------------------------------- */
/* embedded-hal error                                                         */
/* -------------------------------------------------------------------------- */

impl ErrorType for HostPin {
    type Error = Infallible;
}

/* -------------------------------------------------------------------------- */
/* embedded-hal (sync) digital traits                                         */
/* -------------------------------------------------------------------------- */

impl InputPin for HostPin {
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.read())
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.read())
    }
}

impl OutputPin for HostPin {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.write(true);
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.write(false);
        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/* embedded-hal-async digital::Wait                                           */
/* -------------------------------------------------------------------------- */

impl Wait for HostPin {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        while !self.read() {
            Timer::after(Duration::from_millis(5)).await;
        }
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        while self.read() {
            Timer::after(Duration::from_millis(5)).await;
        }
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await?;
        self.wait_for_high().await?;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await?;
        self.wait_for_low().await?;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        let initial = self.read();
        loop {
            if self.read() != initial {
                break;
            }
            Timer::after(Duration::from_millis(5)).await;
        }
        Ok(())
    }
}
