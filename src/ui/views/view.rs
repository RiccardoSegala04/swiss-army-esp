use embedded_graphics::draw_target::DrawTarget;

use crate::devices::display::Display;

use crate::ui::Style;

use super::dummy_view::DummyView;
use super::list_view::ListView;
use super::ir_rx_view::IrRxView;

use crate::services::router::{RouterCommand, RouterEvent};

use embassy_sync::channel::{DynamicReceiver, DynamicSender};

pub enum ViewAction {
    SwitchTo(ViewType),
    Exit
}

#[derive(Clone)]
pub enum ViewType {
    ListView,
    IrRxView,
    DummyView(&'static str),
}

impl ViewType {
    pub fn title(&self) -> &'static str {
        match self {
            ViewType::ListView => "PIPU ZERO",
            ViewType::IrRxView => "IR RX",
            ViewType::DummyView(t) => t,
        }
    }

    pub async fn start<D: Display>(
        &mut self,
        style: &Style,
        context: &mut ViewContext<'_, D>,
    ) -> Result<ViewAction, <D::Target as DrawTarget>::Error> {
        match self {
            ViewType::ListView => ListView::new(style).run(context).await,
            ViewType::IrRxView => IrRxView::new(style).run(context).await,
            ViewType::DummyView(t) => DummyView::new(t).run(context).await
        }
    }
}

// pub enum ViewType<'a> {
//     ListView(ListView<'a>),
//     DummyView(DummyView<'a>),
//     IrRxView(IrRxView<'a>)
// }

// impl<'a> From<DummyView<'a>> for ViewType<'a> {
//     fn from(v: DummyView<'a>) -> ViewType<'a> {
//         ViewType::DummyView(v)
//     }
// }

// impl<'a> From<ListView<'a>> for ViewType<'a> {
//     fn from(v: ListView<'a>) -> ViewType<'a> {
//         ViewType::ListView(v)
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
