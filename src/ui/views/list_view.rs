use embedded_graphics::{draw_target::DrawTarget, prelude::*, text::Text};
use heapless::Vec;

use crate::services::router::RouterEvent;

use crate::devices::controller::ControllerEvent;
use crate::devices::display::Display;

use super::view::{ViewContext, ViewType, Viewable, ViewAction};

use crate::ui::elements::{TopBar};
use crate::ui::Style;

const TOP_BAR_HEIGHT: i32 = 16;
const DISPLAY_WIDTH: i32 = 128;
const FONT_BASELINE: i32 = 8;

const MARKER_SIZE: i32 = 5;
const MARKER_TEXT_GAP: i32 = 2;

const ITEM_HEIGHT: i32 = 10;
const LIST_START_Y: i32 = 22;

pub struct ListView<'a> {
    topbar: TopBar<'a>,
    elements: Vec<ViewType, 10>,
    sel_idx: usize,
    style: &'a Style,
}

impl<'a> ListView<'a> {
    pub fn new(style: &'a Style) -> Self {
        Self {
            topbar: TopBar::new(style, "Swiss Army Esp"),
            elements: Vec::from_array([
                ViewType::IrRxView,
                ViewType::DummyView("View 1"),
                ViewType::DummyView("View 2"),
                ViewType::DummyView("View 3"),
                ViewType::DummyView("View 4"),
                ViewType::DummyView("View 5"),
            ]),
            sel_idx: 0,
            style,
        }
    }

    pub fn draw<D>(&mut self, display: &mut D) -> Result<(), <D::Target as DrawTarget>::Error>
    where
        D: Display,
    {

        display.clear(self.style.color_bg)?;

        display.draw(&self.topbar)?;

        self.draw_list(display)?;

        display.flush();

        Ok(())
    }

    fn draw_list<D>(&self, display: &mut D) -> Result<(), <D::Target as DrawTarget>::Error>
    where
        D: Display,
    {

        let mut y = 25;

        let len = self.elements.len();
        let start = if self.sel_idx <= 1 {
            0
        } else if self.sel_idx < self.elements.len()-2 {
            self.sel_idx - 1
        } else {
            self.elements.len() - 4
        };

        let elements = &self.elements[start..start+4];

        for (idx, element) in elements.iter().enumerate() {

            draw_item_text(display, &element, idx == self.sel_idx-start, y, self.style)?;

            y += 12;
        }

        Ok(())
    }
}

impl<'a, D> Viewable<D> for ListView<'a>
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
                        ControllerEvent::NavNextPressed => self.sel_idx = (self.sel_idx + 1) % self.elements.len(),
                        ControllerEvent::NavPrevPressed => self.sel_idx = (self.sel_idx + self.elements.len()-1) % self.elements.len(),
                        ControllerEvent::ConfirmPressed => return Ok(ViewAction::SwitchTo(self.elements[self.sel_idx].clone())),
                        _ => {}
                    };
                }
                _ => {}
            };

            self.draw(context.display)?;
        }
    }

}

fn draw_item_text<D>(
    display: &mut D,
    element: &ViewType,
    selected: bool,
    y: i32,
    style: &Style,
) -> Result<(), <D::Target as DrawTarget>::Error>
where
    D: Display,
{

    let color = match selected {
        true => style.text_selected_big,
        false => style.text_big,
    };
   
    let text = Text::new(
        element.title(),
        Point::new(5, y),
        color,
    );

    display.draw(&text)
}
