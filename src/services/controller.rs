use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, DynamicReceiver, DynamicSender};

use crate::devices::controller::{Controller, ControllerCommand, ControllerEvent};
use crate::services::router::RouterEvent;

pub static CONTROLLER_COMMANDS_CHANNEL: channel::Channel<
    CriticalSectionRawMutex,
    ControllerCommand,
    1,
> = channel::Channel::new();

pub struct ControllerService<InPin> {
    commands_receiver: DynamicReceiver<'static, ControllerCommand>,
    events_sender: DynamicSender<'static, RouterEvent>,
    controller_driver: Controller<InPin>,
}

impl<InPin> ControllerService<InPin>
where
    InPin: InputPin + Wait,
{
    pub fn new(
        events_sender: DynamicSender<'static, RouterEvent>,
        controller_driver: Controller<InPin>,
    ) -> Self {
        Self {
            commands_receiver: CONTROLLER_COMMANDS_CHANNEL.dyn_receiver(),
            events_sender,
            controller_driver,
        }
    }

    pub fn command_sender() -> DynamicSender<'static, ControllerCommand> {
        CONTROLLER_COMMANDS_CHANNEL.dyn_sender()
    }

    pub async fn run(&mut self) -> ! {
        loop {
            let ev = self.controller_driver.poll_events().await;
            self.send_event(ev).await;
        }
    }

    pub async fn send_event(&mut self, event: ControllerEvent) {
        self.events_sender
            .send(RouterEvent::ControllerEvent(event))
            .await;
    }
}
