
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Line, RoundedRectangle, Rectangle},
    draw_target::DrawTarget,
    Drawable,
};

use crate::Display;
use crate::devices::ir::IrSignal;

pub struct IrSignalViewer<'a> {
    signal: Option<IrSignal<'a>>,
    selected: bool,
    center: Point,
    size: Size, // bounding box for highlighting
}

impl<'a> IrSignalViewer<'a> {
    pub fn new(signal: Option<IrSignal<'a>>, center: Point, size: Size) -> Self {
        Self {signal, selected: false, center, size}
    }

    pub fn select(&mut self) {
        self.selected = true; 
    }

    pub fn deselect(&mut self) {
        self.selected = false;
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn set_signal(&mut self, signal: IrSignal<'a>) {
        self.signal = Some(signal);
    }
}

impl<'a> Drawable for IrSignalViewer<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // Draw selection highlight behind the waveform
        if self.selected() {
            let rect = Rectangle::with_center(self.center, self.size);
            let rect = RoundedRectangle::with_equal_corners(rect, Size::new(2, 2))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(target)?;
        }

        if let Some(signal) = &self.signal {
            let ir_size = Size::new(self.size.width-6, self.size.height-6);
            draw_ir_signal(target, self.center.clone(), ir_size, signal)?;
        }

        Ok(())
    }
}

fn draw_ir_signal<D>(target: &mut D, center: Point, size: Size, signal: &IrSignal) -> Result<(), <D as DrawTarget>::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{

    if let Some(min) = signal.timings.iter().copied().min() {
        let mut high = true;
        let mut cursor = center.x - size.width as i32 / 2 + 1;
        let len = signal.timings.len();
        let pulse_base = center.y + size.height as i32 / 2;

        for pulse in signal.timings {

            let scaled = ((*pulse as f32 / min as f32) * 2.0) as i32;

            let vert = Line::new(Point::new(cursor, pulse_base), Point::new(cursor, pulse_base - size.height as i32 + 1))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1));

            let mut destx = cursor + scaled;

            if destx > center.x + size.width as i32 / 2 - 1 {
                destx = center.x + size.width as i32 / 2 - 1;
            }

            let horiz = if !high {
                Line::new(Point::new(cursor, pulse_base), Point::new(destx, pulse_base))
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            } else {
                Line::new(Point::new(cursor, pulse_base- size.height as i32 + 1), Point::new(destx, pulse_base-size.height as i32 + 1))
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            };


            vert.draw(target)?;
            horiz.draw(target)?;

            if (destx >= center.x + size.width as i32 / 2 - 1) {
                break;
            }

            cursor = destx;
            high = !high;

        }

    }
    Ok(())
}
