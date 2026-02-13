use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::MonoTextStyle,
    mono_font::MonoTextStyleBuilder,
    mono_font::ascii::FONT_4X6,
    mono_font::ascii::FONT_6X10,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::{Text, TextStyle, TextStyleBuilder},
};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, Receiver, Sender};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use super::view::ViewType;
use super::view::Viewable;

use crate::ui::Style;
use crate::ui::elements::Button;
use crate::ui::elements::IrSignalViewer;

use crate::devices::display::Display;
use crate::devices::ir::IrSignal;

use crate::services::router::{RouterCommand, RouterEvent};

static SAMPLE_TIMINGS: &[u16] = &[
    // Header
    9000, 4500, // Address 0x00FF = 0000 0000 1111 1111 (LSB first)
    560, 560, 560, 560, 560, 560, 560, 560, // 0
    560, 560, 560, 560, 560, 560, 560, 560, // 0
    560, 1690, 560, 1690, 560, 1690, 560, 1690, // 1
    560, 1690, 560, 1690, 560, 1690, 560, 1690, // 1
    // Command 0x20DF = 0010 0000 1101 1111 (LSB first)
    560, 560, 560, 560, 560, 1690, 560, 560, // 0010
    560, 560, 560, 560, 560, 560, 560, 560, // 0000
    560, 1690, 560, 1690, 560, 560, 560, 1690, // 1101
    560, 1690, 560, 1690, 560, 1690, 560, 1690, // 1111
    // Repeat bit (optional in NEC protocol)
    560, 1690,
];
pub struct IrRxView<'a> {
    title: &'a str,
    last_signal: Option<IrSignal<'a>>,

    buttons: [Button<'a>; 2],
    signal_viewer: IrSignalViewer<'a>,

    style: &'a Style,
}

impl<'a> IrRxView<'a> {
    pub fn with_style(style: &'a Style) -> Self {
        Self {
            title: "IR RX",
            last_signal: Some(IrSignal {
                timings: SAMPLE_TIMINGS,
            }),
            buttons: [
                Button::new(style, "RECORD", Point::new(33, 53), Size::new(57, 13)),
                Button::new(style, "REPLAY", Point::new(94, 53), Size::new(57, 13)),
            ],
            signal_viewer: IrSignalViewer::new(style, None, Point::new(63, 31), Size::new(118, 23)),
            style,
        }
    }

    fn draw<D>(&self, display: &mut impl Display<D>)
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        display.clear(self.style.color_bg);

        self.draw_top_bar(display);

        display.draw(&self.signal_viewer);

        for btn in &self.buttons {
            display.draw(btn);
        }

        display.flush();
    }

    fn draw_top_bar<D>(&self, display: &mut impl Display<D>)
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        display.clear(self.style.color_bg);

        let bar = Rectangle::new(Point::new(0, 0), Size::new(128, 16)).into_styled(self.style.bar);

        display.draw(&bar);

        let title = Text::new(self.title, Point::new(5, 11), self.style.text_bar_big);

        display.draw(&title);
    }
}

impl<'a, D> Viewable<D> for IrRxView<'a>
where
    D: DrawTarget<Color = BinaryColor>,
{
    async fn run(
        &mut self,
        display: &mut impl Display<D>,
        receiver: DynamicReceiver<'static, RouterEvent>,
    ) {
        if let Some(signal) = &self.last_signal {
            self.signal_viewer.set_signal(signal.clone());
            self.signal_viewer.select();
        }

        self.draw(display);
    }

    fn title(&self) -> &str {
        self.title
    }
}
