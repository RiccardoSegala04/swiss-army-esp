use heapless::Vec;
use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{CornerRadiiBuilder, Rectangle, RoundedRectangle},
    text::{Alignment, Text},
};

use crate::devices::display::Display;

use crate::ui::Style;

pub struct List<'a> {
    selected: bool,
    position: Point,
    size: Size, // store width and height
    style: &'a Style,
    elements: Vec<&'a str, 10>,
    sel_idx: usize
}

impl<'a> List<'a> {
    pub fn new(style: &'a Style, position: Point, size: Size, elements: Vec<&'a str, 10>) -> Self {
        Self {
            selected: false,
            position,
            size,
            style,
            elements,
            sel_idx: 0
        }
    }

    pub fn selected_new(style: &'a Style, position: Point, size: Size, elements: Vec<&'a str, 10>) -> Self {
        Self {
            selected: true,
            position,
            size,
            style,
            elements,
            sel_idx: 0
        }
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn select(&mut self) {
        self.selected = true;
    }

    pub fn deselect(&mut self) {
        self.selected = false;
    }

    pub fn select_next(&mut self) {
        self.sel_idx = (self.sel_idx + 1) % self.elements.len();
    }

    pub fn select_prev(&mut self) {
        self.sel_idx = (self.sel_idx + self.elements.len()-1) % self.elements.len();
    }

    pub fn selected_index(&mut self) -> usize {
        self.sel_idx
    }
}

impl<'a> Drawable for List<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        
        if !self.elements.is_empty() {

            let mut y = 25;
            let len = self.elements.len();

            let start = if self.sel_idx <= 1 || self.elements.len() <= 4 {
                0
            } else if self.sel_idx < self.elements.len()-2 {
                self.sel_idx - 1
            } else {
                self.elements.len() - 4
            };
     
            let elements = &self.elements[start..start+self.elements.len().min(4)];

            for (idx, element) in elements.iter().enumerate() {

                draw_item_text(target, &element, idx == self.sel_idx-start, y, self.style)?;

                y += 12;
            }
        }

        Ok(())
    }
}

fn draw_item_text<D>(
    target: &mut D,
    element: &str,
    selected: bool,
    y: i32,
    style: &Style,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{

    let color = match selected {
        true => style.text_selected_big,
        false => style.text_big,
    };
   
    let text = Text::new(
        element,
        Point::new(5, y),
        color,
    ).draw(target)?;

    Ok(())
}

