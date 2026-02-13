use embedded_graphics::{
    draw_target::DrawTarget,
    prelude::*,
    primitives::Rectangle,
    text::Text,
};

use defmt::info;

use heapless::Vec;

use super::view::{Viewable, ViewContext};

use crate::ui::Style;
use crate::ui::elements::Button;
use crate::ui::elements::IrSignalViewer;

use crate::devices::display::Display;
use crate::devices::ir::{IrSignal, InfraredCommand};
use crate::devices::controller::{ControllerEvent};

use crate::services::router::{RouterEvent, RouterCommand};


static SAMPLE_TIMINGS: &[u16] = &[
    // Header
    9000, 4500, // Address 0x00FF = 0000 0000 1111 1111 (LSB first)
    560, 560, 560, 560, 560, 560, 560, 560, // 0
    560, 560, 560, 560, 560, 560, 560, 560, // 0
    560, 1690, 560, 1690, 560, 1690, 560, 1690, // 1111
    // Repeat bit (optional in NEC protocol)
    560, 1690,
];

pub struct IrRxView<'a> {
    title: &'a str,
    last_signal: Option<IrSignal>,

    buttons: [Button<'a>; 2],
    signal_viewer: IrSignalViewer<'a>,

    style: &'a Style,
}

impl<'a> IrRxView<'a> {
    pub fn with_style(style: &'a Style) -> Self {
        Self {
            title: "IR RX",
            last_signal: Some(IrSignal {
                timings: Vec::from_slice(&SAMPLE_TIMINGS).unwrap(),
            }),
            buttons: [
                Button::new(style, "RECORD", Point::new(33, 53), Size::new(57, 13)),
                Button::new(style, "REPLAY", Point::new(94, 53), Size::new(57, 13)),
            ],
            signal_viewer: IrSignalViewer::new(style, None, Point::new(63, 31), Size::new(118, 23)),
            style,
        }
    }

    fn draw<D>(&self, display: &mut D) -> Result<(), <D::Target as DrawTarget>::Error>
    where
        D: Display
    {
        display.clear(self.style.color_bg)?;

        self.draw_top_bar(display)?;

        display.draw(&self.signal_viewer)?;

        for btn in &self.buttons {
            display.draw(btn)?;
        }

        display.flush();

        Ok(())
    }

    fn draw_top_bar<D>(&self, display: &mut D) -> Result<(), <D::Target as DrawTarget>::Error>
    where
        D: Display
    {
        display.clear(self.style.color_bg)?;

        let bar = Rectangle::new(Point::new(0, 0), Size::new(128, 16)).into_styled(self.style.bar);

        display.draw(&bar)?;

        let title = Text::new(self.title, Point::new(5, 11), self.style.text_bar_big);

        display.draw(&title)?;

        Ok(())
    }
}

impl<'a, D> Viewable<D> for IrRxView<'a>
where
    D: Display
{

    async fn run(
        &mut self,
        context: &mut ViewContext<'_, D>
    ) -> Result<(), <D::Target as DrawTarget>::Error> {

        if let Some(signal) = &self.last_signal {
            self.signal_viewer.set_signal(signal.clone());
            self.signal_viewer.select();
        }

        loop {
            
            self.draw(context.display)?;

            let ev = context.receiver.receive().await;

            match ev {
                RouterEvent::ControllerEvent(ev) => {
                    match ev {
                        ControllerEvent::NavNextPressed => {
                            if let Some(signal) = &self.last_signal {
                                context.sender.send(RouterCommand::InfraredCommand(InfraredCommand::Play(signal.clone()))).await;
                                info!("Transmitted");
                            } else {
                                info!("No transmission saved")
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            };


        }

    }

    fn title(&self) -> &str {
        self.title
    }
}
