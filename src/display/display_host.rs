use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::draw_target::DrawTarget;

use embedded_graphics_simulator::{BinaryColorTheme, SimulatorDisplay, Window, OutputSettingsBuilder};

use super::Display;

pub struct DisplayHost {
    target: SimulatorDisplay<BinaryColor>, 
}

impl DisplayHost {
    pub fn new() -> Self {
        let mut target = SimulatorDisplay::new(Size::new(128, 64));

        Self {
            target
        }
    }

    pub fn show(&mut self) {

        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::OledBlue)
            .build();

        Window::new("PipuZero", &output_settings).show_static(&self.target);
    }
}

impl Display<SimulatorDisplay<BinaryColor>> for DisplayHost {
    
    fn draw<T>(&mut self, item: &T) -> Result<(), <SimulatorDisplay<BinaryColor> as DrawTarget>::Error>
    where
        T: Drawable<Color = BinaryColor>
    {
        item.draw(&mut self.target)?;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), <SimulatorDisplay<BinaryColor> as DrawTarget>::Error> {
        self.target.clear(BinaryColor::Off)
    }

    fn flush(&mut self) {}
}
