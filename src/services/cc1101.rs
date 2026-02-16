use embedded_hal::digital::{InputPin, OutputPin};

use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, DynamicReceiver, DynamicSender};

use crate::devices::cc1101::{Cc1101, RadioCommand, RadioEvent};
use crate::services::router::RouterEvent;

pub static INFRARED_COMMANDS_CHANNEL: channel::Channel<CriticalSectionRawMutex, RadioCommand, 1> =
    channel::Channel::new();

pub struct RadioService<SPId, OutPin, InPin> {
    commands_receiver: DynamicReceiver<'static, RadioCommand>,
    events_sender: DynamicSender<'static, RouterEvent>,
    cc1101: Cc1101<SPId, InPin, OutPin>,
}

impl<SPId, OutPin, InPin> RadioService<SPId, OutPin, InPin>
where
    InPin: InputPin + Wait,
    OutPin: OutputPin,
    SPId: SpiDevice,
{
    pub fn new(
        events_sender: DynamicSender<'static, RouterEvent>,
        cc1101: Cc1101<SPId, InPin, OutPin>,
    ) -> Self {
        Self {
            commands_receiver: INFRARED_COMMANDS_CHANNEL.dyn_receiver(),
            events_sender,
            cc1101,
        }
    }

    pub fn command_sender() -> DynamicSender<'static, RadioCommand> {
        INFRARED_COMMANDS_CHANNEL.dyn_sender()
    }

    pub async fn run(&mut self) {
        loop {
            let comm = self.commands_receiver.receive().await;
            match comm {
                RadioCommand::Play(sig) => {
                    match self.cc1101.transmit_signal(&sig).await {
                        Ok(_) => {
                            self.send_event(RadioEvent::SignalPlayed).await;
                        }
                        Err(_) => {
                            self.send_event(RadioEvent::Error).await;
                        }
                    };
                }
                RadioCommand::Listen => match self.cc1101.listen_signal().await {
                    Ok(ev) => {
                        self.send_event(ev).await;
                    }
                    Err(_) => {
                        self.send_event(RadioEvent::Error).await;
                    }
                },
            }
        }
    }

    pub async fn send_event(&mut self, event: RadioEvent) {
        self.events_sender
            .send(RouterEvent::RadioEvent(event))
            .await;
    }
}
