use embedded_graphics::prelude::DrawTarget;

use heapless::Vec;

use crate::ViewContext;
use crate::devices::display::Display;
use crate::ui::Style;
use crate::ui::views::view::ViewAction;
use crate::ui::views::view::ViewType;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::DynamicSender;

use crate::services::router::{RouterEvent, RouterService};

static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, RouterEvent, 8> = Channel::new();

pub struct App<'a, D>
where
    D: Display,
{
    style: &'a Style,
    context: ViewContext<'a, D>,
    view_stack: Vec<ViewType, 4>,
}

impl<'a, D> App<'a, D>
where
    D: Display,
{
    pub fn new(style: &'a Style, display: &'a mut D) -> Self {
        Self {
            context: ViewContext::new(
                display,
                EVENT_CHANNEL.dyn_receiver(),
                RouterService::command_sender(),
            ),
            style,
            view_stack: Vec::new(),
        }
    }

    pub async fn start(
        &mut self,
        mut entry: ViewType,
    ) -> Result<(), <D::Target as DrawTarget>::Error> {
        loop {
            match entry.start(self.style, &mut self.context).await? {
                ViewAction::SwitchTo(v) => {
                    self.view_stack.push(entry);
                    entry = v;
                }

                ViewAction::Exit => {
                    if let Some(v) = self.view_stack.pop() {
                        entry = v;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn event_sender() -> DynamicSender<'static, RouterEvent> {
    EVENT_CHANNEL.dyn_sender()
}
