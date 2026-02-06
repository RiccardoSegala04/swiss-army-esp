use crate::services::service_router::{self, ServiceRouterEvent};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel;

pub enum ControllerCommand {
    LedColor { red: u8, green: u8, blue: u8 },
}

pub enum ControllerEvent {
    ConfirmPressed,
}

static CONTROLLER_COMMANDS_CHANNEL: channel::Channel<
    CriticalSectionRawMutex,
    ControllerCommand,
    1,
> = channel::Channel::new();

pub struct ControllerService {
    commands_channel: &'static channel::Channel<CriticalSectionRawMutex, ControllerCommand, 1>,
    events_sender: channel::DynamicSender<'static, service_router::ServiceRouterEvent>,
}

impl ControllerService {
    pub fn new(
        events_sender: channel::DynamicSender<'static, service_router::ServiceRouterEvent>,
    ) -> Self {
        Self {
            commands_channel: &CONTROLLER_COMMANDS_CHANNEL,
            events_sender,
        }
    }

    pub fn commands_sender(&self) -> channel::DynamicSender<'_, ControllerCommand> {
        self.commands_channel.dyn_sender()
    }

    fn commands_receiver(&self) -> channel::DynamicReceiver<'_, ControllerCommand> {
        self.commands_channel.dyn_receiver()
    }

    pub async fn run(self) -> ! {
        let commands = self.commands_receiver();
        loop {
            let command = commands.receive().await;
            match command {
                ControllerCommand::LedColor { red, green, blue } => {
                    // Handle LED color change
                    // For example, set the LED color using the device driver
                }
            }
        }
    }

    pub async fn send_event(&mut self, event: ControllerEvent) {
        self.events_sender
            .send(ServiceRouterEvent::ControllerEvent(event))
            .await;
    }
}
