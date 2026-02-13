use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::BinaryColor,
};

use embassy_sync::channel::{DynamicReceiver};

use crate::services::router::RouterEvent;

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
    ) -> Result<(), D::Error> {
        _ = display;
        _ = receiver;

        Ok(())
    }

    fn title(&self) -> &str {
        self.title
    }
}
