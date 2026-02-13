use heapless::Vec;

use embedded_hal::pwm::SetDutyCycle;

pub enum InfraredCommand {
    Listen
}

pub enum InfraredEvent {
    Signal(IrSignal)
}

#[derive(Clone)]
pub struct IrSignal {
    pub timings: Vec<u16, 128>
}

pub struct Infrared<PWM> {
    tx: PWM
}

impl<PWM> Infrared<PWM>
where
    PWM: SetDutyCycle
{
    pub fn new(tx: PWM) -> Self {
        Self { tx }
    }

    pub fn led_test(&mut self, percent: u8) {
        self.tx.set_duty_cycle_percent(percent); 
    }
    
}


