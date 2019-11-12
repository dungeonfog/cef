use parking_lot::Mutex;
use crate::string::CefStringList;
use crate::settings::LogSeverity;
use cef_sys::cef_size_t;
use crate::values::Size;
use cef_sys::cef_string_t;
use crate::string::CefString;
use cef_sys::cef_string_list_t;
use std::mem::ManuallyDrop;
use crate::{
    browser::{Browser},
    frame::Frame,
    refcounted::{RefCountedPtr, Wrapper},
};
use cef_sys::{
    cef_browser_t,
    cef_frame_t,
    cef_display_handler_t,
    cef_log_severity_t,
};
use std::os::raw::c_int;

ref_counted_ptr!{
    /// Instantiate this structure to handle events related to browser display state.
    pub struct DisplayHandler(*mut cef_display_handler_t);
}

impl DisplayHandler {
    pub fn new<C: DisplayHandlerCallbacks>(callbacks: C) -> DisplayHandler {
        unsafe{ DisplayHandler::from_ptr_unchecked(DisplayHandlerWrapper(Mutex::new(Box::new(callbacks))).wrap().into_raw()) }
    }
}

/// Implement this trait to handle events related to browser display state.
pub trait DisplayHandlerCallbacks: 'static + Send {
    /// Called when a frame's address has changed.
    fn on_address_change(
        &mut self,
        browser: Browser,
        frame: Frame,
        url: &str
    ) {
    }
    /// Called when the page title changes.
    fn on_title_change(
        &mut self,
        browser: Browser,
        title: &str,
    ) {
    }
    /// Called when the page icon changes.
    fn on_favicon_url_change(
        &mut self,
        browser: Browser,
        icon_urls: &[String],
    ) {
    }
    /// Called when web content in the page has toggled fullscreen mode. If
    /// `fullscreen` is `true` the content will automatically be sized to fill
    /// the browser content area. If `fullscreen` is `false` the content will
    /// automatically return to its original size and position. The client is
    /// responsible for resizing the browser if desired.
    fn on_fullscreen_mode_change(
        &mut self,
        browser: Browser,
        fullscreen: bool
    ) {
    }
    /// Called when the browser is about to display a tooltip. `text` contains the
    /// text that will be displayed in the tooltip. To handle the display of the
    /// tooltip yourself return `true`. Otherwise, you can optionally modify
    /// `text` and then return `false` to allow the browser to display the
    /// tooltip. When window rendering is disabled the application is responsible
    /// for drawing tooltips and the return value is ignored.
    fn on_tooltip(
        &mut self,
        browser: Browser,
        text: &mut String,
    ) -> bool {
        false
    }
    /// Called when the browser receives a status message. `value` contains the
    /// text that will be displayed in the status message.
    fn on_status_message(
        &mut self,
        browser: Browser,
        value: &str,
    ) {
    }
    /// Called to display a console message. Return `true` to stop the message
    /// from being output to the console.
    fn on_console_message(
        &mut self,
        browser: Browser,
        level: LogSeverity,
        message: &str,
        source: &str,
        line: usize,
    ) -> bool {
        false
    }
    /// Called when auto-resize is enabled via
    /// cef_browser_host_t::SetAutoResizeEnabled and the contents have auto-
    /// resized. `new_size` will be the desired size in view coordinates. Return
    /// `true` if the resize was handled or `false` for default handling.
    fn on_auto_resize(
        &mut self,
        browser: Browser,
        new_size: Size,
    ) -> bool {
        false
    }
    /// Called when the overall page loading progress has changed. `progress`
    /// ranges from 0.0 to 1.0.
    fn on_loading_progress_change(
        &mut self,
        browser: Browser,
        progress: f64,
    ) {
    }
}

pub struct DisplayHandlerWrapper(Mutex<Box<dyn DisplayHandlerCallbacks>>);

impl Wrapper for DisplayHandlerWrapper {
    type Cef = cef_display_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_display_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_address_change: Some(Self::on_address_change),
                on_title_change: Some(Self::on_title_change),
                on_favicon_urlchange: Some(Self::on_favicon_urlchange),
                on_fullscreen_mode_change: Some(Self::on_fullscreen_mode_change),
                on_tooltip: Some(Self::on_tooltip),
                on_status_message: Some(Self::on_status_message),
                on_console_message: Some(Self::on_console_message),
                on_auto_resize: Some(Self::on_auto_resize),
                on_loading_progress_change: Some(Self::on_loading_progress_change),
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for DisplayHandlerWrapper: cef_display_handler_t {
        fn on_address_change(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t,
            url: &CefString: *const cef_string_t
        ) {
            self.0.lock().on_address_change(
                browser,
                frame,
                &String::from(url),
            );
        }
        fn on_title_change(
            &self,
            browser: Browser: *mut cef_browser_t,
            title: &CefString: *const cef_string_t,
        ) {
            self.0.lock().on_title_change(
                browser,
                &String::from(title),
            );
        }
        fn on_favicon_urlchange(
            &self,
            browser: Browser: *mut cef_browser_t,
            icon_urls: ManuallyDrop<CefStringList>: cef_string_list_t,
        ) {
            self.0.lock().on_favicon_url_change(
                browser,
                &(&*icon_urls).into_iter().map(String::from).collect::<Vec<_>>(),
            );
        }
        fn on_fullscreen_mode_change(
            &self,
            browser: Browser: *mut cef_browser_t,
            fullscreen: c_int: c_int
        ) {
            self.0.lock().on_fullscreen_mode_change(browser, fullscreen != 0);
        }
        fn on_tooltip(
            &self,
            browser: Browser: *mut cef_browser_t,
            text: &mut CefString: *mut cef_string_t,
        ) -> c_int {
            let mut text_rs = String::from(&*text);
            let ret = self.0.lock().on_tooltip(browser, &mut text_rs) as c_int;
            *text = CefString::new(&text_rs);
            ret
        }
        fn on_status_message(
            &self,
            browser: Browser: *mut cef_browser_t,
            value: &CefString: *const cef_string_t,
        ) {
            self.0.lock().on_status_message(
                browser,
                &String::from(value),
            );
        }
        fn on_console_message(
            &self,
            browser: Browser: *mut cef_browser_t,
            level: LogSeverity: cef_log_severity_t::Type,
            message: &CefString: *const cef_string_t,
            source: &CefString: *const cef_string_t,
            line: c_int: c_int,
        ) -> c_int {
            self.0.lock().on_console_message(
                browser,
                level,
                &String::from(message),
                &String::from(source),
                line as usize,
            ) as c_int
        }
        fn on_auto_resize(
            &self,
            browser: Browser: *mut cef_browser_t,
            new_size: &Size: *const cef_size_t,
        ) -> c_int {
            self.0.lock().on_auto_resize(browser, *new_size) as c_int
        }
        fn on_loading_progress_change(
            &self,
            browser: Browser: *mut cef_browser_t,
            progress: f64: f64,
        ) {
            self.0.lock().on_loading_progress_change(browser, progress)
        }
    }
}
