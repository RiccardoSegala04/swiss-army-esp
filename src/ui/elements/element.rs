use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::BinaryColor,
    prelude::*,
};

use crate::ui::elements::{ir_signal_viewer, button};

pub enum ElementType<'a> {
    Button(button::Button<'a>),
    IrSignalViewer(ir_signal_viewer::IrSignalViewer<'a>),
}

macro_rules! delegate_method {
    ($fn_name:ident $(, $arg:ident : $ty:ty)*) => {
        pub fn $fn_name(&mut self $(, $arg : $ty)*) {
            match self {
                ElementType::Button(e) => e.$fn_name($($arg),*),
                ElementType::IrSignalViewer(e) => e.$fn_name($($arg),*),
            }
        }
    };
}

impl<'a> ElementType<'a> {
    delegate_method!(select);
    delegate_method!(deselect);
}

impl<'a> Drawable for ElementType<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        match self {
            ElementType::Button(e) => e.draw(target),
            ElementType::IrSignalViewer(e) => e.draw(target),
        }
    }
}

macro_rules! impl_from_element {
    ($variant:ident, $ty:ty) => {
        impl<'a> From<$ty> for ElementType<'a> {
            fn from(e: $ty) -> ElementType<'a> {
                ElementType::$variant(e)
            }
        }
    };
}

impl_from_element!(Button, button::Button<'a>);
impl_from_element!(IrSignalViewer, ir_signal_viewer::IrSignalViewer<'a>);
