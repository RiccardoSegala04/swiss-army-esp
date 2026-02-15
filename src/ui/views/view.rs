use embedded_graphics::draw_target::DrawTarget;

use crate::devices::display::Display;

use crate::ui::Style;

use super::dummy_view::DummyView;
use super::main_menu_view::MainMenuView;
use super::ir_rx_view::IrRxView;
use super::ir_saved_view::IrSavedView;

use crate::services::router::{RouterCommand, RouterEvent};

use embassy_sync::channel::{DynamicReceiver, DynamicSender};

pub enum ViewAction {
    SwitchTo(ViewType),
    Exit
}

#[derive(Clone)]
pub enum ViewType {
    MainMenuView,
    IrRxView,
    IrSavedView,
    DummyView(&'static str),
}

impl ViewType {
    pub fn title(&self) -> &'static str {
        match self {
            ViewType::MainMenuView => "PIPU ZERO",
            ViewType::IrRxView => "IR RX",
            ViewType::IrSavedView => "IR SAVED",
            ViewType::DummyView(t) => t,
        }
    }

    pub async fn start<D: Display>(
        &mut self,
        style: &Style,
        context: &mut ViewContext<'_, D>,
    ) -> Result<ViewAction, <D::Target as DrawTarget>::Error> {
        match self {
            ViewType::MainMenuView => MainMenuView::new(style).run(context).await,
            ViewType::IrRxView => IrRxView::new(style).run(context).await,
            ViewType::IrSavedView => IrSavedView::new(style).await.run(context).await,
            ViewType::DummyView(t) => DummyView::new(t).run(context).await
        }
    }
}

// pub enum ViewType<'a> {
//     MainMenuView(MainMenuView<'a>),
//     DummyView(DummyView<'a>),
//     IrRxView(IrRxView<'a>)
// }

// impl<'a> From<DummyView<'a>> for ViewType<'a> {
//     fn from(v: DummyView<'a>) -> ViewType<'a> {
//         ViewType::DummyView(v)
//     }
// }

// impl<'a> From<MainMenuView<'a>> for ViewType<'a> {
//     fn from(v: MainMenuView<'a>) -> ViewType<'a> {
//         ViewType::MainMenuView(v)
//     }
// }

// impl<'a> From<IrRxView<'a>> for ViewType<'a> {
//     fn from(v: IrRxView<'a>) -> ViewType<'a> {
//         ViewType::IrRxView(v)
//     }
// }

pub struct ViewContext<'a, D>
where
    D: Display,
{
    pub display: &'a mut D,
    pub receiver: DynamicReceiver<'static, RouterEvent>,
    pub sender: DynamicSender<'static, RouterCommand>,
}

impl<'a, D> ViewContext<'a, D>
where
    D: Display,
{
    pub fn new(
        display: &'a mut D,
        receiver: DynamicReceiver<'static, RouterEvent>,
        sender: DynamicSender<'static, RouterCommand>,
    ) -> Self {
        Self {
            display,
            receiver,
            sender,
        }
    }
}

pub trait Viewable<D>
where
    D: Display,
{
    async fn run(
        &mut self,
        context: &mut ViewContext<D>,
    ) -> Result<ViewAction, <D::Target as DrawTarget>::Error>;

}
