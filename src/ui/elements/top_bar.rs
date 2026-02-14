use embedded_graphics::{
    draw_target::DrawTarget,
    prelude::*,
    primitives::{Rectangle, Circle},
    pixelcolor::BinaryColor,
    text::Text,
};

use crate::ui::Style;

pub struct TopBar<'a> {
    style: &'a Style,
    title: &'a str,
    record: bool
}

impl<'a> TopBar<'a> {  
    pub fn new(style: &'a Style, title: &'a str) -> Self {
        Self {style, title, record: false}
    }

    pub fn start_record(&mut self) {
        self.record = true;
    }

    pub fn stop_record(&mut self) {
        self.record = false;
    }

    pub fn title(&self) -> &'a str {
        self.title
    }

}

impl<'a> Drawable for TopBar<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {

        Rectangle::new(Point::new(0, 0), Size::new(128, 16))
            .into_styled(self.style.bar_bg)
            .draw(target)?;

        if self.record {
            Circle::new(Point::new(128-5-6, 8-3), 6)
                .into_styled(self.style.bar_fg)
                .draw(target)?;
        }


        Text::new(self.title, Point::new(5, 11), self.style.text_bar_big)
            .draw(target)?;

        Ok(())

    }
}

