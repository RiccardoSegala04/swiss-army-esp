

use defmt::info;

use embedded_hal::pwm::SetDutyCycle;
use embedded_hal::digital::InputPin;

use embedded_hal_async::digital::Wait;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, DynamicReceiver, DynamicSender};

use crate::devices::ir::{Infrared, InfraredCommand, InfraredEvent};
use crate::services::router::RouterEvent;

pub static INFRARED_COMMANDS_CHANNEL: channel::Channel<
    CriticalSectionRawMutex,
    InfraredCommand,
    1,
> = channel::Channel::new();

pub struct InfraredService<PWM, InPin> {
    commands_receiver: DynamicReceiver<'static, InfraredCommand>,
    events_sender: DynamicSender<'static, RouterEvent>,
    ir: Infrared<PWM, InPin>,
}

impl<PWM, InPin> InfraredService<PWM, InPin>
where
    PWM: SetDutyCycle,
    InPin: InputPin + Wait
{

    pub fn new(events_sender: DynamicSender<'static, RouterEvent>, ir: Infrared<PWM, InPin>) -> Self {
        Self {
            commands_receiver: INFRARED_COMMANDS_CHANNEL.dyn_receiver(),
            events_sender,
            ir,
        }
    }
   
    pub fn command_sender() -> DynamicSender<'static, InfraredCommand> {
        INFRARED_COMMANDS_CHANNEL.dyn_sender()
    }

    pub async fn run(&mut self) {
        loop {
            let comm = self.commands_receiver.receive().await;
            match comm {
                InfraredCommand::Play(sig) => {
                    self.ir.transmit(&sig).await;
                },
                InfraredCommand::Listen => {
                    info!("Start listening");
                    if let Some(signal) = self.ir.listen().await {
                        self.send_event(InfraredEvent::Signal(signal)).await;
                        info!("Signal event sended");
                    }                    

                },
            }  
        }
    }

    pub async fn send_event(&mut self, event: InfraredEvent) {
        self.events_sender
            .send(RouterEvent::InfraredEvent(event))
            .await;
    }
}
