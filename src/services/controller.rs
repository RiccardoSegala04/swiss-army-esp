use embedded_hal::digital::InputPin;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::digital::Wait;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, Receiver, Sender};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use crate::devices::controller::{Controller, ControllerCommand, ControllerEvent};
use crate::services::router::{self, RouterEvent};

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

    pub fn commands_sender() -> DynamicSender<'static, ControllerCommand> {
        CONTROLLER_COMMANDS_CHANNEL.dyn_sender()
    }

    pub async fn run(&mut self) -> ! {
        // let commands = ControllerService::<PWMpin, InPin>::commands_receiver();
        // loop {
        //     let command = commands.receive().await;
        //     match command {
        //         ControllerCommand::LedColor { red, green, blue } => {
        //             // Handle LED color change
        //             // For example, set the LED color using the device driver
        //             //self.controller_driver.set_led(red, green, blue).ok();
        //             self.send_event(ControllerEvent::ConfirmPressed).await;
        //         }
        //     }
        // }

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
