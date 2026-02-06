use crate::services::controller_service;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel;

pub enum ServiceRouterCommand {
    ControllerCommand(controller_service::ControllerCommand),
}

pub enum ServiceRouterEvent {
    ControllerEvent(controller_service::ControllerEvent),
}

static COMMANDS_CHANNEL: channel::Channel<CriticalSectionRawMutex, ServiceRouterCommand, 1> =
    channel::Channel::new();

pub struct CommandRouter {
    commands_channel: &'static channel::Channel<CriticalSectionRawMutex, ServiceRouterCommand, 1>,
    controller_commands: channel::DynamicSender<'static, controller_service::ControllerCommand>,
}

impl CommandRouter {
    pub fn new(controller: &'static controller_service::ControllerService) -> Self {
        let controller_commands = controller.commands_sender();

        Self {
            commands_channel: &COMMANDS_CHANNEL,
            controller_commands: controller_commands,
        }
    }

    pub fn commands_sender(&self) -> channel::DynamicSender<'_, ServiceRouterCommand> {
        self.commands_channel.dyn_sender()
    }

    fn commands_receiver(&self) -> channel::DynamicReceiver<'_, ServiceRouterCommand> {
        self.commands_channel.dyn_receiver()
    }

    pub async fn run(&mut self) -> ! {
        let commands = self.commands_receiver();
        loop {
            let command = commands.receive().await;
            match command {
                ServiceRouterCommand::ControllerCommand(controller_command) => {
                    self.controller_commands.send(controller_command).await;
                }
            }
        }
    }
}

static EVENTS_CHANNEL: channel::Channel<CriticalSectionRawMutex, ServiceRouterEvent, 1> =
    channel::Channel::new();

pub struct EventRouter;

impl EventRouter {
    pub fn new() -> Self {
        Self
    }

    pub fn events_sender(&self) -> channel::DynamicSender<'static, ServiceRouterEvent> {
        EVENTS_CHANNEL.dyn_sender()
    }

    pub fn events_receiver(&self) -> channel::DynamicReceiver<'static, ServiceRouterEvent> {
        EVENTS_CHANNEL.dyn_receiver()
    }
}
