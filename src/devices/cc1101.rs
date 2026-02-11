use embedded_hal_async::spi::{
    Operation::{Read, Transfer, Write},
    SpiDevice,
};

pub struct CC1101<SPId> {
    cc1101_spi: SPId,
}

impl<SPId> CC1101<SPId>
where
    SPId: SpiDevice,
{
    pub fn new(spi_dev: SPId) -> Self {
        Self {
            cc1101_spi: spi_dev,
        }
    }

    async fn write_reg(&mut self, reg: Register, value: u8) -> Result<(), <SPId>::Error> {
        let address: u8 = (reg as u8) | (RegOffset::Write as u8);

        self.cc1101_spi
            .transaction(&mut [
                Write(&[address]), //
                Write(&[value]),   //
            ])
            .await
    }

    async fn write_burst(&mut self, reg: Register, buffer: &[u8]) -> Result<(), <SPId>::Error> {
        let address: u8 = (reg as u8) | (RegOffset::WriteBurst as u8);
        self.cc1101_spi
            .transaction(&mut [
                Write(&[address]), //
                Write(buffer),     //
            ])
            .await
    }

    async fn read_reg(&mut self, reg: Register) -> Result<u8, <SPId>::Error> {
        let mut value: u8 = 0;
        let address: u8 = (reg as u8) | (RegOffset::Read as u8);
        self.cc1101_spi
            .transaction(&mut [
                Write(&[address]),  //
                Read(&mut [value]), //
            ])
            .await?;
        Ok(value)
    }

    async fn read_burst(&mut self, reg: Register, buffer: &mut [u8]) -> Result<(), <SPId>::Error> {
        let address: u8 = (reg as u8) | (RegOffset::ReadBurst as u8);

        self.cc1101_spi
            .transaction(&mut [
                Write(&[address]), //
                Read(buffer),      //
            ])
            .await
    }

    async fn read_status(&mut self, reg: StatusReg) -> Result<u8, <SPId>::Error> {
        let mut value: u8 = 0;
        let address: u8 = (reg as u8) | (RegOffset::ReadBurst as u8);
        self.cc1101_spi
            .transaction(&mut [
                Write(&[address]),  //
                Read(&mut [value]), //
            ])
            .await?;
        Ok(value)
    }

    async fn strobe_cmd(&mut self, cmd: StrobeCmd) -> Result<u8, <SPId>::Error> {
        let command: u8 = (cmd as u8);
        let mut reply: u8 = 0;
        self.cc1101_spi
            .transaction(&mut [
                Transfer(&mut [reply], &[command]), //
            ])
            .await?;
        Ok(reply)
    }
}

#[repr(u8)]
pub enum Register {
    IOCFG2 = 0x00,   // GDO2 output pin configuration
    IOCFG1 = 0x01,   // GDO1 output pin configuration
    IOCFG0 = 0x02,   // GDO0 output pin configuration
    FIFOTHR = 0x03,  // RX FIFO and TX FIFO thresholds
    SYNC1 = 0x04,    // Sync word, high INT8U
    SYNC0 = 0x05,    // Sync word, low INT8U
    PKTLEN = 0x06,   // Packet length
    PKTCTRL1 = 0x07, // Packet automation control
    PKTCTRL0 = 0x08, // Packet automation control
    ADDR = 0x09,     // Device address
    CHANNR = 0x0A,   // Channel number
    FSCTRL1 = 0x0B,  // Frequency synthesizer control
    FSCTRL0 = 0x0C,  // Frequency synthesizer control
    FREQ2 = 0x0D,    // Frequency control word, high INT8U
    FREQ1 = 0x0E,    // Frequency control word, middle INT8U
    FREQ0 = 0x0F,    // Frequency control word, low INT8U
    MDMCFG4 = 0x10,  // Modem configuration
    MDMCFG3 = 0x11,  // Modem configuration
    MDMCFG2 = 0x12,  // Modem configuration
    MDMCFG1 = 0x13,  // Modem configuration
    MDMCFG0 = 0x14,  // Modem configuration
    DEVIATN = 0x15,  // Modem deviation setting
    MCSM2 = 0x16,    // Main Radio Control State Machine configuration
    MCSM1 = 0x17,    // Main Radio Control State Machine configuration
    MCSM0 = 0x18,    // Main Radio Control State Machine configuration
    FOCCFG = 0x19,   // Frequency Offset Compensation configuration
    BSCFG = 0x1A,    // Bit Synchronization configuration
    AGCCTRL2 = 0x1B, // AGC control
    AGCCTRL1 = 0x1C, // AGC control
    AGCCTRL0 = 0x1D, // AGC control
    WOREVT1 = 0x1E,  // High INT8U Event 0 timeout
    WOREVT0 = 0x1F,  // Low INT8U Event 0 timeout
    WORCTRL = 0x20,  // Wake On Radio control
    FREND1 = 0x21,   // Front end RX configuration
    FREND0 = 0x22,   // Front end TX configuration
    FSCAL3 = 0x23,   // Frequency synthesizer calibration
    FSCAL2 = 0x24,   // Frequency synthesizer calibration
    FSCAL1 = 0x25,   // Frequency synthesizer calibration
    FSCAL0 = 0x26,   // Frequency synthesizer calibration
    RCCTRL1 = 0x27,  // RC oscillator configuration
    RCCTRL0 = 0x28,  // RC oscillator configuration
    FSTEST = 0x29,   // Frequency synthesizer calibration control
    PTEST = 0x2A,    // Production test
    AGCTEST = 0x2B,  // AGC test
    TEST2 = 0x2C,    // Various test settings
    TEST1 = 0x2D,    // Various test settings
    TEST0 = 0x2E,    // Various test settings

    //CC1101 PATABLE,TXFIFO,RXFIFO
    PATABLE = 0x3E,
    TXRX_FIFO = 0x3F,
}

#[repr(u8)]
pub enum StatusReg {
    // CC1101 STATUS REGISTERS
    PARTNUM = 0x30,
    VERSION = 0x31,
    FREQEST = 0x32,
    LQI = 0x33,
    RSSI = 0x34,
    MARCSTATE = 0x35,
    WORTIME1 = 0x36,
    WORTIME0 = 0x37,
    PKTSTATUS = 0x38,
    VCO_VC_DAC = 0x39,
    TXBYTES = 0x3A,
    RXBYTES = 0x3B,
    RCCTRL1_STATUS = 0x3C,
    RCCTRL2_STATUS = 0x3D,
}

#[repr(u8)]
enum RegOffset {
    Write = 0x00,
    WriteBurst = 0x40,
    Read = 0x80,
    ReadBurst = 0xC0,
}

#[repr(u8)]
pub enum StrobeCmd {
    // CC1101 Strobe commands
    SRES = 0x30,    // Reset chip.
    SFSTXON = 0x31, // Enable and calibrate frequency synthesizer (if MCSM0.FS_AUTOCAL=1).
    // If in RX/TX: Go to a wait state where only the synthesizer is
    // running (for quick RX / TX turnaround).
    SXOFF = 0x32, // Turn off crystal oscillator.
    SCAL = 0x33,  // Calibrate frequency synthesizer and turn it off
    // (enables quick start).
    SRX = 0x34, // Enable RX. Perform calibration first if coming from IDLE and
    // MCSM0.FS_AUTOCAL=1.
    STX = 0x35, // In IDLE state: Enable TX. Perform calibration first if
    // MCSM0.FS_AUTOCAL=1. If in RX state and CCA is enabled:
    // Only go to TX if channel is clear.
    SIDLE = 0x36, // Exit RX / TX, turn off frequency synthesizer and exit
    // Wake-On-Radio mode if applicable.
    SAFC = 0x37,    // Perform AFC adjustment of the frequency synthesizer
    SWOR = 0x38,    // Start automatic RX polling sequence (Wake-on-Radio)
    SPWD = 0x39,    // Enter power down mode when CSn goes high.
    SFRX = 0x3A,    // Flush the RX FIFO buffer.
    SFTX = 0x3B,    // Flush the TX FIFO buffer.
    SWORRST = 0x3C, // Reset real time clock.
    SNOP = 0x3D,    // No operation. May be used to pad strobe commands to two
                    // INT8Us for simpler software.
}
