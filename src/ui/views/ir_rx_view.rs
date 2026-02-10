
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle, Line},
    text::{Text, TextStyleBuilder, TextStyle},
    mono_font::ascii::FONT_6X10,
    mono_font::ascii::FONT_4X6,
    mono_font::MonoTextStyleBuilder,
    draw_target::DrawTarget
};

use super::view::Viewable;
use super::view::ViewType;

use crate::ui::elements::Button;
use crate::ui::elements::IrSignalViewer;

use crate::devices::display::Display;
use crate::devices::ir::IrSignal;


static SAMPLE_TIMINGS: &[u16] = &[
    // Header
    9000, 4500,

    // Address 0x00FF = 0000 0000 1111 1111 (LSB first)
    560, 560, 560, 560, 560, 560, 560, 560, // 0
    560, 560, 560, 560, 560, 560, 560, 560, // 0
    560, 1690, 560, 1690, 560, 1690, 560, 1690, // 1
    560, 1690, 560, 1690, 560, 1690, 560, 1690, // 1

    // Command 0x20DF = 0010 0000 1101 1111 (LSB first)
    560, 560, 560, 560, 560, 1690, 560, 560,  // 0010
    560, 560, 560, 560, 560, 560, 560, 560,  // 0000
    560, 1690, 560, 1690, 560, 560, 560, 1690, // 1101
    560, 1690, 560, 1690, 560, 1690, 560, 1690, // 1111

    // Repeat bit (optional in NEC protocol)
    560, 1690,
];
pub struct IrRxView<'a> {
    title: &'a str,
    last_signal: Option<IrSignal<'a>>,

    buttons: [Button<'a>;2],
    signal_viewer: IrSignalViewer<'a>
}

impl<'a> IrRxView<'a> {
    pub fn new() -> Self {
        Self {
            title: "IR RX",
            last_signal: Some(IrSignal {
               timings: SAMPLE_TIMINGS
            }),
            buttons: [
                Button::new("RECORD", Point::new(33, 49), Size::new(57, 20)),
                Button::new("REPLAY", Point::new(94, 49), Size::new(57, 20))
            ],
            signal_viewer: IrSignalViewer::new(None, Point::new(63, 19), Size::new(118, 32))
        }
    }
}

impl<'a, D> Viewable<D> for IrRxView<'_>
where
    D: DrawTarget<Color = BinaryColor>
{

    fn run(&mut self, display: &mut impl Display<D>) {
        if let Some(signal) = &self.last_signal {
            self.signal_viewer.set_signal(signal.clone());
            self.signal_viewer.select();
            display.draw(&self.signal_viewer);
        }

        for btn in &self.buttons {
            display.draw(btn);
        }

    }

    fn title(&self) -> &str {
        self.title
    }

}

