use embedded_hal_async::spi::SpiDevice;

use crate::devices::cc1101_driver::{CC1101Driver, Register, StatusReg, StrobeCmd};
use embedded_time::rate;

pub struct CC1101<SPId> {
    chip: CC1101Driver<SPId>,
}

impl<SPId> CC1101<SPId>
where
    SPId: SpiDevice,
{
    pub async fn new(driver: CC1101Driver<SPId>) -> Result<Self, ()> {
        let mut chip = driver;
        let version = chip.read_status(StatusReg::VERSION).await.map_err(|_| ())?;
        if version != 20 {
            Err(())?
        }

        Ok(Self { chip: chip })
    }

    pub async fn get_state(&mut self) -> Result<State, ()> {
        let mut previous = self
            .chip
            .strobe_cmd(StrobeCmd::SNOP)
            .await
            .map_err(|_| ())?;

        loop {
            let current = self
                .chip
                .strobe_cmd(StrobeCmd::SNOP)
                .await
                .map_err(|_| ())?;
            if current == previous {
                return State::try_from((current >> 4) & 0b00111);
            }
            previous = current;
        }
    }

    pub async fn go_idle(&mut self) -> Result<(), ()> {
        self.chip
            .strobe_cmd(StrobeCmd::SIDLE)
            .await
            .map_err(|_| ())?;
        while self.get_state().await? != State::Idle {}
        Ok(())
    }

    pub async fn go_powerdown(&mut self) -> Result<(), ()> {
        self.go_idle().await?;
        self.chip
            .strobe_cmd(StrobeCmd::SFRX)
            .await
            .map_err(|_| ())?;
        self.chip
            .strobe_cmd(StrobeCmd::SFTX)
            .await
            .map_err(|_| ())?;
        self.chip
            .strobe_cmd(StrobeCmd::SPWD)
            .await
            .map_err(|_| ())?;
        Ok(())
    }

    pub async fn set_whitening(&mut self, active: bool) -> Result<(), ()> {
        self.go_idle().await?;
        self.chip
            .set_reg_bit(active, Register::PKTCTRL0)
            .await
            .map_err(|_| ())
    }

    pub async fn set_frequency(&mut self, freq: rate::Hertz) -> Result<(), ()> {
        let reg_new = (((freq.0 as u64) << 16) / 26000000) as u32;
        self.go_idle().await?;
        self.chip
            .write_reg(Register::CHANNR, 0)
            .await
            .map_err(|_| ())?;
        self.chip
            .write_reg(Register::FREQ2, ((reg_new >> 16) & 0xFF) as u8)
            .await
            .map_err(|_| ())?;
        self.chip
            .write_reg(Register::FREQ1, ((reg_new >> 8) & 0xFF) as u8)
            .await
            .map_err(|_| ())?;
        self.chip
            .write_reg(Register::FREQ0, ((reg_new >> 0) & 0xFF) as u8)
            .await
            .map_err(|_| ())?;
        Ok(())
    }

    pub async fn set_sync_word(&mut self, word: u16) -> Result<(), ()> {
        self.go_idle().await?;
        self.chip
            .write_reg(Register::SYNC0, ((word >> 0) & 0xFF) as u8)
            .await
            .map_err(|_| ())?;
        self.chip
            .write_reg(Register::SYNC1, ((word >> 8) & 0xFF) as u8)
            .await
            .map_err(|_| ())?;
        Ok(())
    }
}

#[derive(PartialEq)]
pub enum State {
    Idle = 0b000,
    Rx = 0b001,
    Tx = 0b010,
    FsTxOn = 0b011,
    Calibrate = 0b100,
    PllSettling = 0b101,
    RxFifoUnderflow = 0b110,
    TxFifoUnderflow = 0b111,
}

impl TryFrom<u8> for State {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b000 => Ok(State::Idle),
            0b001 => Ok(State::Rx),
            0b010 => Ok(State::Tx),
            0b011 => Ok(State::FsTxOn),
            0b100 => Ok(State::Calibrate),
            0b101 => Ok(State::PllSettling),
            0b110 => Ok(State::RxFifoUnderflow),
            0b111 => Ok(State::TxFifoUnderflow),
            _ => Err(()),
        }
    }
}
