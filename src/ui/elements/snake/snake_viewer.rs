use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{CornerRadiiBuilder, Rectangle, RoundedRectangle},
    text::{Alignment, Text},
};

use crate::ui::elements::snake::game::{Direction, Game, GameState, Tile};

use crate::ui::Style;

pub struct SnakeViewer<'a> {
    style: &'a Style,
    snake: Game,
}

impl<'a> SnakeViewer<'a> {
    pub fn new(style: &'a Style) -> Self {
        Self {
            snake: Game::new(),
            style,
        }
    }

    pub fn snake(&mut self) -> &mut Game {
        &mut self.snake
    }
}

impl<'a> Drawable for SnakeViewer<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        for y in 0..self.snake.field().len() {
            for x in 0..self.snake.field()[0].len() {
                match self.snake.field()[y][x] {
                    Tile::Free => {}
                    _ => {
                        Rectangle::new(
                            Point::new(x as i32 * 4, y as i32 * 4 + 16),
                            Size::new(4, 4),
                        )
                        .into_styled(self.style.selected)
                        .draw(target)?;
                    }
                }
            }
        }

        Ok(())
    }
}
