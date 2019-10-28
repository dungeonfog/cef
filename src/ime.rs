use cef_sys::{cef_composition_underline_t, cef_range_t};

use crate::{color::Color, values::Range};

/// Structure representing IME composition underline information. This is a thin
/// wrapper around Blink's WebCompositionUnderline class.
pub struct CompositionUnderline(cef_composition_underline_t);

impl CompositionUnderline {
    pub fn new() -> Self {
        Self(unsafe { std::mem::zeroed() })
    }
    pub(crate) fn wrap(underline: cef_composition_underline_t) -> Self {
        Self(underline)
    }
    /// Underline character range.
    pub fn set_range(&mut self, range: Range) {
        self.0.range = range.into();
    }
    /// Underline character range.
    pub fn range(&self) -> Range {
        Range::from(cef_range_t { ..self.0.range })
    }
    /// Text color.
    pub fn set_color(&mut self, color: Color) {
        self.0.color = color.get();
    }
    /// Text color.
    pub fn color(&self) -> Color {
        Color::wrap(self.0.color)
    }
    /// Background color.
    pub fn set_background_color(&mut self, color: Color) {
        self.0.background_color = color.get();
    }
    /// Background color.
    pub fn background_color(&self) -> Color {
        Color::wrap(self.0.background_color)
    }
    /// Set to true for thick underline.
    pub fn set_thick(&mut self, thick: bool) {
        self.0.thick = thick as i32;
    }
    /// True for thick underline.
    pub fn thick(&self) -> bool {
        self.0.thick != 0
    }

    pub(crate) fn as_ptr(&self) -> *const cef_composition_underline_t {
        &self.0
    }
}

impl Default for CompositionUnderline {
    fn default() -> Self {
        Self::new()
    }
}
