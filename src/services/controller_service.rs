use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel;

pub enum ControllerCommand {
    LedColor { red: u8, green: u8, blue: u8 },
}

pub enum ControllerEvent {
    ConfirmPressed,
}

pub struct ControllerService {
    commands_channel: channel::Channel<CriticalSectionRawMutex, ControllerCommand, 1>,
    events_channel: channel::Channel<CriticalSectionRawMutex, ControllerEvent, 1>,
}

impl ControllerService {
    pub fn new() -> Self {
        let commands_channel = channel::Channel::new();
        let events_channel = channel::Channel::new();

        Self {
            commands_channel,
            events_channel,
        }
    }

    pub fn commands_sender(&self) -> channel::DynamicSender<'_, ControllerCommand> {
        self.commands_channel.dyn_sender()
    }

    fn commands_receiver(&self) -> channel::DynamicReceiver<'_, ControllerCommand> {
        self.commands_channel.dyn_receiver()
    }

    fn events_sender(&self) -> channel::DynamicSender<'_, ControllerEvent> {
        self.events_channel.dyn_sender()
    }

    pub fn events_receiver(&self) -> channel::DynamicReceiver<'_, ControllerEvent> {
        self.events_channel.dyn_receiver()
    }
}
