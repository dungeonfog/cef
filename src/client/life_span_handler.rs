use crate::{
    browser::{Browser, BrowserSettings},
    client::{
        Client,
        request_handler::WindowOpenDisposition,
    },
    frame::Frame,
    refcounted::{RefCountedPtr, Wrapper},
    values::DictionaryValue,
    window::WindowInfo,
    string::CefString,
};
use cef_sys::{
    cef_browser_t,
    cef_frame_t,
    cef_string_t,
    _cef_popup_features_t,
    cef_window_info_t,
    cef_client_t,
    cef_browser_settings_t,
    cef_dictionary_value_t,
    cef_window_open_disposition_t,
    cef_life_span_handler_t,
};

ref_counted_ptr!{
    pub struct LifeSpanHandler(*mut cef_life_span_handler_t);
}

impl LifeSpanHandler {
    pub fn new<C: LifeSpanHandlerCallbacks>(callbacks: C) -> LifeSpanHandler {
        unsafe{ LifeSpanHandler::from_ptr_unchecked(LifeSpanHandlerWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }
}

pub trait LifeSpanHandlerCallbacks: 'static + Send + Sync {
    /// Called on the UI thread before a new popup browser is created.
    /// # Parameters
    /// - `browser` and `frame`: source of the popup request.
    /// - `target_url` and `target_frame_name`: where the popup browser should navigate.
    /// - `target_disposition`: where the user intended to open the popup (e.g. current tab, new tab, etc).
    /// - `user_gesture` will be `true` if the popup was opened via explicit user gesture (e.g. clicking a link) or `false` if the popup opened automatically (e.g. via the DomContentLoaded event).
    /// - `popup_features`: contains additional information about the requested popup window.
    /// To allow creation
    /// of the popup browser optionally modify `window_info`, `client`, `settings`
    /// and `no_javascript_access` and return `false`. To cancel creation of the
    /// popup browser return `true`. The `client` and `settings` values will
    /// default to the source browser's values. If the `no_javascript_access` value
    /// is set to `false` the new browser will not be scriptable and may not be
    /// hosted in the same renderer process as the source browser. Any
    /// modifications to `window_info` will be ignored if the parent browser is
    /// wrapped in a cef_browser_view_t. Popup browser creation will be canceled if
    /// the parent browser is destroyed before the popup browser creation completes
    /// (indicated by a call to `on_after_created` for the popup browser). The
    /// `extra_info` parameter provides an opportunity to specify extra information
    /// specific to the created popup browser that will be passed to
    /// cef_render_process_handler_t::on_browser_created() in the render process.
    fn on_before_popup(
        &self,
        browser: Browser,
        frame: Frame,
        target_url: Option<&str>,
        target_frame_name: Option<&str>,
        target_disposition: WindowOpenDisposition,
        user_gesture: bool,
        popup_features: PopupFeatures, //*const _cef_popup_features_t,
        window_info: &mut WindowInfo, //*mut _cef_window_info_t,
        client: &mut Client, // *mut *mut _cef_client_t,
        settings: &mut BrowserSettings, // *mut _cef_browser_settings_t,
        extra_info: &mut DictionaryValue, // *mut *mut _cef_dictionary_value_t,
        no_javascript_access: &mut bool // *mut c_int
    ) -> bool {
        println!("popup window");
        false
    }
    /// Called after a new browser is created. This callback will be the first
    /// notification that references `browser`.
    fn on_after_created(&self, browser: Browser) {

    }

    /// Called when a browser has recieved a request to close. This may result
    /// directly from a call to cef_browser_host_t::*close_browser() or indirectly
    /// if the browser is parented to a top-level window created by CEF and the
    /// user attempts to close that window (by clicking the 'X', for example). The
    /// do_close() function will be called after the JavaScript 'onunload' event
    /// has been fired.
    ///
    /// An application should handle top-level owner window close notifications by
    /// calling cef_browser_host_t::try_close_browser() or
    /// cef_browser_host_t::CloseBrowser(`false`) instead of allowing the window
    /// to close immediately (see the examples below). This gives CEF an
    /// opportunity to process the 'onbeforeunload' event and optionally cancel the
    /// close before do_close() is called.
    ///
    /// When windowed rendering is enabled CEF will internally create a window or
    /// view to host the browser. In that case returning `false` from do_close()
    /// will send the standard close notification to the browser's top-level owner
    /// window (e.g. WM_CLOSE on Windows, performClose: on OS X, "delete_event" on
    /// Linux or cef_window_delegate_t::can_close() callback from Views). If the
    /// browser's host window/view has already been destroyed (via view hierarchy
    /// tear-down, for example) then do_close() will not be called for that browser
    /// since is no longer possible to cancel the close.
    ///
    /// When windowed rendering is disabled returning `false` from do_close()
    /// will cause the browser object to be destroyed immediately.
    ///
    /// If the browser's top-level owner window requires a non-standard close
    /// notification then send that notification from do_close() and return `true`
    /// (1).
    ///
    /// The cef_life_span_handler_t::on_before_close() function will be called
    /// after do_close() (if do_close() is called) and immediately before the
    /// browser object is destroyed. The application should only exit after
    /// on_before_close() has been called for all existing browsers.
    ///
    /// The below examples describe what should happen during window close when the
    /// browser is parented to an application-provided top-level window.
    ///
    /// Example 1: Using cef_browser_host_t::try_close_browser(). This is
    /// recommended for clients using standard close handling and windows created
    /// on the browser process UI thread. 1.  User clicks the window close button
    /// which sends a close notification to
    ///     the application's top-level window.
    /// 2.  Application's top-level window receives the close notification and
    ///     calls TryCloseBrowser() (which internally calls CloseBrowser(false)).
    ///     TryCloseBrowser() returns false so the client cancels the window close.
    /// 3.  JavaScript 'onbeforeunload' handler executes and shows the close
    ///     confirmation dialog (which can be overridden via
    ///     CefJSDialogHandler::OnBeforeUnloadDialog()).
    /// 4.  User approves the close. 5.  JavaScript 'onunload' handler executes. 6.
    /// CEF sends a close notification to the application's top-level window
    ///     (because DoClose() returned false by default).
    /// 7.  Application's top-level window receives the close notification and
    ///     calls TryCloseBrowser(). TryCloseBrowser() returns `true` so the client
    ///     allows the window close.
    /// 8.  Application's top-level window is destroyed. 9.  Application's
    /// on_before_close() handler is called and the browser object
    ///     is destroyed.
    /// 10. Application exits by calling cef_quit_message_loop() if no other
    /// browsers
    ///     exist.
    ///
    /// Example 2: Using cef_browser_host_t::CloseBrowser(`false`) and
    /// implementing the do_close() callback. This is recommended for clients using
    /// non-standard close handling or windows that were not created on the browser
    /// process UI thread. 1.  User clicks the window close button which sends a
    /// close notification to
    ///     the application's top-level window.
    /// 2.  Application's top-level window receives the close notification and:
    ///     A. Calls CefBrowserHost::CloseBrowser(false).
    ///     B. Cancels the window close.
    /// 3.  JavaScript 'onbeforeunload' handler executes and shows the close
    ///     confirmation dialog (which can be overridden via
    ///     CefJSDialogHandler::OnBeforeUnloadDialog()).
    /// 4.  User approves the close. 5.  JavaScript 'onunload' handler executes. 6.
    /// Application's do_close() handler is called. Application will:
    ///     A. Set a flag to indicate that the next close attempt will be allowed.
    ///     B. Return false.
    /// 7.  CEF sends an close notification to the application's top-level window.
    /// 8.  Application's top-level window receives the close notification and
    ///     allows the window to close based on the flag from #6B.
    /// 9.  Application's top-level window is destroyed. 10. Application's
    /// on_before_close() handler is called and the browser object
    ///     is destroyed.
    /// 11. Application exits by calling cef_quit_message_loop() if no other
    /// browsers
    ///     exist.
    fn do_close(&self, browser: Browser) -> bool {
        false
    }
    /// Called just before a browser is destroyed. Release all references to the
    /// browser object and do not attempt to execute any functions on the browser
    /// object (other than GetIdentifier or IsSame) after this callback returns.
    /// This callback will be the last notification that references `browser` on
    /// the UI thread. Any in-progress network requests associated with `browser`
    /// will be aborted when the browser is destroyed, and
    /// cef_resource_request_handler_t callbacks related to those requests may
    /// still arrive on the IO thread after this function is called. See do_close()
    /// documentation for additional usage information.
    fn on_before_close(&self, browser: Browser) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PopupFeatures {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub menu_bar_visible: bool,
    pub status_bar_visible: bool,
    pub tool_bar_visible: bool,
    pub scroll_bars_visible: bool,
}

impl PopupFeatures {
    pub fn new(popup_features: &_cef_popup_features_t) -> PopupFeatures {
        PopupFeatures {
            x: match popup_features.xSet {
                0 => None,
                _ => Some(popup_features.x),
            },
            y: match popup_features.ySet {
                0 => None,
                _ => Some(popup_features.y),
            },
            width: match popup_features.widthSet {
                0 => None,
                _ => Some(popup_features.width),
            },
            height: match popup_features.heightSet {
                0 => None,
                _ => Some(popup_features.height),
            },
            menu_bar_visible: popup_features.menuBarVisible != 0,
            status_bar_visible: popup_features.statusBarVisible != 0,
            tool_bar_visible: popup_features.toolBarVisible != 0,
            scroll_bars_visible: popup_features.scrollbarsVisible != 0,
        }
    }
}

struct LifeSpanHandlerWrapper(Box<dyn LifeSpanHandlerCallbacks>);

impl Wrapper for LifeSpanHandlerWrapper {
    type Cef = cef_life_span_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_life_span_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_before_popup: Some(Self::on_before_popup),
                on_after_created: Some(Self::on_after_created),
                do_close: Some(Self::do_close),
                on_before_close: Some(Self::on_before_close),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for LifeSpanHandlerWrapper: cef_life_span_handler_t {
        fn on_before_popup(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t,
            target_url: Option<&CefString>: *const cef_string_t,
            target_frame_name: Option<&CefString>: *const cef_string_t,
            target_disposition: WindowOpenDisposition: cef_window_open_disposition_t::Type,
            user_gesture: bool: std::os::raw::c_int,
            popup_features: PopupFeatures: *const _cef_popup_features_t,
            window_info: *mut cef_window_info_t: *mut cef_window_info_t,
            client: &mut Client: *mut *mut cef_client_t,
            settings: *mut cef_browser_settings_t: *mut cef_browser_settings_t,
            extra_info: &mut DictionaryValue: *mut *mut cef_dictionary_value_t,
            no_javascript_access: &mut std::os::raw::c_int: *mut std::os::raw::c_int
        ) -> std::os::raw::c_int {
            let window_info_ref = unsafe{ &mut *window_info };
            let mut window_info_rust = unsafe{ WindowInfo::from_raw(&*window_info) };
            let settings_ref = unsafe{ &mut *settings };
            let mut settings_rust = unsafe{ BrowserSettings::from_raw(&*settings) };
            let mut no_javascript_access_rust = *no_javascript_access != 0;
            let ret = self.0.on_before_popup(
                browser,
                frame,
                target_url
                    .map(String::from)
                    .as_ref()
                    .map(|s| &**s),
                target_frame_name
                    .map(String::from)
                    .as_ref()
                    .map(|s| &**s),
                target_disposition,
                user_gesture,
                popup_features,
                &mut window_info_rust,
                client,
                &mut settings_rust,
                extra_info,
                &mut no_javascript_access_rust,
            ) as _;
            *window_info_ref = window_info_rust.into_raw();
            *settings_ref = settings_rust.into_raw();
            *no_javascript_access = no_javascript_access_rust as _;
            ret
        }
        fn on_after_created(&self, browser: Browser: *mut cef_browser_t) {
            self.0.on_after_created(browser);
        }
        fn do_close(&self, browser: Browser: *mut cef_browser_t) -> std::os::raw::c_int {
            self.0.do_close(browser) as _
        }
        fn on_before_close(&self, browser: Browser: *mut cef_browser_t) {
            self.0.on_before_close(browser.clone());
            unsafe{ browser.poison(); }
        }
    }
}
