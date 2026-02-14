use embedded_graphics::{draw_target::DrawTarget, prelude::*};



use super::view::{ViewContext, Viewable};

use crate::ui::Style;
use crate::ui::elements::Button;
use crate::ui::elements::IrSignalViewer;
use crate::ui::elements::TopBar;

use crate::devices::controller::ControllerEvent;
use crate::devices::display::Display;
use crate::devices::ir::{InfraredCommand, InfraredEvent, IrSignal};

use crate::services::router::{RouterCommand, RouterEvent};

static SAMPLE_TIMINGS: &[u16] = &[
    9000, 4500, 560, 560, 560, 560, 560, 560, 560, 560, 560, 560, 560, 560, 560, 560, 560, 560,
    560, 1690, 560, 1690, 560, 1690, 560, 560, 1690,
];

pub struct IrRxView<'a> {
    topbar: TopBar<'a>,
    last_signal: Option<IrSignal>,

    buttons: [Button<'a>; 2],
    signal_viewer: IrSignalViewer<'a>,

    style: &'a Style,
}

impl<'a> IrRxView<'a> {
    pub fn with_style(style: &'a Style) -> Self {
        Self {
            last_signal: Some(IrSignal::new()),
            topbar: TopBar::new(style, "IR_RX"),
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
        D: Display,
    {
        display.clear(self.style.color_bg)?;

        display.draw(&self.topbar)?;

        display.draw(&self.signal_viewer)?;

        for btn in &self.buttons {
            display.draw(btn)?;
        }

        display.flush();

        Ok(())
    }
}

impl<'a, D> Viewable<D> for IrRxView<'a>
where
    D: Display,
{
    async fn run(
        &mut self,
        context: &mut ViewContext<'_, D>,
    ) -> Result<(), <D::Target as DrawTarget>::Error> {
        loop {
            self.draw(context.display)?;

            let ev = context.receiver.receive().await;

            match ev {
                RouterEvent::ControllerEvent(ev) => match ev {
                    ControllerEvent::NavNextPressed => {
                        if let Some(signal) = &self.last_signal {
                            context
                                .sender
                                .send(RouterCommand::InfraredCommand(InfraredCommand::Play(
                                    signal.clone(),
                                )))
                                .await;
                        }
                    }
                    ControllerEvent::NavPrevPressed => {
                        self.topbar.start_record();
                        context
                            .sender
                            .send(RouterCommand::InfraredCommand(InfraredCommand::Listen))
                            .await;
                    }
                    _ => {}
                },
                RouterEvent::InfraredEvent(ev) => match ev {
                    InfraredEvent::Signal(sig) => {
                        self.topbar.stop_record();

                        self.signal_viewer.set_signal(sig.clone());

                        self.last_signal = Some(sig);
                    }

                    InfraredEvent::NoSignal | InfraredEvent::SignalTooLong => {
                        self.topbar.stop_record()
                    }

                    _ => {}
                },
                _ => {}
            };
        }
    }

    fn title(&self) -> &str {
        self.topbar.title()
    }
}
