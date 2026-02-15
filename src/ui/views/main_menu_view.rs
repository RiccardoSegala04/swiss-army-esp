use embedded_graphics::{draw_target::DrawTarget, prelude::*, text::Text};
use heapless::Vec;

use crate::services::router::RouterEvent;

use crate::devices::controller::ControllerEvent;
use crate::devices::display::Display;

use super::view::{ViewContext, ViewType, Viewable, ViewAction};

use crate::ui::elements::{TopBar, List};
use crate::ui::Style;

const TOP_BAR_HEIGHT: i32 = 16;
const DISPLAY_WIDTH: i32 = 128;
const FONT_BASELINE: i32 = 8;

const MARKER_SIZE: i32 = 5;
const MARKER_TEXT_GAP: i32 = 2;

const ITEM_HEIGHT: i32 = 10;
const LIST_START_Y: i32 = 22;

pub struct MainMenuView<'a> {
    topbar: TopBar<'a>,
    list: List<'a>,
    elements: Vec<ViewType, 10>,
    style: &'a Style,
}

impl<'a> MainMenuView<'a> {

    pub fn new(style: &'a Style) -> Self {

        let elem = Vec::from_array([
            ViewType::IrRxView,
            ViewType::IrSavedView,
            ViewType::DummyView("View 1"),
            ViewType::DummyView("View 2"),
            ViewType::DummyView("View 3"),
            ViewType::DummyView("View 4"),
        ]);

        let elem_str: Vec<&str, 10> = elem.iter().map(|v| v.title()).collect();

        Self {
            list: List::new(style, Point::new(4, 20), Size::new(128-8, 64-16-8), elem_str),
            topbar: TopBar::new(style, "Swiss Army Esp"),
            elements: elem,
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

impl<'a, D> Viewable<D> for MainMenuView<'a>
where
    D: Display,
{
    async fn run(
        &mut self,
        context: &mut ViewContext<'_, D>,
    ) -> Result<ViewAction, <D::Target as DrawTarget>::Error> {


        loop {
            self.draw(context.display)?;

            let ev = context.receiver.receive().await;

            match ev {
                RouterEvent::ControllerEvent(ev) => {
                    match ev {
                        ControllerEvent::NavNextPressed => self.list.select_next(),
                        ControllerEvent::NavPrevPressed => self.list.select_prev(),
                        ControllerEvent::ConfirmPressed => return Ok(ViewAction::SwitchTo(self.elements[self.list.selected_index()].clone())),
                        _ => {}
                    };
                }
                _ => {}
            };

            self.draw(context.display)?;
        }
    }

}

