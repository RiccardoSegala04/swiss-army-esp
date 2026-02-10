
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, CornerRadii, CornerRadiiBuilder, Rectangle, RoundedRectangle},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    text::{Text, Alignment},
    draw_target::DrawTarget,
};

pub struct Button<'a> {
    name: &'a str,
    selected: bool,
    center: Point,
    size: Size, // store width and height
}

impl<'a> Button<'a> {

 pub fn new(name: &'a str, center: Point, size: Size) -> Self {
        Self {
            name,
            selected: false,
            center,
            size,
        }
    }
    
    pub fn selected_new(name: &'a str, center: Point, size: Size) -> Self {
        Self {
            name,
            selected: true,
            center,
            size,
        }
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn select(&mut self) {
        self.selected = true;
    }

    pub fn deselect(&mut self) {
        self.selected = false;
    }
}

impl<'a> Drawable for Button<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // Compute top-left corner from center
        let top_left = Point::new(
            self.center.x - self.size.width as i32 / 2,
            self.center.y - self.size.height as i32 / 2,
        );

        // Rectangle style
        let rect_style = if self.selected {
            PrimitiveStyle::with_fill(BinaryColor::On)
        } else {
            PrimitiveStyle::with_stroke(BinaryColor::On, 1)
        };


        let radii = CornerRadiiBuilder::new().all(Size::new(40, 40)).build();

        let rect = Rectangle::with_center(self.center, self.size);
        let rect = RoundedRectangle::with_equal_corners(rect, Size::new(2, 2))
            .into_styled(rect_style)
            .draw(target)?;

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(if self.selected { BinaryColor::Off } else { BinaryColor::On })
            .build();

        let text_center = Point::new(self.center.x, self.center.y+3);

        Text::with_alignment(self.name, text_center, text_style, Alignment::Center)
            .draw(target)?;

        Ok(())
    }
}
