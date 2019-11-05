use cef_sys::{cef_print_handler_t, _cef_print_settings_t};
use crate::{
    browser::Browser,
    values::Size,
    refcounted::{RefCountedPtr, Wrapper},
};

ref_counted_ptr! {
    pub struct PrintSettings(*mut _cef_print_settings_t);
}

/// Implement this structure to handle printing on Linux. Each browser will have
/// only one print job in progress at a time. The functions of this structure
/// will be called on the browser process UI thread.
pub trait PrintHandler {
    /// Called when printing has started for the specified `browser`. This function
    /// will be called before the other OnPrint*() functions and irrespective of
    /// how printing was initiated (e.g. [BrowserHost::print], JavaScript
    /// window.print() or PDF extension print button).
    fn on_print_start(&self, browser: Browser) {}
    /// Synchronize `settings` with client state. If `get_defaults` is true
    /// then populate `settings` with the default print settings.
    fn on_print_settings(&self, browser: Browser, settings: PrintSettings, get_defaults: bool) {}
    /// Show the print dialog. Execute `callback` once the dialog is dismissed.
    /// Return true if the dialog will be displayed or false to cancel the
    /// printing immediately.
    fn on_print_dialog(&self, browser: Browser, has_selection: bool, callback: Box<dyn FnOnce() + Send>) -> bool { false }
    /// Send the print job to the printer. Execute `callback` once the job is
    /// completed. Return true if the job will proceed or false to cancel
    /// the job immediately.
    fn on_print_job(&self, browser: Browser, document_name: &str, pdf_file_path: &str, callback: Box<dyn FnOnce() + Send>) -> bool { false }
    /// Reset client state related to printing.
    fn on_print_reset(&self, browser: Browser) {}
    /// Return the PDF paper size in device units. Used in combination with
    /// [BrowserHost::print_to_pdf].
    fn get_pdf_paper_size(&self, device_units_per_inch: i32) -> Size { Size::default() }
}


pub(crate) struct PrintHandlerWrapper(*mut cef_print_handler_t);

unsafe impl Send for PrintHandlerWrapper {}
unsafe impl Sync for PrintHandlerWrapper {}

impl Wrapper for PrintHandlerWrapper {
    type Cef = cef_print_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_print_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_print_start: None,
                on_print_settings: None,
                on_print_dialog: None,
                on_print_job: None,
                on_print_reset: None,
                get_pdf_paper_size: None,
            },
            self,
        )
    }
}

impl PrintHandlerWrapper {
    pub(crate) fn new(_: Box<dyn PrintHandler>) -> Self {
        unimplemented!()
    }
}
