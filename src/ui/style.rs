use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_4X6, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    primitives::PrimitiveStyle,
};

pub const TOP_BAR_HEIGHT: i32 = 16;

pub struct Style {
    pub text_bar_big: MonoTextStyle<'static, BinaryColor>,
    pub text_bar_small: MonoTextStyle<'static, BinaryColor>,
    pub text_big: MonoTextStyle<'static, BinaryColor>,
    pub text_small: MonoTextStyle<'static, BinaryColor>,
    pub bar_bg: PrimitiveStyle<BinaryColor>,
    pub bar_fg: PrimitiveStyle<BinaryColor>,
    pub selected: PrimitiveStyle<BinaryColor>,
    pub deselected: PrimitiveStyle<BinaryColor>,
    pub color_bg: BinaryColor,
    pub color_fg: BinaryColor,
}

impl Style {
    pub fn normal() -> Self {
        Self {
            text_bar_big: MonoTextStyle::new(&FONT_6X10, BinaryColor::Off),
            text_bar_small: MonoTextStyle::new(&FONT_4X6, BinaryColor::Off),
            text_big: MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
            text_small: MonoTextStyle::new(&FONT_4X6, BinaryColor::On),
            bar_bg: PrimitiveStyle::with_fill(BinaryColor::On),
            bar_fg: PrimitiveStyle::with_fill(BinaryColor::Off),
            selected: PrimitiveStyle::with_fill(BinaryColor::On),
            deselected: PrimitiveStyle::with_stroke(BinaryColor::On, 1),
            color_bg: BinaryColor::Off,
            color_fg: BinaryColor::On,
        }
    }

    pub fn inverted() -> Self {
        Self {
            text_bar_big: MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
            text_bar_small: MonoTextStyle::new(&FONT_4X6, BinaryColor::On),
            text_big: MonoTextStyle::new(&FONT_6X10, BinaryColor::Off),
            text_small: MonoTextStyle::new(&FONT_4X6, BinaryColor::Off),
            bar_bg: PrimitiveStyle::with_fill(BinaryColor::Off),
            bar_fg: PrimitiveStyle::with_fill(BinaryColor::On),
            selected: PrimitiveStyle::with_fill(BinaryColor::Off),
            deselected: PrimitiveStyle::with_stroke(BinaryColor::Off, 1),
            color_bg: BinaryColor::On,
            color_fg: BinaryColor::Off,
        }
    }
}
