

use defmt::info;

use embedded_hal::pwm::SetDutyCycle;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, DynamicReceiver, DynamicSender};

use crate::devices::ir::{Infrared, InfraredCommand, InfraredEvent};
use crate::services::router::RouterEvent;

pub static INFRARED_COMMANDS_CHANNEL: channel::Channel<
    CriticalSectionRawMutex,
    InfraredCommand,
    1,
> = channel::Channel::new();

pub struct InfraredService<PWM> {
    commands_receiver: DynamicReceiver<'static, InfraredCommand>,
    events_sender: DynamicSender<'static, RouterEvent>,
    ir: Infrared<PWM>,
}

impl<PWM> InfraredService<PWM>
where
    PWM: SetDutyCycle
{

    pub fn new(events_sender: DynamicSender<'static, RouterEvent>, ir: Infrared<PWM>) -> Self {
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
                _ => {}
            }  
        }
    }

    pub async fn send_event(&mut self, event: InfraredEvent) {
        self.events_sender
            .send(RouterEvent::InfraredEvent(event))
            .await;
    }
}
