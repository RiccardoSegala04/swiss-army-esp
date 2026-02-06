use embedded_graphics::prelude::*;

pub trait Display<D>
where
    D: DrawTarget,
{
    fn draw<T>(&mut self, item: &T) -> Result<(), D::Error>
    where
        T: Drawable<Color = D::Color>;

    fn draw_all<'a, T, I>(&mut self, items: I) -> Result<(), D::Error>
    where
        T: Drawable<Color = D::Color> + 'a,
        I: IntoIterator<Item = &'a T>,
    {
        for item in items {
            self.draw(item)?;
        }
        Ok(())
    }

    fn clear(&mut self) -> Result<(), D::Error>;
    fn flush(&mut self);
}

