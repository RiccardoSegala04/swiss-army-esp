use defmt::info;
use embedded_graphics::{draw_target::DrawTarget, prelude::*};
use heapless::Vec;

use crate::devices::cc1101::RadioCommand;
use crate::devices::controller::ControllerEvent;
use crate::devices::display::Display;
use crate::services::router::{RouterCommand, RouterEvent};

use super::view::{ViewAction, ViewContext, Viewable};

use crate::ui::Style;
use crate::ui::elements::{List, TopBar};

const TOP_BAR_HEIGHT: i32 = 16;
const DISPLAY_WIDTH: i32 = 128;
const FONT_BASELINE: i32 = 8;

const MARKER_SIZE: i32 = 5;
const MARKER_TEXT_GAP: i32 = 2;

const ITEM_HEIGHT: i32 = 10;
const LIST_START_Y: i32 = 22;

pub struct RadioSavedView<'a> {
    topbar: TopBar<'a>,
    list: List<'a>,
    style: &'a Style,
}

impl<'a> RadioSavedView<'a> {
    pub async fn new(style: &'a Style) -> Self {
        // Lock the signal history
        let history_lock = crate::devices::cc1101::SIGNAL_HISTORY.get().lock().await;

        // TODO: add number to saved signals
        let elem_str: Vec<&str, 10> = history_lock.iter().map(|_| "RADIO SIG").collect();

        for str in &elem_str {
            info!("{}", str);
        }

        Self {
            list: List::new(
                style,
                Point::new(4, 20),
                Size::new(128 - 8, 64 - 16 - 8),
                elem_str,
            ),
            topbar: TopBar::new(style, "RADIO SAVED"),
            style,
        }
    }

    pub fn draw<D>(&mut self, display: &mut D) -> Result<(), <D::Target as DrawTarget>::Error>
    where
        D: Display,
    {
        display.clear(self.style.color_bg)?;

        display.draw(&self.topbar)?;

        display.draw(&self.list)?;

        display.flush();

        Ok(())
    }
}

impl<'a, D> Viewable<D> for RadioSavedView<'a>
where
    D: Display,
{
    async fn run(
        &mut self,
        context: &mut ViewContext<'_, D>,
    ) -> Result<ViewAction, <D::Target as DrawTarget>::Error> {
        self.draw(context.display)?;

        loop {
            let ev = context.receiver.receive().await;

            match ev {
                RouterEvent::ControllerEvent(ev) => {
                    match ev {
                        ControllerEvent::NavNextPressed => self.list.select_next(),
                        ControllerEvent::NavPrevPressed => self.list.select_prev(),
                        ControllerEvent::ConfirmPressed => {
                            context
                                .sender
                                .send(RouterCommand::RadioCommand(RadioCommand::Play(
                                    crate::devices::cc1101::SIGNAL_HISTORY
                                        .get()
                                        .lock()
                                        .await
                                        .get(self.list.selected_index())
                                        .unwrap()
                                        .clone(),
                                )))
                                .await
                        }
                        ControllerEvent::BackPressed => return Ok(ViewAction::Exit),
                        _ => {}
                    };
                }
                _ => {}
            };

            self.draw(context.display)?;
        }
    }
}
