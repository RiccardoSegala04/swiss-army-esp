use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::mock_display::MockDisplay;

use super::Display;

pub struct DisplayHost {
    target: MockDisplay<BinaryColor>, 
}

impl Display<MockDisplay<BinaryColor>> for DisplayHost {
    
    fn draw<T>(&mut self, item: &T) -> Result<(), D::Error>
    where
        T: Drawable<Color = D::Color>
    {
        item.draw(&mut target)?
        Ok(())
    }

    fn clear(&mut self) -> Result<(), D::Error> {
        self.target.clear(BinaryColor::Off)
    }

    fn flush(&mut self) {}
}
