
use super::view::View;
use crate::Display;

use embedded_graphics::draw_target::DrawTarget;

pub struct MockView<'a> {
    title: &'a str,
}

impl<'a> MockView<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title
        }
    }
}

impl<'a, D> View<D> for MockView<'a>
where
    D: DrawTarget
{

    fn run(&mut self, display: &mut impl Display<D>) {
        // Do nothing
        _ = display; 
    }
    
    fn title(&self) -> &str {
        self.title
    }
}


