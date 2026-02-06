
use embedded_hal::i2c::Operation;
use embedded_hal::i2c::{AddressMode, SevenBitAddress};
use embedded_hal_async::i2c::I2c;
use core::convert::Infallible;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

/// Mock host I2C device
#[derive(Clone)]
pub struct MockI2cDevice {
    pub address: u16,
    pub memory: Vec<u8>,
}

impl MockI2cDevice {
    pub fn new(address: u16, size: usize) -> Self {
        Self {
            address,
            memory: vec![0; size],
        }
    }
}

/// Host I2C bus (can hold multiple devices)
#[derive(Clone)]
pub struct HostI2c {
    devices: HashMap<u16, MockI2cDevice>,
}

impl HostI2c {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    pub fn add_device(&mut self, device: MockI2cDevice) {
        self.devices.insert(device.address, device);
    }

    fn get_device_mut(&mut self, address: u16) -> Option<&mut MockI2cDevice> {
        self.devices.get_mut(&address)
    }
}

impl embedded_hal::i2c::ErrorType for HostI2c {
    type Error = Infallible;
}

#[async_trait::async_trait(?Send)]
impl<A: AddressMode + Send> I2c<A> for HostI2c {
    async fn transaction(
        &mut self,
        address: A,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        let addr = address.to_u16();

        if let Some(device) = self.get_device_mut(addr) {
            for op in operations.iter_mut() {
                match op {
                    Operation::Read(buf) => {
                        // just copy the first N bytes
                        let len = buf.len().min(device.memory.len());
                        buf[..len].copy_from_slice(&device.memory[..len]);
                        // simulate delay
                        sleep(Duration::from_millis(10)).await;
                    }
                    Operation::Write(buf) => {
                        let len = buf.len().min(device.memory.len());
                        device.memory[..len].copy_from_slice(&buf[..len]);
                        sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        } else {
            // Infallible version always succeeds; in real code you could Err here
        }

        Ok(())
    }
}
