use crate::devices::cc1101_driver::{CC1101Driver, Register, State, StatusReg, StrobeCmd};
use embedded_hal_async::spi::SpiDevice;
use embedded_time::rate;

pub struct CC1101<SPId> {
    chip: CC1101Driver<SPId>,
}

pub enum CC1101Error<SPIe> {
    Spi(SPIe),
    InvalidVersion(u8),
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
            return Err(CC1101Error::InvalidVersion(version));
        }

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
        self.chip.set_reg_bit(active, Register::PKTCTRL0).await?;
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
}
