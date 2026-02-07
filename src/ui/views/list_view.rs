

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    text::{Text, TextStyleBuilder},
    draw_target::DrawTarget
};

use super::view::Viewable;
use super::view::ViewType;

use crate::devices::display::Display;

pub struct ListView<'a> {
    title: &'a str,
    elements: &'a mut [ViewType<'a>],
    sel_idx: usize,
}

impl<'a> ListView<'a> {
    pub fn new(title: &'a str, elements: &'a mut [ViewType<'a>]) -> Self {
        Self {
            title,
            elements,
            sel_idx: 0,
        }
    }
}

impl<'a, D> Viewable<D> for ListView<'a>
where
    D: DrawTarget<Color = BinaryColor>
{
    fn run(&mut self, display: &mut impl Display<D>) {
        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let title = Text::new(self.title, Point::new(4, 9), style);

        display.draw(&title);

        let mut y: i32 = 20;
        for element in self.elements.iter_mut() {
            let elem_text = Text::new(<ViewType<'a> as Viewable<D>>::title(element), Point::new(4, y), style);
            display.draw(&elem_text);

            y+=11;
        }

        display.flush();
    }

    fn title(&self) -> &str {
        self.title
    }
}
