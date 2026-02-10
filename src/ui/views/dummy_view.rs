
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
    text::{Text, TextStyleBuilder},
    mono_font::ascii::FONT_6X10,
    draw_target::DrawTarget
};

use super::view::Viewable;
use super::view::ViewType;

use crate::devices::display::Display;

pub struct DummyView<'a> {
    title: &'a str,
}

impl<'a> DummyView<'a> {
    pub fn new(title: &'a str) -> Self {
        Self { title }
    }
}

impl<'a, D> Viewable<D> for DummyView<'a>
where
    D: DrawTarget<Color = BinaryColor>
{
    fn run(&mut self, display: &mut impl Display<D>) {
        _ = display;
    }

    fn title(&self) -> &str {
        self.title
    }
}
