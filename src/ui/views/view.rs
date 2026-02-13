use embedded_graphics::{
    draw_target::DrawTarget
};

use crate::devices::display::Display;

use super::dummy_view::DummyView;
use super::list_view::ListView;

use crate::services::router::{RouterEvent};

use embassy_sync::channel::{DynamicReceiver};

pub struct ViewContext<'a, D>
where
    D: Display,
{
    pub display: &'a mut D,
    pub receiver: DynamicReceiver<'static, RouterEvent>,
}

impl<'a, D> ViewContext<'a, D>
where
    D: Display
{
    pub fn new(display: &'a mut D, receiver: DynamicReceiver<'static, RouterEvent>) -> Self {
        Self {
            display,
            receiver
        }       
    }
}

pub trait Viewable<D>
where
    D: Display
{
    async fn run(
        &mut self,
        context: &mut ViewContext<D>
    ) -> Result<(), <D::Target as DrawTarget>::Error>;

    fn title(&self) -> &str;
}

pub enum ViewType<'a> {
    ListView(ListView<'a>),
    DummyView(DummyView<'a>),
}

impl<'a, D> Viewable<D> for ViewType<'a>
where
    D: Display,
{
    async fn run(
        &mut self,
        context: &mut ViewContext<'_, D>
    ) -> Result<(), <D::Target as DrawTarget>::Error> {
        
        match self {
            ViewType::ListView(v) => v.run(context).await?,
            ViewType::DummyView(v) => v.run(context).await?,
        };

        Ok(())
    }

    fn title(&self) -> &str {
        match self {
            ViewType::ListView(v) => <ListView<'_> as Viewable<D>>::title(v),
            ViewType::DummyView(v) => <DummyView<'_> as Viewable<D>>::title(v),
        }
    }
}

impl<'a> From<DummyView<'a>> for ViewType<'a> {
    fn from(v: DummyView<'a>) -> ViewType<'a> {
        ViewType::DummyView(v)
    }
}

impl<'a> From<ListView<'a>> for ViewType<'a> {
    fn from(v: ListView<'a>) -> ViewType<'a> {
        ViewType::ListView(v)
    }
}
