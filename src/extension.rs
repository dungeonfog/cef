use crate::stream::StreamReader;
use crate::browser::BrowserSettings;
use crate::window::WindowInfo;
use cef_sys::cef_browser_t;
use cef_sys::cef_window_info_t;
use crate::browser::Browser;
use cef_sys::cef_get_extension_resource_callback_t;

use crate::load_handler::ErrorCode;
use cef_sys::cef_string_t;
use cef_sys::cef_client_t;
use cef_sys::cef_browser_settings_t;
use std::os::raw::c_int;
use crate::client::Client;
use cef_sys::{cef_extension_t, cef_extension_handler_t, cef_errorcode_t};
use std::{
    collections::HashMap,
    path::PathBuf,
};

use crate::{
    request_context::RequestContext,
    string::CefString,
    send_cell::SendCell,
    refcounted::{RefCountedPtr, Wrapper},
    values::{DictionaryValue, StoredValue},
};

ref_counted_ptr! {
    /// Object representing an extension. Methods may be called on any thread unless
    /// otherwise indicated.
    pub struct Extension(*mut cef_extension_t);
}

ref_counted_ptr!{
    pub struct ExtensionHandler(*mut cef_extension_handler_t);
}

ref_counted_ptr!{
    pub struct GetExtensionResourceCallback(*mut cef_get_extension_resource_callback_t);
}

impl Extension {
    /// Returns the unique extension identifier. This is calculated based on the
    /// extension public key, if available, or on the extension path. See
    /// https://developer.chrome.com/extensions/manifest/key for details.
    pub fn get_identifier(&self) -> String {
        self.0
            .get_identifier
            .and_then(|get_identifier| unsafe { get_identifier(self.as_ptr()).as_mut() })
            .map(unsafe{ |s| CefString::from_userfree_unchecked(s) })
            .map(String::from)
            .unwrap_or_default()
    }

    /// Returns the absolute path to the extension directory on disk. This value
    /// will be prefixed with PK_DIR_RESOURCES if a relative path was passed to
    /// [RequestContext::load_extension].
    pub fn get_path(&self) -> PathBuf {
        self.0
            .get_path
            .and_then(|get_path| unsafe { get_path(self.as_ptr()).as_mut() })
            .map(unsafe{ |s| CefString::from_userfree_unchecked(s) })
            .map(String::from)
            .map(PathBuf::from)
            .unwrap_or_default()
    }
    // Returns the extension manifest contents as a dictionary object.
    // See https://developer.chrome.com/extensions/manifest for details.
    pub fn get_manifest(&self) -> HashMap<String, StoredValue> {
        self.0
            .get_manifest
            .and_then(|get_manifest| unsafe { DictionaryValue::from_ptr(get_manifest(self.0.as_ptr())) })
            .map(DictionaryValue::into)
            .unwrap_or_default()
    }
    /// Returns the handler for this extension. Will return None for internal
    /// extensions or if no handler was passed to [RequestContext::load_extension].
    pub fn get_handler(&self) -> Option<ExtensionHandler> {
        self.0
            .get_handler
            .and_then(|get_handler| unsafe{ ExtensionHandler::from_ptr(get_handler(self.as_ptr()))})
    }
    /// Returns the request context that loaded this extension. Will return None
    /// for internal extensions or if the extension has been unloaded. See the
    /// [RequestContext::load_extension] documentation for more information
    /// about loader contexts. Must be called on the browser process UI thread.
    pub fn get_loader_context(&self) -> Option<RequestContext> {
        self.0
            .get_loader_context
            .and_then(|get_loader_context| unsafe {
                RequestContext::from_ptr(get_loader_context(self.0.as_ptr()))
            })
    }
    /// Returns `true` if this extension is currently loaded. Must be called on
    /// the browser process UI thread.
    pub fn is_loaded(&self) -> bool {
        self.0
            .is_loaded
            .map(|is_loaded| unsafe { is_loaded(self.0.as_ptr()) != 0 })
            .unwrap_or_default()
    }
    /// Unload this extension if it is not an internal extension and is currently
    /// loaded. Will result in a call to
    /// [ExtensionHandler::on_extension_unloaded] on success.
    pub fn unload(&self) {
        if let Some(unload) = self.0.unload {
            unsafe {
                unload(self.0.as_ptr());
            }
        }
    }
}

impl GetExtensionResourceCallback {
    pub fn cont(&self, stream: StreamReader) {
        unsafe { self.0.cont.unwrap()(self.as_ptr(), stream.into_raw()) }
    }
    pub fn cancel(&self) {
        unsafe { self.0.cancel.unwrap()(self.as_ptr()) }
    }
}

impl ExtensionHandler {
    pub fn new<C: ExtensionHandlerCallbacks>(callbacks: C) -> ExtensionHandler {
        unsafe{ ExtensionHandler::from_ptr_unchecked(ExtensionHandlerWrapper::new(Box::new(callbacks)).wrap().into_raw()) }
    }
}

/// Implement this trait to handle events related to browser extensions. The
/// functions of this trait will be called on the UI thread. See
/// [RequestContext::load_extension] for information about extension loading.
pub trait ExtensionHandlerCallbacks: 'static + Send {
    /// Called if the cef_request_tContext::LoadExtension request fails. `result`
    /// will be the error code.
    fn on_extension_load_failed(
        &mut self,
        result: ErrorCode
    );
    /// Called if the cef_request_tContext::LoadExtension request succeeds.
    /// `extension` is the loaded extension.
    fn on_extension_loaded(
        &mut self,
        extension: Extension
    );
    /// Called after the cef_extension_t::Unload request has completed.
    fn on_extension_unloaded(
        &mut self,
        extension: Extension
    );
    /// Called when an extension needs a browser to host a background script
    /// specified via the "background" manifest key. The browser will have no
    /// visible window and cannot be displayed. `extension` is the extension that
    /// is loading the background script. `url` is an internally generated
    /// reference to an HTML page that will be used to load the background script
    /// via a <script> src attribute. To allow creation of the browser optionally
    /// modify `client` and `settings` and return `false`. To cancel creation of
    /// the browser (and consequently cancel load of the background script) return
    /// `true`. Successful creation will be indicated by a call to
    /// cef_life_span_handler_t::OnAfterCreated, and
    /// cef_browser_host_t::IsBackgroundHost will return `true` for the resulting
    /// browser. See https://developer.chrome.com/extensions/event_pages for more
    /// information about extension background script usage.
    fn on_before_background_browser(
        &mut self,
        extension: Extension,
        url: &str,
        client: &mut Client,
        settings: &mut BrowserSettings
    ) -> bool;
    /// Called when an extension API (e.g. chrome.tabs.create) requests creation of
    /// a new browser. `extension` and `browser` are the source of the API call.
    /// `active_browser` may optionally be specified via the windowId property or
    /// returned via the get_active_browser() callback and provides the default
    /// `client` and `settings` values for the new browser. `index` is the position
    /// value optionally specified via the index property. `url` is the URL that
    /// will be loaded in the browser. `active` is `true` if the new browser
    /// should be active when opened.  To allow creation of the browser optionally
    /// modify `windowInfo`, `client` and `settings` and return `false`. To
    /// cancel creation of the browser return `true`. Successful creation will be
    /// indicated by a call to cef_life_span_handler_t::OnAfterCreated. Any
    /// modifications to `windowInfo` will be ignored if `active_browser` is
    /// wrapped in a cef_browser_view_t.
    fn on_before_browser(
        &mut self,
        extension: Extension,
        browser: Browser,
        active_browser: Browser,
        index: usize,
        url: &str,
        active: bool,
        window_info: &mut WindowInfo,
        client: &mut Client,
        settings: &mut BrowserSettings,
    ) -> bool;
    /// Called when no tabId is specified to an extension API call that accepts a
    /// tabId parameter (e.g. chrome.tabs.*). `extension` and `browser` are the
    /// source of the API call. Return the browser that will be acted on by the API
    /// call or return NULL to act on `browser`. The returned browser must share
    /// the same cef_request_tContext as `browser`. Incognito browsers should not
    /// be considered unless the source extension has incognito access enabled, in
    /// which case `include_incognito` will be `true`.
    fn get_active_browser(
        &mut self,
        extension: Extension,
        browser: Browser,
        include_incognito: bool
    ) -> Browser;
    /// Called when the tabId associated with `target_browser` is specified to an
    /// extension API call that accepts a tabId parameter (e.g. chrome.tabs.*).
    /// `extension` and `browser` are the source of the API call. Return `true`
    /// to allow access of `false` to deny access. Access to incognito browsers
    /// should not be allowed unless the source extension has incognito access
    /// enabled, in which case `include_incognito` will be `true`.
    fn can_access_browser(
        &mut self,
        extension: Extension,
        browser: Browser,
        include_incognito: bool,
        target_browser: Browser,
    ) -> bool;
    /// Called to retrieve an extension resource that would normally be loaded from
    /// disk (e.g. if a file parameter is specified to chrome.tabs.executeScript).
    /// `extension` and `browser` are the source of the resource request. `file` is
    /// the requested relative file path. To handle the resource request return
    /// `true` and execute `callback` either synchronously or asynchronously. For
    /// the default behavior which reads the resource from the extension directory
    /// on disk return `false`. Localization substitutions will not be applied to
    /// resources handled via this function.
    fn get_extension_resource(
        &mut self,
        extension: Extension,
        browser: Browser,
        file: &str,
        callback: GetExtensionResourceCallback,
    ) -> bool;
}

struct ExtensionHandlerWrapper {
    delegate: SendCell<Box<dyn ExtensionHandlerCallbacks>>
}

impl ExtensionHandlerWrapper {
    fn new(delegate: Box<dyn ExtensionHandlerCallbacks>) -> ExtensionHandlerWrapper {
        ExtensionHandlerWrapper {
            delegate: SendCell::new(delegate)
        }
    }
}

impl Wrapper for ExtensionHandlerWrapper {
    type Cef = cef_extension_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_extension_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_extension_load_failed: Some(Self::on_extension_load_failed),
                on_extension_loaded: Some(Self::on_extension_loaded),
                on_extension_unloaded: Some(Self::on_extension_unloaded),
                on_before_background_browser: Some(Self::on_before_background_browser),
                on_before_browser: Some(Self::on_before_browser),
                get_active_browser: Some(Self::get_active_browser),
                can_access_browser: Some(Self::can_access_browser),
                get_extension_resource: Some(Self::get_extension_resource),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for ExtensionHandlerWrapper: cef_extension_handler_t {
        fn on_extension_load_failed(
            &self,
            result: ErrorCode: cef_errorcode_t::Type
        ) {
            unsafe{ self.delegate.get() }.on_extension_load_failed(result);
        }
        fn on_extension_loaded(
            &self,
            extension: Extension: *mut cef_extension_t
        ) {
            unsafe{ self.delegate.get() }.on_extension_loaded(extension);
        }
        fn on_extension_unloaded(
            &self,
            extension: Extension: *mut cef_extension_t
        ) {
            unsafe{ self.delegate.get() }.on_extension_unloaded(extension);
        }
        fn on_before_background_browser(
            &self,
            extension: Extension: *mut cef_extension_t,
            url: &CefString: *const cef_string_t,
            client: &mut Client: *mut *mut cef_client_t,
            settings: &mut cef_browser_settings_t: *mut cef_browser_settings_t
        ) -> c_int {
            let mut settings_rs = unsafe{ BrowserSettings::from_raw(settings) };
            let ret = unsafe{ self.delegate.get() }.on_before_background_browser(
                extension,
                &String::from(url),
                client,
                &mut settings_rs
            ) as c_int;
            *settings = settings_rs.into_raw();
            ret
        }
        fn on_before_browser(
            &self,
            extension: Extension: *mut cef_extension_t,
            browser: Browser: *mut cef_browser_t,
            active_browser: Browser: *mut cef_browser_t,
            index: c_int: c_int,
            url: &CefString: *const cef_string_t,
            active: c_int: c_int,
            window_info: &mut cef_window_info_t: *mut cef_window_info_t,
            client: &mut Client: *mut *mut cef_client_t,
            settings: &mut cef_browser_settings_t: *mut cef_browser_settings_t
        ) -> c_int {
            let mut window_info_rs = unsafe{ WindowInfo::from_raw(window_info) };
            let mut settings_rs = unsafe{ BrowserSettings::from_raw(settings) };
            let ret = unsafe{ self.delegate.get() }.on_before_browser(
                extension,
                browser,
                active_browser,
                index as usize,
                &String::from(url),
                active != 0,
                &mut window_info_rs,
                client,
                &mut settings_rs
            ) as c_int;
            *window_info = window_info_rs.into_raw();
            *settings = settings_rs.into_raw();
            ret
        }
        fn get_active_browser(
            &self,
            extension: Extension: *mut cef_extension_t,
            browser: Browser: *mut cef_browser_t,
            include_incognito: c_int: c_int
        ) -> *mut cef_browser_t {
            unsafe{ self.delegate.get() }.get_active_browser(extension, browser, include_incognito != 0).into_raw()
        }
        fn can_access_browser(
            &self,
            extension: Extension: *mut cef_extension_t,
            browser: Browser: *mut cef_browser_t,
            include_incognito: c_int: c_int,
            target_browser: Browser: *mut cef_browser_t
        ) -> c_int {
            unsafe{ self.delegate.get() }.can_access_browser(extension, browser, include_incognito != 0, target_browser) as c_int
        }
        fn get_extension_resource(
            &self,
            extension: Extension: *mut cef_extension_t,
            browser: Browser: *mut cef_browser_t,
            file: &CefString: *const cef_string_t,
            callback: GetExtensionResourceCallback: *mut cef_get_extension_resource_callback_t
        ) -> c_int {
            unsafe{ self.delegate.get() }.get_extension_resource(extension, browser, &String::from(file), callback) as c_int
        }
    }
}
