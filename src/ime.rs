use cef_sys::{cef_color_t, cef_composition_underline_t, cef_range_t};

use crate::{color::Color, values::Range};

/// Structure representing IME composition underline information. This is a thin
/// wrapper around Blink's WebCompositionUnderline class.
pub struct CompositionUnderline {
    /// Underline character range.
    range: Range,
    /// Text color.
    color: Color,
    /// Background color.
    background_color: Color,
    /// Set to true for thick underline.
    thick: bool,
}
