use cef_sys::{cef_pdf_print_margin_type_t, cef_pdf_print_settings_t};

use crate::{
    string::CefString,
};

#[repr(i32)]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum PDFPrintMargin {
    Default = cef_pdf_print_margin_type_t::PDF_PRINT_MARGIN_DEFAULT,
    None = cef_pdf_print_margin_type_t::PDF_PRINT_MARGIN_NONE,
    Minimum = cef_pdf_print_margin_type_t::PDF_PRINT_MARGIN_MINIMUM,
    Custom = cef_pdf_print_margin_type_t::PDF_PRINT_MARGIN_CUSTOM,
}

/// Structure representing PDF print settings.
pub struct PDFPrintSettings(cef_pdf_print_settings_t);

impl PDFPrintSettings {
    pub fn new() -> Self {
        Self( unsafe { std::mem::zeroed() })
    }

    /// Page title to display in the header. Only used if [enable_header_footer]
    /// is called.
    pub fn set_header_footer_title(&mut self, title: &str) {
        self.0.header_footer_title = CefString::new(title).into_raw();
    }

    /// URL to display in the footer. Only used if [enable_header_footer] is called.
    pub fn set_header_footer_url(&mut self, url: &str) {
        self.0.header_footer_url = CefString::new(url).into_raw();
    }

    /// Output page size in microns. If either this or `page_height` is less than or
    /// equal to zero then the default paper size (A4) will be used.
    pub fn set_page_width(&mut self, width: i32) {
        self.0.page_width = width;
    }
    /// Output page size in microns. If either this or `page_width` is less than or
    /// equal to zero then the default paper size (A4) will be used.
    pub fn set_page_height(&mut self, height: i32) {
        self.0.page_height = height;
    }

    /// The percentage to scale the PDF by before printing (e.g. 50 is 50%).
    /// If this value is less than or equal to zero the default value of 100
    /// will be used.
    pub fn set_scale_factor(&mut self, factor: i32) {
        self.0.scale_factor = factor;
    }

    /// Margins in millimeters. Only used if `margin_type` is set to
    /// [PDFPrintMargin::Custom].
    pub fn set_margin_top(&mut self, margin: f64) {
        self.0.margin_top = margin;
    }
    /// Margins in millimeters. Only used if `margin_type` is set to
    /// [PDFPrintMargin::Custom].
    pub fn set_margin_right(&mut self, margin: f64) {
        self.0.margin_right = margin;
    }
    /// Margins in millimeters. Only used if `margin_type` is set to
    /// [PDFPrintMargin::Custom].
    pub fn set_margin_bottom(&mut self, margin: f64) {
        self.0.margin_bottom = margin;
    }
    /// Margins in millimeters. Only used if `margin_type` is set to
    /// [PDFPrintMargin::Custom].
    pub fn set_margin_left(&mut self, margin: f64) {
        self.0.margin_left = margin;
    }

    /// Margin type.
    pub fn set_margin_type(&mut self, margin_type: PDFPrintMargin) {
        self.0.margin_type = margin_type as i32;
    }

    /// Call this to print headers and footers.
    pub fn enable_header_footer(&mut self) {
        self.0.header_footer_enabled = 1;
    }

    /// Call to print the selection only or don't to print all.
    pub fn print_selection_only(&mut self) {
        self.0.selection_only = 1;
    }

    /// Call for landscape mode or don't for portrait mode.
    pub fn enable_landscape(&mut self) {
        self.0.landscape = 1;
    }

    /// Call to print background graphics.
    pub fn enable_backgrounds(&mut self) {
        self.0.backgrounds_enabled = 1;
    }

    pub(crate) fn as_ptr(&self) -> *const cef_pdf_print_settings_t {
        &self.0
    }
}

impl Default for PDFPrintSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PDFPrintSettings {
    fn drop(&mut self) {
        if let Some(dtor) = self.0.header_footer_title.dtor {
            unsafe {
                dtor(self.0.header_footer_title.str);
            }
        }
        if let Some(dtor) = self.0.header_footer_url.dtor {
            unsafe {
                dtor(self.0.header_footer_url.str);
            }
        }
    }
}
