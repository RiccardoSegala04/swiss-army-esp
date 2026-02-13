use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, Receiver, Sender};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use crate::devices::controller;

pub enum RouterCommand {
    ControllerCommand(controller::ControllerCommand),
}

pub enum RouterEvent {
    ControllerEvent(controller::ControllerEvent),
}

pub static COMMANDS_CHANNEL: channel::Channel<CriticalSectionRawMutex, RouterCommand, 1> =
    channel::Channel::new();

pub struct RouterService<'a> {
    router_channel: DynamicReceiver<'a, RouterCommand>,
    controller_channel: DynamicSender<'a, controller::ControllerCommand>,
}

impl<'a> RouterService<'a> {
    pub fn new(controller_channel: DynamicSender<'static, controller::ControllerCommand>) -> Self {
        Self {
            router_channel: COMMANDS_CHANNEL.receiver().into(),
            controller_channel,
        }
    }

    pub fn command_sender(&self) -> Sender<'_, CriticalSectionRawMutex, RouterCommand, 1> {
        COMMANDS_CHANNEL.sender()
    }

    pub async fn run(&mut self) -> ! {
        loop {
            let comm = self.router_channel.receive().await;
            match comm {
                RouterCommand::ControllerCommand(c) => {
                    self.controller_channel.send(c).await;
                }
            }
        }
    }
}

// static EVENTS_CHANNEL: channel::Channel<CriticalSectionRawMutex, ServiceRouterEvent, 1> =
//     channel::Channel::new();

// pub struct EventRouter;

// impl EventRouter {
//     pub const fn new() -> Self {
//         Self
//     }

//     pub fn events_sender() -> DynamicSender<'static, ServiceRouterEvent> {
//         EVENTS_CHANNEL.dyn_sender()
//     }

//     pub fn events_receiver() -> DynamicReceiver<'static, ServiceRouterEvent> {
//         EVENTS_CHANNEL.dyn_receiver()
//     }
// }
//
