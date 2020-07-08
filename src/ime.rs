use cef_sys::{cef_composition_underline_t, cef_composition_underline_style_t};

use crate::{color::Color, values::Range};

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CompositionUnderlineStyle {
    Solid = cef_composition_underline_style_t::CEF_CUS_SOLID as isize,
    Dot = cef_composition_underline_style_t::CEF_CUS_DOT as isize,
    Dash = cef_composition_underline_style_t::CEF_CUS_DASH as isize,
    None = cef_composition_underline_style_t::CEF_CUS_NONE as isize,
}

/// Structure representing IME composition underline information. This is a thin
/// wrapper around Blink's WebCompositionUnderline class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompositionUnderline {
    pub range: Range,
    pub color: Color,
    pub background_color: Color,
    pub thick: bool,
    pub style: CompositionUnderlineStyle,
}

impl Default for CompositionUnderlineStyle {
    fn default() -> Self {
        Self::Solid
    }
}

impl From<&'_ CompositionUnderline> for cef_composition_underline_t {
    fn from(composition_underline: &'_ CompositionUnderline) -> cef_composition_underline_t {
        (*composition_underline).into()
    }
}

impl From<CompositionUnderline> for cef_composition_underline_t {
    fn from(composition_underline: CompositionUnderline) -> cef_composition_underline_t {
        cef_composition_underline_t {
            range: composition_underline.range.into(),
            color: composition_underline.color.get(),
            background_color: composition_underline.background_color.get(),
            thick: composition_underline.thick as _,
            style: composition_underline.style as _,
        }
    }
}
