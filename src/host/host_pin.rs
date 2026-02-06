use embedded_hal_async::digital::Wait;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use core::convert::Infallible;

/// Simulated host pin.
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

    pub async fn set_state(&self, high: bool) {
        let mut s = self.state.lock().unwrap();
        *s = high;
    }

    fn read(&self) -> bool {
        *self.state.lock().unwrap()
    }
}

impl embedded_hal::digital::ErrorType for HostPin {
    type Error = Infallible;
}

impl Wait for HostPin {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        while !self.read() {
            sleep(Duration::from_millis(10)).await;
        }
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        while self.read() {
            sleep(Duration::from_millis(10)).await;
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
            sleep(Duration::from_millis(10)).await;
        }
        Ok(())
    }
}
