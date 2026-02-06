use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::Drawable;

pub struct Display<D>
where
    D: DrawTarget,
{
    target: D,
}

impl<D> Display<D>
where
    D: DrawTarget,
{
    pub fn new(target: D) -> Self {
        Self { target }
    }

    pub fn draw<T>(&mut self, item: &T) -> Result<(), D::Error>
    where
        T: Drawable<Color = D::Color>,
    {
        item.draw(&mut self.target)?;
        Ok(())
    }

    pub fn draw_all<'a, T, I>(&mut self, items: I) -> Result<(), D::Error>
    where
        T: Drawable<Color = D::Color> + 'a,
        I: IntoIterator<Item = &'a T>,
    {
        for item in items {
            item.draw(&mut self.target)?;
        }
        Ok(())
    }

    pub fn clear<C>(&mut self, color: C) -> Result<(), D::Error>
    where
        C: Into<D::Color>,
    {
        self.target.clear(color.into())
    }
}
