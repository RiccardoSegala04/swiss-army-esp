use heapless::Vec;

use defmt::info;

use embedded_hal::pwm::SetDutyCycle;

use embassy_time::{Duration, Timer};

pub enum InfraredCommand {
    Listen,
    Play(IrSignal)
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
    pub fn new(mut tx: PWM) -> Self {
        tx.set_duty_cycle_fully_off();
        Self { tx }
    }

    fn tx_on(&mut self) {
        self.tx.set_duty_cycle_percent(50); 
    }

    fn tx_off(&mut self) {
        self.tx.set_duty_cycle_fully_off(); 
    }

    fn tx_set(&mut self, tx: bool) {
        if tx {
            self.tx_on();
        } else {
            self.tx_off();
        }
    }

    pub async fn transmit(&mut self, signal: &IrSignal) {
        let mut tx = true;
        for sample in &signal.timings {
           
            self.tx_set(tx);
            tx = !tx;

            Timer::after(Duration::from_micros(*sample as u64)).await;            
        }

        self.tx_off();
    }
   
}


