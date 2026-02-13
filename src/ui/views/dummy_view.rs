use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::ascii::FONT_6X10,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Text, TextStyleBuilder},
};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, Receiver, Sender};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use crate::services::router::{RouterCommand, RouterEvent};

use super::view::ViewType;
use super::view::Viewable;

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
    D: DrawTarget<Color = BinaryColor>,
{
    async fn run(
        &mut self,
        display: &mut impl Display<D>,
        receiver: DynamicReceiver<'static, RouterEvent>,
    ) {
        _ = display;
        _ = receiver;
    }

    fn title(&self) -> &str {
        self.title
    }
}
