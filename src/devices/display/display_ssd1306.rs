use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use ssd1306::Ssd1306;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;

use super::Display;

pub struct DisplaySsd1306<DI, SIZE>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    ssd: Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
}

impl<DI, SIZE> DisplaySsd1306<DI, SIZE>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    pub fn new(ssd: Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>) -> Self {
        Self { ssd }
    }
}

impl<DI, SIZE> Display<Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>> for DisplaySsd1306<DI, SIZE>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    fn draw<T>(
        &mut self,
        item: &T,
    ) -> Result<(), <Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>> as DrawTarget>::Error>
    where
        T: Drawable<Color = <Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>> as DrawTarget>::Color>,
    {
        item.draw(&mut self.ssd)?;
        Ok(())
    }

    fn clear(
        &mut self, color: BinaryColor
    ) -> Result<(), <Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>> as DrawTarget>::Error> {
        self.ssd.clear(color)
    }

    fn flush(&mut self) {
        let _ = self.ssd.flush();
    }
}
