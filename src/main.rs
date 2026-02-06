mod host;
use host::i2c::HostI2c;
use embedded_hal_async::i2c::I2c;
use embedded_hal::i2c::SevenBitAddress;

#[tokio::main]
async fn main() {
    let mut bus = HostI2c::new();
    bus.add_device(host::i2c::MockI2cDevice::new(0x42, 16));

    let mut buf = [0u8; 4];
    bus.read(SevenBitAddress::try_from(0x42).unwrap(), &mut buf).await.unwrap();
    println!("Read: {:?}", buf);

    bus.write(SevenBitAddress::try_from(0x42).unwrap(), &[1, 2, 3, 4])
        .await
        .unwrap();

    bus.read(SevenBitAddress::try_from(0x42).unwrap(), &mut buf).await.unwrap();
    println!("Read after write: {:?}", buf);
}
