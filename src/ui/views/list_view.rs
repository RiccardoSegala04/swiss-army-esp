use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    text::Text,
    draw_target::DrawTarget,
};

use crate::devices::display::Display;
use super::view::{Viewable, ViewType};

const DISPLAY_WIDTH: i32 = 128;
const TOP_BAR_HEIGHT: i32 = 16;

const FONT_BASELINE: i32 = 8;

const MARKER_SIZE: i32 = 5;
const MARKER_TEXT_GAP: i32 = 2;

const ITEM_HEIGHT: i32 = 10;
const LIST_START_Y: i32 = 22;


struct Styles {
    text_on: MonoTextStyle<'static, BinaryColor>,
    text_off: MonoTextStyle<'static, BinaryColor>,
    bar: PrimitiveStyle<BinaryColor>,
    marker_on: PrimitiveStyle<BinaryColor>,
    marker_empty: PrimitiveStyle<BinaryColor>,
}

impl Styles {
    fn new() -> Self {
        Self {
            text_on: MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
            text_off: MonoTextStyle::new(&FONT_6X10, BinaryColor::Off),
            bar: PrimitiveStyle::with_fill(BinaryColor::On),
            marker_on: PrimitiveStyle::with_fill(BinaryColor::On),
            marker_empty: PrimitiveStyle::with_stroke(BinaryColor::On, 1),
        }
    }
}


pub struct ListView<'a> {
    title: &'a str,
    elements: &'a mut [ViewType<'a>],
    sel_idx: usize,
    vpad: u8,
    hpad: u8,
}

impl<'a> ListView<'a> {
    pub fn new(title: &'a str, elements: &'a mut [ViewType<'a>]) -> Self {
        Self {
            title,
            elements,
            sel_idx: 0,
            vpad: 2,
            hpad: 2,
        }
    }

    pub fn draw<D>(&mut self, display: &mut impl Display<D>)
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        display.clear();

        let styles = Styles::new();

        self.draw_top_bar(display, &styles);
        self.draw_list(display, &styles);

        display.flush();
    }

    fn draw_top_bar<D>(&self, display: &mut impl Display<D>, styles: &Styles)
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let bar = Rectangle::new(
            Point::new(0, 0),
            Size::new(DISPLAY_WIDTH as u32, TOP_BAR_HEIGHT as u32),
        )
        .into_styled(styles.bar);

        display.draw(&bar);

        let title = Text::new(
            self.title,
            Point::new(self.hpad as i32, self.vpad as i32 + FONT_BASELINE),
            styles.text_off,
        );

        display.draw(&title);
    }

    fn draw_list<D>(&mut self, display: &mut impl Display<D>, styles: &Styles)
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let sel_idx = self.sel_idx;
        let hpad = self.hpad;
        let vpad = self.vpad;

        let mut y = LIST_START_Y + vpad as i32;

        for (idx, element) in self.elements.iter_mut().enumerate() {
            draw_marker(display, idx, sel_idx, hpad, y, styles);
            draw_item_text(display, element, hpad, y, styles);

            y += ITEM_HEIGHT + vpad as i32;
        }
    }
}

impl<'a, D> Viewable<D> for ListView<'a>
where
    D: DrawTarget<Color = BinaryColor>,
{
    fn run(&mut self, display: &mut impl Display<D>) {

        // Loop which handles events, generates commands, and draws on the screen
        
        self.draw(display);
    }

    fn title(&self) -> &str {
        self.title
    }
}


fn draw_marker<D>(
    display: &mut impl Display<D>,
    idx: usize,
    sel_idx: usize,
    hpad: u8,
    y: i32,
    styles: &Styles,
) where
    D: DrawTarget<Color = BinaryColor>,
{
    let base = Rectangle::new(
        Point::new(hpad as i32, y - MARKER_SIZE),
        Size::new(MARKER_SIZE as u32, MARKER_SIZE as u32),
    );

    let marker = if idx == sel_idx {
        base.into_styled(styles.marker_on)
    } else {
        base.into_styled(styles.marker_empty)
    };

    display.draw(&marker);
}

fn draw_item_text<D>(
    display: &mut impl Display<D>,
    element: &ViewType<'_>,
    hpad: u8,
    y: i32,
    styles: &Styles,
) where
    D: DrawTarget<Color = BinaryColor>,
{
    let text = Text::new(
        <ViewType<'_> as Viewable<D>>::title(element),
        Point::new(
            hpad as i32 + MARKER_SIZE + MARKER_TEXT_GAP,
            y,
        ),
        styles.text_on,
    );

    display.draw(&text);
}
