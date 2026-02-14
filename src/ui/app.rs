
use embedded_graphics::prelude::DrawTarget;

use heapless::Vec;

use crate::devices::display::Display;
use crate::ui::Style;
use crate::ui::views::view::ViewType;
use crate::ui::views::view::ViewAction;
use crate::ViewContext;

pub struct App<'a, D>
where
    D: Display
{
    style: &'a Style,
    context: ViewContext<'a, D>,
    view_stack: Vec<ViewType, 4>
}

impl<'a, D> App<'a, D>
where
    D: Display
{
    pub fn new(style: &'a Style, context: ViewContext<'a, D>) -> Self {
        Self { context, style, view_stack: Vec::new() }
    }

    pub async fn start(&mut self, mut entry: ViewType) -> Result<(), <D::Target as DrawTarget>::Error> {
        
        loop {
            match entry.start(self.style, &mut self.context).await? {
              
                ViewAction::SwitchTo(v) => {
                    self.view_stack.push(entry);
                    entry = v;
                }, 

                ViewAction::Exit => {
                    if let Some(v) = self.view_stack.pop() {
                        entry = v;
                    } else {
                        break;
                    }
                }

            }
        }
        
        Ok(())
    }
}
