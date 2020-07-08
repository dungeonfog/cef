use cef_sys::{cef_pdf_print_margin_type_t, cef_pdf_print_settings_t};

use crate::string::CefString;

#[repr(C)]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum PDFPrintMargin {
    Default = cef_pdf_print_margin_type_t::PDF_PRINT_MARGIN_DEFAULT as isize,
    None = cef_pdf_print_margin_type_t::PDF_PRINT_MARGIN_NONE as isize,
    Minimum = cef_pdf_print_margin_type_t::PDF_PRINT_MARGIN_MINIMUM as isize,
    Custom = cef_pdf_print_margin_type_t::PDF_PRINT_MARGIN_CUSTOM as isize,
}

/// Structure representing PDF print settings.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PDFPrintSettings {
    /// Page title to display in the header. Only used if [enable_header_footer]
    /// is called.
    pub header_footer_title: String,
    /// URL to display in the footer. Only used if [enable_header_footer] is called.
    pub header_footer_url: String,
    /// Output page size in microns. If either this or `page_height` is less than or
    /// equal to zero then the default paper size (A4) will be used.
    pub page_width: i32,
    /// Output page size in microns. If either this or `page_width` is less than or
    /// equal to zero then the default paper size (A4) will be used.
    pub page_height: i32,
    /// The percentage to scale the PDF by before printing (e.g. 50 is 50%).
    /// If this value is less than or equal to zero the default value of 100
    /// will be used.
    pub scale_factor: i32,
    /// Margins in points. Only used if `margin_type` is set to
    /// [PDFPrintMargin::Custom].
    pub margin_top: i32,
    /// Margins in points. Only used if `margin_type` is set to
    /// [PDFPrintMargin::Custom].
    pub margin_right: i32,
    /// Margins in points. Only used if `margin_type` is set to
    /// [PDFPrintMargin::Custom].
    pub margin_bottom: i32,
    /// Margins in points. Only used if `margin_type` is set to
    /// [PDFPrintMargin::Custom].
    pub margin_left: i32,
    /// Margin type.
    pub margin_type: PDFPrintMargin,
    /// Call this to print headers and footers.
    pub header_footer_enabled: bool,
    /// Call to print the selection only or don't to print all.
    pub selection_only: bool,
    /// Call for landscape mode or don't for portrait mode.
    pub landscape: bool,
    /// Call to print background graphics.
    pub backgrounds_enabled: bool,
}

impl From<PDFPrintSettings> for cef_pdf_print_settings_t {
    fn from(settings: PDFPrintSettings) -> cef_pdf_print_settings_t {
        settings.into_raw()
    }
}
impl From<&'_ PDFPrintSettings> for cef_pdf_print_settings_t {
    fn from(settings: &PDFPrintSettings) -> cef_pdf_print_settings_t {
        settings.into_raw()
    }
}

impl PDFPrintSettings {
    fn into_raw(&self) -> cef_pdf_print_settings_t {
        cef_pdf_print_settings_t {
            header_footer_title: CefString::new(&self.header_footer_title).into_raw(),
            header_footer_url: CefString::new(&self.header_footer_url).into_raw(),
            page_width: self.page_width,
            page_height: self.page_height,
            scale_factor: self.scale_factor,
            margin_top: self.margin_top,
            margin_right: self.margin_right,
            margin_bottom: self.margin_bottom,
            margin_left: self.margin_left,
            margin_type: self.margin_type as _,
            header_footer_enabled: self.header_footer_enabled as _,
            selection_only: self.selection_only as _,
            landscape: self.landscape as _,
            backgrounds_enabled: self.backgrounds_enabled as _,
        }
    }
}

impl Default for PDFPrintMargin {
    fn default() -> PDFPrintMargin {
        Self::Default
    }
}
