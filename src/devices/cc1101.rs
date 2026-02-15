use crate::devices::cc1101_driver::{
    CC1101Driver, Modulation, Register, State, StatusReg, StrobeCmd,
};
use defmt::{error, info};
use embassy_time::Timer;
use embedded_hal_async::spi::SpiDevice;
use embedded_time::rate;

pub struct CC1101<SPId> {
    pub chip: CC1101Driver<SPId>,
}

#[derive(Debug)]
pub enum CC1101Error<SPIe> {
    Spi(SPIe),
    InvalidVersion(u8),
    InvalidDeviation(rate::Hertz),
}

impl<E> From<E> for CC1101Error<E> {
    fn from(err: E) -> Self {
        CC1101Error::Spi(err)
    }
}

impl<SPId> CC1101<SPId>
where
    SPId: SpiDevice,
{
    pub async fn new(driver: CC1101Driver<SPId>) -> Result<Self, CC1101Error<SPId::Error>> {
        let mut chip = driver;

        let version = chip.read_status(StatusReg::VERSION).await?;
        if version != 20 {
            error!("Unexpected CC1101 version: {}", version);
            return Err(CC1101Error::InvalidVersion(version));
        }
        chip.write_reg(Register::IOCFG0, 0x06).await?; // GDO0 output pin config: Asserted when sync word is sent/received, and de-asserted at the end of the packet
        //
        chip.write_reg(Register::FIFOTHR, 0x4F).await?; // The "F" 0b1111 ensures that GDO0 assrets only if a full packet is received
        chip.write_reg(Register::MDMCFG3, 0x83).await?;
        chip.write_reg(Register::MCSM0, 0x18).await?;
        chip.write_reg(Register::FOCCFG, 0x16).await?;
        chip.write_reg(Register::AGCCTRL2, 0x43).await?;
        chip.write_reg(Register::WORCTRL, 0xFB).await?;
        chip.write_reg(Register::FSCAL3, 0xE9).await?;
        chip.write_reg(Register::FSCAL2, 0x2A).await?;
        chip.write_reg(Register::FSCAL1, 0x00).await?;
        chip.write_reg(Register::FSCAL0, 0x1F).await?;
        chip.write_reg(Register::TEST2, 0x81).await?;
        chip.write_reg(Register::TEST1, 0x35).await?;
        chip.write_reg(Register::TEST0, 0x09).await?;

        // max pkt size = 61. Dealing with larger packets is hard
        // and given the higher possibility of crc errors
        // probably not worth the effort. Generally the packets should be as
        // short as possible
        chip.write_reg(Register::PKTLEN, 61).await?; // 0x3D
        chip.write_reg(Register::MCSM1, 0x30).await?; // CCA enabled TX->IDLE RX->IDLE
        //

        Ok(Self { chip: chip })
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

    pub async fn go_powerdown(&mut self) -> Result<(), CC1101Error<SPId::Error>> {
        self.go_idle().await?;
        self.chip.strobe_cmd(StrobeCmd::SFRX).await?;
        self.chip.strobe_cmd(StrobeCmd::SFTX).await?;
        self.chip.strobe_cmd(StrobeCmd::SPWD).await?;
        Ok(())
    }

    pub async fn set_whitening(&mut self, active: bool) -> Result<(), CC1101Error<SPId::Error>> {
        self.go_idle().await?;
        self.chip.set_reg_bit(active, Register::PKTCTRL0, 6).await?;
        Ok(())
    }

    pub async fn set_frequency(
        &mut self,
        freq: rate::Hertz,
    ) -> Result<(), CC1101Error<SPId::Error>> {
        let reg_new = (((freq.0 as u64) << 16) / 26000000) as u32;
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
        const XOSC: f64 = 26000000.0;

        const DEV_MIN: f64 = (XOSC as f64 / (1 << 17) as f64) * (8.0 + 0.0) * 1.0;
        const DEV_MAX: f64 = (XOSC as f64 / (1 << 17) as f64) * (8.0 + 7.0) * (1 << 7) as f64;

        if (dev.0 as f64) < DEV_MIN || (dev.0 as f64) > DEV_MAX {
            error!(
                "Invalid deviation: {}. Valid range is {} - {}",
                dev.0, DEV_MIN, DEV_MAX
            );
            return Err(CC1101Error::InvalidDeviation(dev));
        }

        let mut best_e = 0;
        let mut best_m = 0;
        let mut diff = DEV_MAX;

        for e in 0..=7 {
            for m in 0..=7 {
                let t = (XOSC as f64 / (1 << 17) as f64) * (8.0 + m as f64) * (1 << e) as f64;
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
        self.go_idle().await?;
        let reg_before = self.chip.read_reg(Register::MDMCFG2).await?;
        let reg_new = (reg_before & 0x8F) | (modulation as u8);
        self.chip.write_reg(Register::MDMCFG2, reg_new).await?;
        Ok(())
    }

    pub async fn set_manchester_encoding(
        &mut self,
        active: bool,
    ) -> Result<(), CC1101Error<SPId::Error>> {
        self.go_idle().await?;
        self.chip.set_reg_bit(active, Register::MDMCFG2, 3).await?;
        Ok(())
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
}
