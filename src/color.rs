use cef_sys::cef_color_t;

/// 32-bit ARGB color value, not premultiplied. The color components are always
/// in a known order. Equivalent to the SkColor type.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color(u32);

impl Color {
    /// Return a [Color] value with the specified component values in range 0.0 to 1.0.
    pub fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self(
            ((alpha * 255.0) as u32).wrapping_shl(24)
                | ((red * 255.0) as u32).wrapping_shl(16)
                | ((green * 255.0) as u32).wrapping_shl(8)
                | ((blue * 255.0) as u32),
        )
    }
    pub(crate) fn get(&self) -> cef_color_t {
        self.0
    }
}
