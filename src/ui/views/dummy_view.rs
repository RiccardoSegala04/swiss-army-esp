use embedded_graphics::draw_target::DrawTarget;



use super::view::{Viewable, ViewContext};

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
    D: Display,
{
    async fn run(
        &mut self,
        context: &mut ViewContext<'_, D>
    ) -> Result<(), <D::Target as DrawTarget>::Error> {

        _ = context;

        Ok(())
    }

    fn title(&self) -> &str {
        self.title
    }
}
