
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
    text::{Text, TextStyleBuilder},
    mono_font::ascii::FONT_6X10,
    draw_target::DrawTarget
};

use crate::devices::display::Display;

use super::list_view::ListView;
use super::dummy_view::DummyView;

pub trait Viewable<D: DrawTarget<Color = BinaryColor>> {
    fn run(&mut self, display: &mut impl Display<D>);
    fn title(&self) -> &str;
}

pub enum ViewType<'a> {
    ListView(ListView<'a>),
    DummyView(DummyView<'a>),
}

impl<'a, D> Viewable<D> for ViewType<'a>
where
    D: DrawTarget<Color = BinaryColor>
{
    fn run(&mut self, display: &mut impl Display<D>) {
        match self {
            ViewType::ListView(v) => v.run(display),
            ViewType::DummyView(v) => v.run(display),
        }
    }

    fn title(&self) -> &str {
        match self {
            ViewType::ListView(v) => <ListView<'_> as Viewable<D>>::title(v),
            ViewType::DummyView(v) => <DummyView<'_> as Viewable<D>>::title(v),
        }
    }
}

impl<'a> From<DummyView<'a>> for ViewType<'a> {
    fn from(v: DummyView<'a>) -> ViewType<'a> {
        ViewType::DummyView(v)
    }
}

impl<'a> From<ListView<'a>> for ViewType<'a> {
    fn from(v: ListView<'a>) -> ViewType<'a> {
        ViewType::ListView(v)
    }
}
