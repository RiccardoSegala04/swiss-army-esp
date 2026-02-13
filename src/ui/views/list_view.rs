use embedded_graphics::{
    draw_target::DrawTarget, prelude::*, primitives::Rectangle, text::Text,
};


use crate::services::router::RouterEvent;

use crate::devices::controller::ControllerEvent;
use crate::devices::display::Display;

use super::view::{ViewType, Viewable, ViewContext};

use crate::ui::Style;

const TOP_BAR_HEIGHT: i32 = 16;
const DISPLAY_WIDTH: i32 = 128;
const FONT_BASELINE: i32 = 8;

const MARKER_SIZE: i32 = 5;
const MARKER_TEXT_GAP: i32 = 2;

const ITEM_HEIGHT: i32 = 10;
const LIST_START_Y: i32 = 22;

pub struct ListView<'a> {
    title: &'a str,
    elements: &'a mut [ViewType<'a>],
    sel_idx: usize,
    vpad: u8,
    hpad: u8,
    style: &'a Style,
}

impl<'a> ListView<'a> {
    pub fn new(style: &'a Style, title: &'a str, elements: &'a mut [ViewType<'a>]) -> Self {
        Self {
            title,
            elements,
            sel_idx: 0,
            vpad: 2,
            hpad: 2,
            style,
        }
    }

    pub fn draw<D>(&mut self, display: &mut D) -> Result<(), <D::Target as DrawTarget>::Error>
    where
        D: Display
    {
        display.clear(self.style.color_bg)?;

        self.draw_top_bar(display)?;
        self.draw_list(display)?;

        display.flush();

        Ok(())
    }

    fn draw_top_bar<D>(&self, display: &mut D) -> Result<(), <D::Target as DrawTarget>::Error>
    where
        D: Display
    {
        let bar = Rectangle::new(
            Point::new(0, 0),
            Size::new(DISPLAY_WIDTH as u32, TOP_BAR_HEIGHT as u32),
        )
        .into_styled(self.style.bar);

        display.draw(&bar)?;

        let title = Text::new(
            self.title,
            Point::new(self.hpad as i32, self.vpad as i32 + FONT_BASELINE + 1),
            self.style.text_bar_big,
        );

        display.draw(&title)?;

        Ok(())
    }

    fn draw_list<D>(&mut self, display: &mut D) -> Result<(), <D::Target as DrawTarget>::Error>
    where
        D: Display
    {
        let sel_idx = self.sel_idx;
        let hpad = self.hpad;
        let vpad = self.vpad;

        let mut y = LIST_START_Y + vpad as i32;

        for (idx, element) in self.elements.iter_mut().enumerate() {
            draw_marker(display, idx, sel_idx, hpad, y, self.style)?;
            draw_item_text(display, element, hpad, y, self.style)?;

            y += ITEM_HEIGHT + vpad as i32;
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
        context: &mut ViewContext<'_, D>
    ) -> Result<(), <D::Target as DrawTarget>::Error> {

        self.draw(context.display)?;

        loop {
            let ev = context.receiver.receive().await;

            match ev {
                RouterEvent::ControllerEvent(ev) => {
                    match ev {
                        ControllerEvent::NavNextPressed => self.sel_idx = (self.sel_idx + 1) % 4,
                        ControllerEvent::NavPrevPressed => self.sel_idx = (self.sel_idx + 3) % 4,
                        _ => {}
                    };
                },
                _ => {}
            };

            self.draw(context.display)?;
        }
    }

    fn title(&self) -> &str {
        self.title
    }
}

fn draw_marker<D>(
    display: &mut D,
    idx: usize,
    sel_idx: usize,
    hpad: u8,
    y: i32,
    style: &Style,
) -> Result<(), <D::Target as DrawTarget>::Error>
where
    D: Display
{
    let base = Rectangle::new(
        Point::new(hpad as i32, y - MARKER_SIZE),
        Size::new(MARKER_SIZE as u32, MARKER_SIZE as u32),
    );

    if idx == sel_idx {
        let marker = base.into_styled(style.selected);
        display.draw(&marker)?;
    };

    Ok(())

}

fn draw_item_text<D>(
    display: &mut D,
    element: &ViewType<'_>,
    hpad: u8,
    y: i32,
    style: &Style,
) -> Result<(), <D::Target as DrawTarget>::Error>
where
    D: Display,
{
    let text = Text::new(
        <ViewType<'_> as Viewable<D>>::title(element),
        Point::new(hpad as i32 + MARKER_SIZE + MARKER_TEXT_GAP, y),
        style.text_big,
    );

    display.draw(&text)
}
