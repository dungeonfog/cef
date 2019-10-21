use cef_sys::{
    _cef_select_client_certificate_callback_t, cef_request_handler_t, cef_termination_status_t,
    cef_window_open_disposition_t,
};
use num_enum::UnsafeFromPrimitive;
use std::{ptr::null_mut, sync::Arc};

use crate::{
    browser::Browser,
    frame::Frame,
    load_handler::ErrorCode,
    refcounted::{RefCountedPtr, Wrapper},
    request::Request,
    resource_request_handler::ResourceRequestHandler,
    ssl::{SSLInfo, X509Certificate},
    url_request::{AuthCallback, RequestCallback},
};

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum WindowOpenDisposition {
    Unknown = cef_window_open_disposition_t::WOD_UNKNOWN,
    CurrentTab = cef_window_open_disposition_t::WOD_CURRENT_TAB,
    SingletonTab = cef_window_open_disposition_t::WOD_SINGLETON_TAB,
    NewForegroundTab = cef_window_open_disposition_t::WOD_NEW_FOREGROUND_TAB,
    NewBackgroundTab = cef_window_open_disposition_t::WOD_NEW_BACKGROUND_TAB,
    NewPopup = cef_window_open_disposition_t::WOD_NEW_POPUP,
    NewWindow = cef_window_open_disposition_t::WOD_NEW_WINDOW,
    SaveToDisk = cef_window_open_disposition_t::WOD_SAVE_TO_DISK,
    OffTheRecord = cef_window_open_disposition_t::WOD_OFF_THE_RECORD,
    IgnoreAction = cef_window_open_disposition_t::WOD_IGNORE_ACTION,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum TerminationStatus {
    AbnormalTermination = cef_termination_status_t::TS_ABNORMAL_TERMINATION,
    ProcessWasKilled = cef_termination_status_t::TS_PROCESS_WAS_KILLED,
    ProcessCrashed = cef_termination_status_t::TS_PROCESS_CRASHED,
    ProcessOom = cef_termination_status_t::TS_PROCESS_OOM,
}

/// Implement this structure to handle events related to browser requests. The
/// functions of this structure will be called on the thread indicated.
pub trait RequestHandler: Sync + Send + 'static {
    /// Called on the UI thread before browser navigation. Return true to
    /// cancel the navigation or false (0) to allow the navigation to proceed.
    /// [LoadHandler::on_loading_state_change] will be called twice in all cases.
    /// If the navigation is allowed [LoadHandler::on_load_start] and
    /// [LoadHandler::on_load_end] will be called. If the navigation is canceled
    /// [LoadHandler::on_load_error] will be called with an `errorCode` value of
    /// [ErrorCode::Aborted]. The `user_gesture` value will be true if the browser
    /// navigated via explicit user gesture (e.g. clicking a link) or false if
    /// it navigated automatically (e.g. via the DomContentLoaded event).
    fn on_before_browser(
        &self,
        browser: Browser,
        frame: Frame,
        request: Request,
        user_gesture: bool,
        is_redirect: bool,
    ) -> bool {
        false
    }
    /// Called on the UI thread before OnBeforeBrowse in certain limited cases
    /// where navigating a new or different browser might be desirable. This
    /// includes user-initiated navigation that might open in a special way (e.g.
    /// links clicked via middle-click or ctrl + left-click) and certain types of
    /// cross-origin navigation initiated from the renderer process (e.g.
    /// navigating the top-level frame to/from a file URL). The `browser` and
    /// `frame` values represent the source of the navigation. The
    /// `target_disposition` value indicates where the user intended to navigate
    /// the browser based on standard Chromium behaviors (e.g. current tab, new
    /// tab, etc). The `user_gesture` value will be true if the browser
    /// navigated via explicit user gesture (e.g. clicking a link) or false if
    /// it navigated automatically (e.g. via the DomContentLoaded event). Return
    /// true to cancel the navigation or false to allow the navigation to
    /// proceed in the source browser's top-level frame.
    fn on_open_urlfrom_tab(
        &self,
        browser: Browser,
        frame: Frame,
        target_url: &str,
        target_disposition: WindowOpenDisposition,
        user_gesture: bool,
    ) -> bool {
        false
    }
    /// Called on the browser process IO thread before a resource request is
    /// initiated. The `browser` and `frame` values represent the source of the
    /// request. `request` represents the request contents and cannot be modified
    /// in this callback. `is_navigation` will be true if the resource request
    /// is a navigation. `is_download` will be true if the resource request is
    /// a download. `request_initiator` is the origin (scheme + domain) of the page
    /// that initiated the request. Set `disable_default_handling` to true to
    /// disable default handling of the request, in which case it will need to be
    /// handled via [ResourceRequestHandler::get_resource_handler] or it will
    /// be canceled. To allow the resource load to proceed with default handling
    /// return None. To specify a handler for the resource return a
    /// [ResourceRequestHandler] object. If this callback returns None the
    /// same function will be called on the associated [RequestContextHandler],
    /// if any.
    fn get_resource_request_handler(
        &self,
        browser: Browser,
        frame: Frame,
        request: Request,
        is_navigation: bool,
        is_download: bool,
        request_initiator: &str,
        disable_default_handling: &mut bool,
    ) -> Option<Arc<dyn ResourceRequestHandler>> {
        None
    }
    /// Called on the IO thread when the browser needs credentials from the user.
    /// `origin_url` is the origin making this authentication request. `is_proxy`
    /// indicates whether the host is a proxy server. `host` contains the hostname
    /// and `port` contains the port number. `realm` is the realm of the challenge.
    /// `scheme` is the authentication scheme used, such as
    /// "basic" or "digest", and will be None if the source of the request is an
    /// FTP server. Return true to continue the request and call
    /// [AuthCallback::cont] either in this function or at a later time when
    /// the authentication information is available. Return false to cancel the
    /// request immediately.
    fn get_auth_credentials(
        &self,
        browser: Browser,
        origin_url: &str,
        is_proxy: bool,
        host: &str,
        port: u16,
        realm: Option<&str>,
        scheme: Option<&str>,
        callback: AuthCallback,
    ) -> bool {
        false
    }
    /// Called on the IO thread when JavaScript requests a specific storage quota
    /// size via the webkitStorageInfo.requestQuota function. `origin_url` is the
    /// origin of the page making the request. `new_size` is the requested quota
    /// size in bytes. Return true to continue the request and call
    /// [RequestCallback::cont] either in this function or at a later time to
    /// grant or deny the request. Return false to cancel the request
    /// immediately.
    fn on_quota_request(
        &self,
        browser: Browser,
        origin_url: &str,
        new_size: i64,
        callback: RequestCallback,
    ) -> bool {
        false
    }
    /// Called on the UI thread to handle requests for URLs with an invalid SSL
    /// certificate. Return true and call [RequestCallback::cont] either
    /// in this function or at a later time to continue or cancel the request.
    /// Return false to cancel the request immediately. If
    /// [CefSettings::ignore_certificate_errors] is set all invalid certificates will
    /// be accepted without calling this function.
    fn on_certificate_error(
        &self,
        browser: Browser,
        cert_error: ErrorCode,
        request_url: &str,
        ssl_info: &SSLInfo,
        callback: RequestCallback,
    ) -> bool {
        false
    }
    /// Called on the UI thread when a client certificate is being requested for
    /// authentication. Return false to use the default behavior and
    /// automatically select the first certificate available. Return true and
    /// call [SelectClientCertificateCallback::select] either in this
    /// function or at a later time to select a certificate. Do not call [SelectClientCertificateCallback::select] or
    /// call it with None to continue without using any certificate. `is_proxy`
    /// indicates whether the host is an HTTPS proxy or the origin server. `host`
    /// and `port` contains the hostname and port of the SSL server. `certificates`
    /// is the list of certificates to choose from; this list has already been
    /// pruned by Chromium so that it only contains certificates from issuers that
    /// the server trusts.
    fn on_select_client_certificate(
        &self,
        browser: Browser,
        is_proxy: bool,
        host: &str,
        port: u16,
        certificates: &[X509Certificate],
        callback: SelectClientCertificateCallback,
    ) -> bool {
        false
    }
    /// Called on the browser process UI thread when a plugin has crashed.
    /// `plugin_path` is the path of the plugin that crashed.
    fn on_plugin_crashed(&self, browser: Browser, plugin_path: &str) {}
    /// Called on the browser process UI thread when the render view associated
    /// with `browser|`is ready to receive/handle IPC messages in the render
    /// process.
    fn on_render_view_ready(&self, browser: Browser) {}
    /// Called on the browser process UI thread when the render process terminates
    /// unexpectedly. `status` indicates how the process terminated.
    fn on_render_process_terminated(&self, browser: Browser, status: TerminationStatus) {}
}

#[repr(transparent)]
pub struct RequestHandlerWrapper(Arc<dyn RequestHandler>);

impl RequestHandlerWrapper {
    pub(crate) fn new(delegate: Arc<dyn RequestHandler>) -> Self {
        Self(delegate)
    }
}

impl Wrapper for RequestHandlerWrapper {
    type Cef = cef_request_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_request_handler_t {
                base: unsafe { std::mem::zeroed() },
                // TODO
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
}

ref_counted_ptr! {
    /// Callback structure used for asynchronous continuation of url requests.
    pub struct SelectClientCertificateCallback(*mut _cef_select_client_certificate_callback_t);
}

impl SelectClientCertificateCallback {
    /// Chooses the specified certificate for client certificate authentication.
    /// None value means that no client certificate should be used.
    pub fn select(&self, cert: Option<X509Certificate>) {
        unsafe {
            self.0.select.unwrap()(
                self.0.as_ptr(),
                cert.map(|cert| cert.as_ptr()).unwrap_or_else(null_mut),
            );
        }
    }
}
