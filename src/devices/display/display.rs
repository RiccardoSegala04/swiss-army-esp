
use embedded_graphics::{prelude::*, pixelcolor::BinaryColor};

pub trait Display
{
    type Target: DrawTarget<Color = BinaryColor>;
    
    fn draw<T>(&mut self, item: &T) -> Result<(), <Self::Target as DrawTarget>::Error>
    where
        T: Drawable<Color = <Self::Target as DrawTarget>::Color>;

    fn draw_all<'a, T, I>(&mut self, items: I) -> Result<(), <Self::Target as DrawTarget>::Error>
    where
        T: Drawable<Color = <Self::Target as DrawTarget>::Color> + 'a,
        I: IntoIterator<Item = &'a T>,
    {
        for item in items {
            self.draw(item)?;
        }
        Ok(())
    }

    fn clear(&mut self, color: <Self::Target as DrawTarget>::Color) -> Result<(), <Self::Target as DrawTarget>::Error>;
    fn flush(&mut self);
}



