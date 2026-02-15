use heapless::Vec;

use defmt::info;

use embedded_hal::pwm::SetDutyCycle;

use embassy_time::{Duration, Instant, Timer};

use embassy_futures::select::{Either, select};

use embedded_hal_async::digital::Wait;

use embedded_hal::digital::InputPin;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::lazy_lock::LazyLock;
use embassy_sync::mutex::Mutex;

pub static SIGNAL_HISTORY: LazyLock<Mutex<CriticalSectionRawMutex, Vec<IrSignal, 5>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub enum InfraredCommand {
    Listen,
    Play(IrSignal),
}

#[derive(Clone)]
pub enum InfraredEvent {
    Signal(IrSignal),
    SignalTooLong,
    NoSignal,
    SignalPlayed,
}

#[derive(Clone)]
pub struct IrSignal {
    pub timings: Vec<u16, 256>,
    level_high: bool,
}

impl IrSignal {
    pub fn new() -> Self {
        Self {
            timings: Vec::new(),
            level_high: true,
        }
    }

    pub fn with_timings(timings: Vec<u16, 256>) -> Self {
        Self {
            timings: timings,
            level_high: true,
        }
    }

    fn push_timing(&mut self, timing: u16) -> Result<(), u16> {
        self.timings.push(timing)
    }

    pub fn is_empty(&self) -> bool {
        self.timings.is_empty()
    }
}

pub struct Infrared<PWM, InPin> {
    tx: PWM,
    rx: InPin,
}

impl<PWM, InPin> Infrared<PWM, InPin>
where
    PWM: SetDutyCycle,
    InPin: InputPin + Wait,
{
    pub fn new(mut tx: PWM, rx: InPin) -> Self {
        tx.set_duty_cycle_fully_off();
        Self { tx, rx }
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

    pub async fn listen(&mut self) -> InfraredEvent {
        let mut signal = IrSignal::new();
        let mut last_edge: Option<Instant> = None;

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
                                    return InfraredEvent::SignalTooLong;
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

        if signal.is_empty() {
            return InfraredEvent::NoSignal;
        }

        info!("Signal length: {}", signal.timings.len());

        SIGNAL_HISTORY.get().lock().await.push(signal.clone());

        InfraredEvent::Signal(signal)
    }
}
