use defmt::info;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, DynamicReceiver, DynamicSender};

use crate::devices::controller;
use crate::devices::ir;

pub enum RouterCommand {
    ControllerCommand(controller::ControllerCommand),
    InfraredCommand(ir::InfraredCommand),
}

#[derive(Clone)]
pub enum RouterEvent {
    ControllerEvent(controller::ControllerEvent),
    InfraredEvent(ir::InfraredEvent),
}

pub static COMMANDS_CHANNEL: Channel<CriticalSectionRawMutex, RouterCommand, 1> = Channel::new();

static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, RouterEvent, 8> = Channel::new();

pub struct RouterService<'a> {
    command_channel: DynamicReceiver<'a, RouterCommand>,
    event_channel: DynamicReceiver<'a, RouterEvent>,

    ui_channel: DynamicSender<'a, RouterEvent>,
    cli_channel: DynamicSender<'a, RouterEvent>,

    controller_channel: DynamicSender<'a, controller::ControllerCommand>,
    infrared_channel: DynamicSender<'a, ir::InfraredCommand>,
}

impl<'a> RouterService<'a> {
    pub fn new(
        ui_channel: DynamicSender<'static, RouterEvent>,
        cli_channel: DynamicSender<'static, RouterEvent>,
        controller_channel: DynamicSender<'static, controller::ControllerCommand>,
        infrared_channel: DynamicSender<'static, ir::InfraredCommand>,
    ) -> Self {
        Self {
            command_channel: COMMANDS_CHANNEL.dyn_receiver(),
            event_channel: EVENT_CHANNEL.dyn_receiver(),
            ui_channel,
            cli_channel,
            controller_channel,
            infrared_channel,
        }
    }

    pub fn command_sender() -> DynamicSender<'static, RouterCommand> {
        COMMANDS_CHANNEL.dyn_sender()
    }

    pub fn event_sender() -> DynamicSender<'static, RouterEvent> {
        EVENT_CHANNEL.dyn_sender()
    }

    pub async fn route_command(&mut self, command: RouterCommand) {
        match command {
            RouterCommand::ControllerCommand(c) => {
                self.controller_channel.send(c).await;
            }
            RouterCommand::InfraredCommand(c) => {
                self.infrared_channel.send(c).await;
            }
        }
    }

    pub async fn route_event(&mut self, event: RouterEvent) {
        if !crate::services::cli::EVENT_CHANNEL.is_full() {
            self.cli_channel.send(event.clone()).await;
        }
        self.ui_channel.send(event.clone()).await;
    }

    pub async fn run(&mut self) -> ! {
        loop {
            let ev = self.event_channel.receive();
            let comm = self.command_channel.receive();

            match select(ev, comm).await {
                Either::First(ev) => self.route_event(ev).await,
                Either::Second(comm) => self.route_command(comm).await,
            }
        }
    }
}
