use embedded_graphics::{draw_target::DrawTarget, prelude::*};

use embassy_futures::select::{Either, select};

use embassy_time::{Duration, Timer};

use crate::ui::elements::snake::game::{Direction, GameState};
use crate::ui::elements::snake::snake_viewer::SnakeViewer;

use crate::services::router::RouterEvent;

use crate::devices::controller::ControllerEvent;
use crate::devices::display::Display;

use super::view::{ViewAction, ViewContext, Viewable};

use crate::ui::Style;
use crate::ui::elements::TopBar;

const TOP_BAR_HEIGHT: i32 = 16;
const DISPLAY_WIDTH: i32 = 128;
const FONT_BASELINE: i32 = 8;

const MARKER_SIZE: i32 = 5;
const MARKER_TEXT_GAP: i32 = 2;

const ITEM_HEIGHT: i32 = 10;
const LIST_START_Y: i32 = 22;

pub struct SnakeView<'a> {
    topbar: TopBar<'a>,
    snake: SnakeViewer<'a>,
    style: &'a Style,
}

impl<'a> SnakeView<'a> {
    pub fn new(style: &'a Style) -> Self {
        Self {
            topbar: TopBar::new(style, "SNAKE"),
            snake: SnakeViewer::new(style),
            style,
        }
    }

    pub fn draw<D>(&mut self, display: &mut D) -> Result<(), <D::Target as DrawTarget>::Error>
    where
        D: Display,
    {
        display.clear(self.style.color_bg)?;

        display.draw(&self.topbar)?;

        display.draw(&self.snake)?;

        display.flush();

        Ok(())
    }
}

impl<'a, D> Viewable<D> for SnakeView<'a>
where
    D: Display,
{
    async fn run(
        &mut self,
        context: &mut ViewContext<'_, D>,
    ) -> Result<ViewAction, <D::Target as DrawTarget>::Error> {
        loop {
            self.snake.snake().step();
            self.draw(context.display)?;

            let ev = context.receiver.receive();
            let mut timeout = Timer::after(Duration::from_millis(100));

            let mut snake = self.snake.snake();

            match select(ev, timeout).await {
                Either::First(ev) => {
                    match ev {
                        RouterEvent::ControllerEvent(ev) => {
                            match ev {
                                ControllerEvent::NavNextPressed => {
                                    snake.move_to(match snake.direction() {
                                        Direction::Left => Direction::Up,
                                        Direction::Right => Direction::Down,
                                        Direction::Up => Direction::Right,
                                        Direction::Down => Direction::Left,
                                    })
                                }
                                ControllerEvent::NavPrevPressed => {
                                    snake.move_to(match snake.direction() {
                                        Direction::Left => Direction::Down,
                                        Direction::Right => Direction::Up,
                                        Direction::Up => Direction::Left,
                                        Direction::Down => Direction::Right,
                                    })
                                }
                                ControllerEvent::BackPressed => return Ok(ViewAction::Exit),
                                _ => {}
                            };
                        }
                        _ => {}
                    };
                }
                Either::Second(_) => {
                    snake.step();
                    if let GameState::Ended = snake.state() {
                        return Ok(ViewAction::Exit);
                    }
                }
            }

            self.draw(context.display)?;
        }
    }
}
