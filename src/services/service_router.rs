use crate::services::controller_service;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, Receiver, Sender};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use once_cell::sync::Lazy;

pub enum ServiceRouterCommand {
    ControllerCommand(controller_service::ControllerCommand),
}

pub enum ServiceRouterEvent {
    ControllerEvent(controller_service::ControllerEvent),
}

static COMMANDS_CHANNEL: Lazy<channel::Channel<CriticalSectionRawMutex, ServiceRouterCommand, 1>> =
    Lazy::new(|| channel::Channel::new());

pub struct CommandRouter<'a> {
    commands_channel: &'static channel::Channel<CriticalSectionRawMutex, ServiceRouterCommand, 1>,

    controller_commands: DynamicSender<'a, controller_service::ControllerCommand>,
}

impl<'a> CommandRouter<'a> {
    pub fn new(
        controller_commands: DynamicSender<'static, controller_service::ControllerCommand>,
    ) -> Self {
        Self {
            commands_channel: &COMMANDS_CHANNEL,
            controller_commands,
        }
    }

    pub fn commands_sender(&self) -> Sender<'_, CriticalSectionRawMutex, ServiceRouterCommand, 1> {
        self.commands_channel.sender()
    }

    fn commands_receiver(&self) -> Receiver<'_, CriticalSectionRawMutex, ServiceRouterCommand, 1> {
        self.commands_channel.receiver()
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
    pub const fn new() -> Self {
        Self
    }

    pub fn events_sender() -> DynamicSender<'static, ServiceRouterEvent> {
        EVENTS_CHANNEL.dyn_sender()
    }

    pub fn events_receiver() -> DynamicReceiver<'static, ServiceRouterEvent> {
        EVENTS_CHANNEL.dyn_receiver()
    }
}
