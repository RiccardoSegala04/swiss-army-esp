use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel;
use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use crate::devices::ir;
use crate::devices::{cc1101, controller};

pub enum RouterCommand {
    ControllerCommand(controller::ControllerCommand),
    InfraredCommand(ir::InfraredCommand),
}

pub enum RouterEvent {
    ControllerEvent(controller::ControllerEvent),
    InfraredEvent(ir::InfraredEvent),
}

pub static COMMANDS_CHANNEL: channel::Channel<CriticalSectionRawMutex, RouterCommand, 1> =
    channel::Channel::new();

pub struct RouterService<'a> {
    router_channel: DynamicReceiver<'a, RouterCommand>,
    controller_channel: DynamicSender<'a, controller::ControllerCommand>,
    infrared_channel: DynamicSender<'a, ir::InfraredCommand>,
}

impl<'a> RouterService<'a> {
    pub fn new(
        controller_channel: DynamicSender<'static, controller::ControllerCommand>,
        infrared_channel: DynamicSender<'static, ir::InfraredCommand>,
    ) -> Self {
        Self {
            router_channel: COMMANDS_CHANNEL.dyn_receiver(),
            controller_channel,
            infrared_channel,
        }
    }

    pub fn command_sender() -> DynamicSender<'static, RouterCommand> {
        COMMANDS_CHANNEL.dyn_sender()
    }

    pub async fn run(&mut self) -> ! {
        loop {
            let comm = self.router_channel.receive().await;
            match comm {
                RouterCommand::ControllerCommand(c) => {
                    self.controller_channel.send(c).await;
                }
                RouterCommand::InfraredCommand(c) => {
                    self.infrared_channel.send(c).await;
                }
            }
        }
    }
}
