use crate::devices::cc1101_driver::{
    CC1101Driver, Modulation, Register, State, StatusReg, StrobeCmd,
};

use defmt::{error, info};
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;
use embedded_time::rate::{self, Baud, Hertz};
use heapless::Vec;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::lazy_lock::LazyLock;
use embassy_sync::mutex::Mutex;

pub static SIGNAL_HISTORY: LazyLock<Mutex<CriticalSectionRawMutex, Vec<RadioSignal, 5>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Clone)]
pub enum RadioEvent {
    Signal(RadioSignal),
    SignalTooLong,
    NoSignal,
    SignalPlayed,
    Error,
}

pub enum RadioCommand {
    Listen,
    Play(RadioSignal),
}

#[derive(Clone)]
pub struct RadioSignal {
    pub timings: Vec<u16, 256>,
    pub frequency: rate::Hertz,
    pub modulation: Modulation,
}

impl RadioSignal {
    pub fn new() -> Self {
        Self {
            timings: Vec::new(),
            frequency: Hertz(433_000_000),
            modulation: Modulation::ModOok,
        }
    }

    pub fn with_timings(self, timings: Vec<u16, 256>) -> Self {
        let mut copied = self;
        copied.timings = timings;
        copied
    }

    pub fn with_frequency(self, freq: rate::Hertz) -> Self {
        let mut copied = self;
        copied.frequency = freq;
        copied
    }

    pub fn with_modulation(self, modulation: Modulation) -> Self {
        let mut copied = self;
        copied.modulation = modulation;
        copied
    }

    fn push_timing(&mut self, timing: u16) -> Result<(), u16> {
        self.timings.push(timing)
    }

    pub fn is_empty(&self) -> bool {
        self.timings.is_empty()
    }
}

#[derive(Debug)]
pub enum CC1101Error<SPIe> {
    Spi(SPIe),
    InvalidVersion(u8),
    InvalidDeviation(rate::Hertz),
    InvalidBitrate(rate::Baud, Modulation),
}

impl<E> From<E> for CC1101Error<E> {
    fn from(err: E) -> Self {
        CC1101Error::Spi(err)
    }
}

pub struct Cc1101<SPId, InPin, OutPin> {
    pub chip: CC1101Driver<SPId>,
    modulation: Modulation,
    frequency: rate::Hertz,
    rx: InPin,
    tx: OutPin,
}

impl<SPId, InPin, OutPin> Cc1101<SPId, InPin, OutPin>
where
    InPin: InputPin + Wait,
    OutPin: OutputPin,
    SPId: SpiDevice,
{
    pub async fn new(
        driver: CC1101Driver<SPId>,
        tx: OutPin,
        rx: InPin,
    ) -> Result<Self, CC1101Error<SPId::Error>> {
        let mut device = Self {
            chip: driver,
            modulation: Modulation::ModOok,
            frequency: Hertz(433_000_000),
            tx: tx,
            rx: rx,
        };

        let version = device.chip.read_status(StatusReg::VERSION).await?;
        if version != 20 {
            error!("Unexpected Cc1101 version: {}", version);
            return Err(CC1101Error::InvalidVersion(version));
        }

        device
            .chip
            .write_reg_field(Register::MCSM0, 1, 5, 4)
            .await?;
        device.set_modulation(Modulation::ModOok).await?;
        device.set_manchester_encoding(false).await?;
        device.set_whitening(false).await?;
        device
            .chip
            .write_reg_field(Register::MDMCFG2, 0, 2, 0) // disable sync and preamble
            .await
            .unwrap();
        device
            .chip
            .set_reg_bit(Register::PKTCTRL0, false, 2) // disable crc
            .await
            .unwrap();
        device
            .chip
            .set_reg_bit(Register::PKTCTRL1, false, 2) // disable append status
            .await
            .unwrap();
        device
            .chip
            .write_reg_field(Register::PKTCTRL0, 3, 5, 4) // set asynchronous serial mode
            .await
            .unwrap();
        device
            .chip
            .write_reg_field(Register::IOCFG1, 0x0D, 5, 0) // set gdo1 as serial out
            .await
            .unwrap();

        Ok(device)
    }

    fn tx_on(&mut self) {
        self.tx.set_high();
    }

    fn tx_off(&mut self) {
        self.tx.set_low();
    }

    fn tx_set(&mut self, tx: bool) {
        if tx {
            self.tx_on();
        } else {
            self.tx_off();
        }
    }

    pub async fn get_state(&mut self) -> Result<State, CC1101Error<SPId::Error>> {
        let mut previous = self.chip.strobe_cmd(StrobeCmd::SNOP).await?;

        loop {
            let current = self.chip.strobe_cmd(StrobeCmd::SNOP).await?;
            if current == previous {
                return Ok(current.state());
            }
            previous = current;
        }
    }

    pub async fn go_idle(&mut self) -> Result<(), CC1101Error<SPId::Error>> {
        self.chip.strobe_cmd(StrobeCmd::SIDLE).await?;
        while self.get_state().await? != State::Idle {}
        Ok(())
    }

    pub async fn go_rx(&mut self) -> Result<(), CC1101Error<SPId::Error>> {
        self.chip.strobe_cmd(StrobeCmd::SRX).await?;
        while self.get_state().await? != State::Rx {}
        Ok(())
    }

    pub async fn go_tx(&mut self) -> Result<(), CC1101Error<SPId::Error>> {
        self.chip.strobe_cmd(StrobeCmd::STX).await?;
        while self.get_state().await? != State::Tx {}
        Ok(())
    }

    pub async fn go_powerdown(&mut self) -> Result<(), CC1101Error<SPId::Error>> {
        self.go_idle().await?;
        self.chip.strobe_cmd(StrobeCmd::SFRX).await?;
        self.chip.strobe_cmd(StrobeCmd::SFTX).await?;
        self.chip.strobe_cmd(StrobeCmd::SPWD).await?;
        Ok(())
    }

    pub async fn set_whitening(&mut self, active: bool) -> Result<(), CC1101Error<SPId::Error>> {
        self.go_idle().await?;
        self.chip.set_reg_bit(Register::PKTCTRL0, active, 6).await?;
        Ok(())
    }

    pub async fn set_frequency(
        &mut self,
        freq: rate::Hertz,
    ) -> Result<(), CC1101Error<SPId::Error>> {
        let reg_new = (((freq.0 as u64) << 16) / (self.chip.xosc_freq.0 as u64)) as u32;
        self.go_idle().await?;
        self.chip.write_reg(Register::CHANNR, 0).await?;
        self.chip
            .write_reg(Register::FREQ2, ((reg_new >> 16) & 0xFF) as u8)
            .await?;
        self.chip
            .write_reg(Register::FREQ1, ((reg_new >> 8) & 0xFF) as u8)
            .await?;
        self.chip
            .write_reg(Register::FREQ0, ((reg_new >> 0) & 0xFF) as u8)
            .await?;
        Ok(())
    }

    pub async fn set_deviation(
        &mut self,
        dev: rate::Hertz,
    ) -> Result<(), CC1101Error<SPId::Error>> {
        let dev_min: f64 = (self.chip.xosc_freq.0 as f64 / (1 << 17) as f64) * (8.0 + 0.0) * 1.0;
        let dev_min: f64 =
            (self.chip.xosc_freq.0 as f64 / (1 << 17) as f64) * (8.0 + 7.0) * (1 << 7) as f64;

        if (dev.0 as f64) < dev_min || (dev.0 as f64) > dev_min {
            error!(
                "Invalid deviation: {}. Valid range is {} - {}",
                dev.0, dev_min, dev_min
            );
            return Err(CC1101Error::InvalidDeviation(dev));
        }

        let mut best_e = 0;
        let mut best_m = 0;
        let mut diff = dev_min;

        for e in 0..=7 {
            for m in 0..=7 {
                let t = (self.chip.xosc_freq.0 as f64 / (1 << 17) as f64)
                    * (8.0 + m as f64)
                    * (1 << e) as f64;
                if ((dev.0 as f64 - t).abs()) < diff {
                    diff = (dev.0 as f64 - t).abs();
                    best_e = e;
                    best_m = m;
                }
            }
        }

        self.go_idle().await?;
        let reg_before = self.chip.read_reg(Register::DEVIATN).await?;
        let reg_new = (reg_before & 0x88) | ((best_m & 0x07) << 2) | ((best_e & 0x07) << 6);
        self.chip.write_reg(Register::DEVIATN, reg_new).await?;
        Ok(())
    }

    pub async fn set_sync_word(&mut self, word: u16) -> Result<(), CC1101Error<SPId::Error>> {
        self.go_idle().await?;
        self.chip
            .write_reg(Register::SYNC0, ((word >> 0) & 0xFF) as u8)
            .await?;
        self.chip
            .write_reg(Register::SYNC1, ((word >> 8) & 0xFF) as u8)
            .await?;
        Ok(())
    }

    pub async fn set_modulation(
        &mut self,
        modulation: Modulation,
    ) -> Result<(), CC1101Error<SPId::Error>> {
        self.modulation = modulation;
        self.go_idle().await?;
        let reg_before = self.chip.read_reg(Register::MDMCFG2).await?;
        let reg_new = (reg_before & 0x8F) | (self.modulation as u8);
        self.chip.write_reg(Register::MDMCFG2, reg_new).await?;
        Ok(())
    }

    pub async fn set_manchester_encoding(
        &mut self,
        active: bool,
    ) -> Result<(), CC1101Error<SPId::Error>> {
        self.go_idle().await?;
        self.chip.set_reg_bit(Register::MDMCFG2, active, 3).await?;
        Ok(())
    }

    pub async fn wait_fifo(&mut self) -> Result<u8, CC1101Error<SPId::Error>> {
        let mut bytes = self
            .chip
            .read_status_field(StatusReg::RXBYTES, 6, 0)
            .await?;

        while bytes == 0 {
            Timer::after(Duration::from_micros(15)).await;
            bytes = self
                .chip
                .read_status_field(StatusReg::RXBYTES, 6, 0)
                .await?;
        }
        Ok(bytes)
    }

    pub async fn send_packet(&mut self, packet: &[u8]) -> Result<(), CC1101Error<SPId::Error>> {
        if packet.len() > 61 {
            error!("Packet too long: {} bytes. Max is 61", packet.len());
            return Err(CC1101Error::InvalidVersion(packet.len() as u8));
        }
        self.go_idle().await?;
        self.chip.strobe_cmd(StrobeCmd::SFRX).await?;

        self.chip.write_burst(Register::TXRX_FIFO, packet).await?;
        self.chip.strobe_cmd(StrobeCmd::STX).await?;
        while self.get_state().await? != State::Idle {}
        Ok(())
    }

    pub async fn set_baudrate(&mut self, baudrate: Baud) -> Result<(), CC1101Error<SPId::Error>> {
        let range = self.modulation.range();
        if baudrate < range.min || baudrate > range.max {
            return Err(CC1101Error::InvalidBitrate(baudrate, self.modulation));
        }

        let num_e = ((baudrate.0 as u64) << 20) as f64;
        let den_e = self.chip.xosc_freq.0 as f64;
        let mut drate_e = fpmath::floor(fpmath::log2(num_e / den_e)) as u8;

        let mut drate_m = ((((baudrate.0 as u64) << (28 - drate_e)) as f64
            / (self.chip.xosc_freq.0 as f64))
            - 256.0) as u32;

        info!("drate_e: {}, drate_m: {}", drate_e, drate_m);

        if drate_m == 256 {
            drate_m = 0;
            drate_e += 1;
        }

        self.chip
            .write_reg_field(Register::MDMCFG4, drate_e, 3, 0)
            .await?;
        self.chip
            .write_reg(Register::MDMCFG3, drate_m as u8)
            .await?;

        Ok(())
    }

    pub async fn transmit_signal(
        &mut self,
        signal: &RadioSignal,
    ) -> Result<(), CC1101Error<SPId::Error>> {
        if self.frequency != signal.frequency {
            self.set_frequency(signal.frequency);
        }
        if self.modulation != signal.modulation {
            self.set_modulation(signal.modulation);
        }

        self.go_tx().await?;
        let mut tx = true;
        for sample in &signal.timings {
            self.tx_set(tx);
            tx = !tx;

            Timer::after(Duration::from_micros(*sample as u64)).await;
        }

        self.tx_off();
        self.go_idle().await?;

        Ok(())
    }

    pub async fn listen_signal(&mut self) -> Result<RadioEvent, CC1101Error<SPId::Error>> {
        let mut signal = RadioSignal::new()
            .with_frequency(self.frequency)
            .with_modulation(self.modulation);
        let mut last_edge: Option<Instant> = None;

        self.go_rx().await?;

        let mut timeout = Timer::after(Duration::from_millis(2000));

        loop {
            let rising = self.rx.wait_for_any_edge();

            match select(timeout, rising).await {
                Either::First(_) => {
                    break;
                }
                Either::Second(_) => {
                    last_edge = match last_edge {
                        None => Some(Instant::now()),
                        Some(last_edge) => {
                            let now = Instant::now();
                            let delta = now - last_edge;

                            match signal.push_timing(delta.as_micros().try_into().unwrap()) {
                                Err(_) => {
                                    return Ok(RadioEvent::SignalTooLong);
                                }
                                Ok(()) => {}
                            };

                            Some(now)
                        }
                    };
                }
            }
            timeout = Timer::after(Duration::from_millis(50));
        }

        self.go_idle().await?;

        if signal.is_empty() {
            return Ok(RadioEvent::NoSignal);
        }

        info!("Signal length: {}", signal.timings.len());

        Ok(RadioEvent::Signal(signal))
    }
}
