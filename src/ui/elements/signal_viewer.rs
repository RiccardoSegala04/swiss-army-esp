use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle, RoundedRectangle},
};

use crate::ui::Style;

pub trait Signal {
    fn timings(&self) -> &[u16];
}

pub struct SignalViewer<'a, S>
where
    S: Signal,
{
    signal: Option<S>,
    center: Point,
    size: Size, // bounding box for highlighting
    style: &'a Style,
}

impl<'a, S> SignalViewer<'a, S>
where
    S: Signal,
{
    pub fn new(style: &'a Style, signal: Option<S>, center: Point, size: Size) -> Self {
        Self {
            signal,
            center,
            size,
            style,
        }
    }

    pub fn set_signal(&mut self, signal: Option<S>) {
        self.signal = signal;
    }

    fn draw_ir_signal<D>(
        &self,
        target: &mut D,
        center: Point,
        size: Size,
    ) -> Result<(), <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        if let Some(signal) = &self.signal {
            if let Some(min) = signal.timings().iter().copied().min() {
                let mut high = true;
                let mut cursor = center.x - size.width as i32 / 2 + 1;
                let len = signal.timings().len();
                let pulse_base = center.y + size.height as i32 / 2;

                for pulse in signal.timings() {
                    let scaled = ((*pulse as f32 / min as f32) * 2.0) as i32;

                    let vert = Line::new(
                        Point::new(cursor, pulse_base),
                        Point::new(cursor, pulse_base - size.height as i32 + 1),
                    )
                    .into_styled(PrimitiveStyle::with_stroke(self.style.color_fg, 1));

                    let mut destx = cursor + scaled;

                    if destx > center.x + size.width as i32 / 2 - 1 {
                        destx = center.x + size.width as i32 / 2 - 1;
                    }

                    let horiz = if !high {
                        Line::new(
                            Point::new(cursor, pulse_base),
                            Point::new(destx, pulse_base),
                        )
                        .into_styled(PrimitiveStyle::with_stroke(self.style.color_fg, 1))
                    } else {
                        Line::new(
                            Point::new(cursor, pulse_base - size.height as i32 + 1),
                            Point::new(destx, pulse_base - size.height as i32 + 1),
                        )
                        .into_styled(PrimitiveStyle::with_stroke(self.style.color_fg, 1))
                    };

                    vert.draw(target)?;
                    horiz.draw(target)?;

                    if destx >= center.x + size.width as i32 / 2 - 1 {
                        break;
                    }

                    cursor = destx;
                    high = !high;
                }
            }
        }
        Ok(())
    }
}

impl<'a, S> Drawable for SignalViewer<'a, S>
where
    S: Signal,
{
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let rect = Rectangle::with_center(self.center, self.size);
        let rect = RoundedRectangle::with_equal_corners(rect, Size::new(2, 2))
            .into_styled(PrimitiveStyle::with_stroke(self.style.color_fg, 1))
            .draw(target)?;

        let ir_size = Size::new(self.size.width - 6, self.size.height - 6);
        self.draw_ir_signal(target, self.center.clone(), ir_size)?;

        Ok(())
    }
}
