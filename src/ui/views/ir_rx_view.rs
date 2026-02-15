use embedded_graphics::{draw_target::DrawTarget, prelude::*};

use super::view::{ViewAction, ViewContext, Viewable};

use crate::ui::Style;
use crate::ui::elements::Button;
use crate::ui::elements::ElementType;
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

    signal_viewer: IrSignalViewer<'a>,
    elements: [ElementType<'a>; 2],
    sel_idx: usize,

    style: &'a Style,
}

impl<'a> IrRxView<'a> {
    pub async fn new(style: &'a Style) -> Self {
        let last_sig = crate::devices::ir::SIGNAL_HISTORY
            .get()
            .lock()
            .await
            .last()
            .cloned();
        Self {
            last_signal: last_sig.clone(),
            topbar: TopBar::new(style, "IR_RX"),
            signal_viewer: IrSignalViewer::selected_new(
                style,
                last_sig.clone(),
                Point::new(63, 31),
                Size::new(118, 23),
            ),
            elements: [
                Button::selected_new(style, "RECORD", Point::new(33, 53), Size::new(57, 13)).into(),
                Button::new(style, "REPLAY", Point::new(94, 53), Size::new(57, 13)).into(),
            ],
            sel_idx: 0,
            style,
        }
    }

    fn select_next(&mut self) {
        self.elements[self.sel_idx].deselect();
        self.sel_idx = (self.sel_idx + 1) % self.elements.len();
        self.elements[self.sel_idx].select();
    }

    fn select_prev(&mut self) {
        self.elements[self.sel_idx].deselect();
        self.sel_idx = (self.sel_idx + self.elements.len() - 1) % self.elements.len();
        self.elements[self.sel_idx].select();
    }

    async fn confirm_pressed<D: Display>(&mut self, context: &mut ViewContext<'_, D>) {
        match self.sel_idx {
            0 => {
                self.topbar.start_record();
                context
                    .sender
                    .send(RouterCommand::InfraredCommand(InfraredCommand::Listen))
                    .await;
            }
            1 => {
                if let Some(signal) = &self.last_signal {
                    self.topbar.start_record();
                    context
                        .sender
                        .send(RouterCommand::InfraredCommand(InfraredCommand::Play(
                            signal.clone(),
                        )))
                        .await;
                }
            }
            _ => {}
        }
    }

    async fn handle_infrared_event(&mut self, ev: InfraredEvent) {
        match ev {
            InfraredEvent::Signal(sig) => {
                self.topbar.stop_record();

                self.signal_viewer.set_signal(sig.clone());

                self.last_signal = Some(sig);
            }
            InfraredEvent::NoSignal | InfraredEvent::SignalTooLong => self.topbar.stop_record(),
            InfraredEvent::SignalPlayed => self.topbar.stop_record(),
            _ => {}
        }
    }

    fn draw<D>(&self, display: &mut D) -> Result<(), <D::Target as DrawTarget>::Error>
    where
        D: Display,
    {
        display.clear(self.style.color_bg)?;

        display.draw(&self.topbar)?;

        display.draw(&self.signal_viewer)?;

        for e in &self.elements {
            display.draw(e)?;
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
    ) -> Result<ViewAction, <D::Target as DrawTarget>::Error> {
        loop {
            self.draw(context.display)?;

            let ev = context.receiver.receive().await;

            match ev {
                RouterEvent::ControllerEvent(ev) => match ev {
                    ControllerEvent::NavNextPressed => self.select_next(),
                    ControllerEvent::NavPrevPressed => self.select_prev(),
                    ControllerEvent::ConfirmPressed => self.confirm_pressed(context).await,
                    ControllerEvent::BackPressed => return Ok(ViewAction::Exit),
                    _ => {}
                },
                RouterEvent::InfraredEvent(ev) => self.handle_infrared_event(ev).await,
                _ => {}
            };
        }
    }
}
