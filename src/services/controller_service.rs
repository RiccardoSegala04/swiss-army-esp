use crate::devices::controller;
use crate::services::service_router::{self, ServiceRouterEvent};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, Receiver, Sender};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use embedded_hal::digital::InputPin;
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;
//use esp_hal::gpio::OutputPin;

pub enum ControllerCommand {
    LedColor { red: u8, green: u8, blue: u8 },
}

pub enum ControllerEvent {
    ConfirmPressed,
}

pub static CONTROLLER_COMMANDS_CHANNEL: channel::Channel<
    CriticalSectionRawMutex,
    ControllerCommand,
    1,
> = channel::Channel::new();

pub struct ControllerService<PWMpin, InPin> {
    //commands_channel: &'static channel::Channel<CriticalSectionRawMutex, ControllerCommand, 1>,
    events_sender: DynamicSender<'static, service_router::ServiceRouterEvent>,
    controller_driver: controller::Controller<PWMpin, InPin>,
}

impl<PWMpin, InPin> ControllerService<PWMpin, InPin>
where
    PWMpin: OutputPin, //SetDutyCycle,
    InPin: InputPin,
{
    pub fn new(
        events_sender: DynamicSender<'static, service_router::ServiceRouterEvent>,
        controller_driver: controller::Controller<PWMpin, InPin>,
    ) -> Self {
        Self {
            events_sender,
            controller_driver,
        }
    }

    pub fn commands_sender() -> DynamicSender<'static, ControllerCommand> {
        CONTROLLER_COMMANDS_CHANNEL.dyn_sender()
    }

    fn commands_receiver() -> DynamicReceiver<'static, ControllerCommand> {
        CONTROLLER_COMMANDS_CHANNEL.dyn_receiver()
    }

    pub async fn run(&mut self) -> ! {
        let commands = ControllerService::<PWMpin, InPin>::commands_receiver();
        loop {
            let command = commands.receive().await;
            match command {
                ControllerCommand::LedColor { red, green, blue } => {
                    // Handle LED color change
                    // For example, set the LED color using the device driver
                    //self.controller_driver.set_led(red, green, blue).ok();
                    self.send_event(ControllerEvent::ConfirmPressed).await;
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
