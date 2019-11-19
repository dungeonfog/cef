use std::net::IpAddr;
use crate::string::CefStringList;
use parking_lot::Mutex;
use std::mem::ManuallyDrop;
use std::os::raw::c_int;
use crate::load_handler::ErrorCode;
use crate::scheme::SchemeHandlerFactory;
use cef_sys::cef_resolve_callback_t;
use crate::cookie::CookieManager;
use crate::values::{StoredValue, DictionaryValue, Value};
use cef_sys::cef_string_list_t;
use crate::extension::ExtensionHandler;
use crate::extension::Extension;
use crate::callback::CompletionCallback;
use cef_sys::{
    cef_browser_t, cef_create_context_shared, cef_frame_t, cef_plugin_policy_t,
    cef_request_context_create_context, cef_request_context_get_global_context,
    cef_request_context_handler_t, cef_request_context_settings_t, cef_request_context_t,
    cef_request_t, cef_resource_request_handler_t, cef_string_t, cef_string_utf8_to_utf16,
    cef_web_plugin_info_t, cef_errorcode_t,
};

use std::{
    path::Path,
    ptr::{null, null_mut},
    convert::TryFrom,
};

use crate::{
    browser::Browser,
    frame::Frame,
    refcounted::{RefCountedPtr, Wrapper},
    request::Request,
    resource_request_handler::{ResourceRequestHandler},
    string::CefString,
    web_plugin::WebPluginInfo,
};

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PluginPolicy {
    Allow = cef_plugin_policy_t::PLUGIN_POLICY_ALLOW as isize,
    DetectImportant = cef_plugin_policy_t::PLUGIN_POLICY_DETECT_IMPORTANT as isize,
    Block = cef_plugin_policy_t::PLUGIN_POLICY_BLOCK as isize,
    Disable = cef_plugin_policy_t::PLUGIN_POLICY_DISABLE as isize,
}

impl PluginPolicy {
    pub unsafe fn from_unchecked(c: crate::CEnumType) -> Self {
        std::mem::transmute(c)
    }
}

ref_counted_ptr!{
    pub struct RequestContextHandler(*mut cef_request_context_handler_t);
}

impl RequestContextHandler {
    pub fn new<C: RequestContextHandlerCallbacks>(callbacks: C) -> RequestContextHandler {
        unsafe{ RequestContextHandler::from_ptr_unchecked(RequestContextHandlerWrapper::new(Box::new(callbacks)).wrap().into_raw()) }
    }
}

/// Implement this structure to provide handler implementations. The handler
/// instance will not be released until all objects related to the context have
/// been destroyed.
pub trait RequestContextHandlerCallbacks: 'static + Send + Sync {
    /// Called on the browser process UI thread immediately after the request
    /// context has been initialized.
    fn on_request_context_initialized(&self, request_context: RequestContext) {}
    /// Called on multiple browser process threads before a plugin instance is
    /// loaded. `mime_type` is the mime type of the plugin that will be loaded.
    /// `plugin_url` is the content URL that the plugin will load and may be None.
    /// `is_main_frame` will be true if the plugin is being loaded in the main
    /// (top-level) frame, `top_origin_url` is the URL for the top-level frame that
    /// contains the plugin when loading a specific plugin instance or None when
    /// building the initial list of enabled plugins for 'navigator.plugins'
    /// JavaScript state. `plugin_info` includes additional information about the
    /// plugin that will be loaded. `plugin_policy` is the recommended policy.
    /// Return Some([PluginPolicy]) to change the policy. Return
    /// None to use the recommended policy. The default plugin policy can be
    /// set at runtime using the `--plugin-policy=[allow`detect`block]` command-
    /// line flag. Decisions to mark a plugin as disabled by setting
    /// `plugin_policy` to PLUGIN_POLICY_DISABLED may be cached when
    /// `top_origin_url` is None. To purge the plugin list cache and potentially
    /// trigger new calls to this function call
    /// [RequestContext::purge_plugin_list_cache].
    fn on_before_plugin_load(
        &self,
        mime_type: &str,
        plugin_url: Option<&str>,
        is_main_frame: bool,
        top_origin_url: Option<&str>,
        plugin_info: &WebPluginInfo,
        plugin_policy: PluginPolicy,
    ) -> Option<PluginPolicy> {
        None
    }
    /// Called on the browser process IO thread before a resource request is
    /// initiated. The `browser` and `frame` values represent the source of the
    /// request, and may be None for requests originating from service workers or
    /// [URLRequest]. `request` represents the request contents and cannot be
    /// modified in this callback. `is_navigation` will be true if the resource
    /// request is a navigation. `is_download` will be true if the resource
    /// request is a download. `request_initiator` is the origin (scheme + domain)
    /// of the page that initiated the request. Set `disable_default_handling` to
    /// true to disable default handling of the request, in which case it will
    /// need to be handled via [ResourceRequestHandlerCallbacks::get_resource_handler]
    /// or it will be canceled. To allow the resource load to proceed with default
    /// handling return None. To specify a handler for the resource return a
    /// [ResourceRequestHandlerCallbacks] object. This function will not be called if
    /// the client associated with `browser` returns a non-None value from
    /// [RequestHandlerCallbacks::get_resource_request_handler] for the same request
    /// (identified by [Request::get_identifier]).
    fn get_resource_request_handler(
        &self,
        browser: Option<Browser>,
        frame: Option<Frame>,
        request: Request,
        is_navigation: bool,
        is_download: bool,
        request_initiator: &str,
        disable_default_handling: &mut bool,
    ) -> Option<ResourceRequestHandler> {
        None
    }
}

pub(crate) struct RequestContextHandlerWrapper(Box<dyn RequestContextHandlerCallbacks>);

impl Wrapper for RequestContextHandlerWrapper {
    type Cef = cef_request_context_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_request_context_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_request_context_initialized: Some(Self::request_context_initialized),
                on_before_plugin_load: Some(Self::before_plugin_load),
                get_resource_request_handler: Some(Self::get_resource_request_handler),
            },
            self,
        )
    }
}

impl RequestContextHandlerWrapper {
    pub(crate) fn new(delegate: Box<dyn RequestContextHandlerCallbacks>) -> RequestContextHandlerWrapper {
        Self(delegate)
    }
}
cef_callback_impl! {
    impl for RequestContextHandlerWrapper: cef_request_context_handler_t {
        fn request_context_initialized(
            &self,
            request_context: RequestContext: *mut cef_request_context_t,
        ) {
            self.0
                .on_request_context_initialized(request_context);
        }
        fn before_plugin_load(
            &self,
            mime_type: &CefString: *const cef_string_t,
            plugin_url: Option<&CefString>: *const cef_string_t,
            is_main_frame: bool: std::os::raw::c_int,
            top_origin_url: Option<&CefString>: *const cef_string_t,
            plugin_info: WebPluginInfo: *mut cef_web_plugin_info_t,
            plugin_policy: &mut PluginPolicy: *mut cef_plugin_policy_t::Type,
        ) -> std::os::raw::c_int {
            if let Some(policy) = self.0.on_before_plugin_load(
                &String::from(mime_type),
                plugin_url
                    .map(String::from)
                    .as_ref()
                    .map(|s| &**s),
                is_main_frame,
                top_origin_url
                    .map(String::from)
                    .as_ref()
                    .map(|s| &**s),
                &plugin_info,
                *plugin_policy,
            ) {
                *plugin_policy = policy;
                1
            } else {
                0
            }
        }
        fn get_resource_request_handler(
            &self,
            browser: Option<Browser>: *mut cef_browser_t,
            frame: Option<Frame>: *mut cef_frame_t,
            request: Request: *mut cef_request_t,
            is_navigation: bool: std::os::raw::c_int,
            is_download: bool: std::os::raw::c_int,
            request_initiator: &CefString: *const cef_string_t,
            disable_default_handling: &mut std::os::raw::c_int: *mut std::os::raw::c_int,
        ) -> *mut cef_resource_request_handler_t {
            let mut local_disable_default_handling = *disable_default_handling != 0;
            let ret = self.0.get_resource_request_handler(
                browser,
                frame,
                request,
                is_navigation,
                is_download,
                &String::from(request_initiator),
                &mut local_disable_default_handling,
            ).map(|cef| cef.into_raw()).unwrap_or(null_mut());
            *disable_default_handling = local_disable_default_handling as std::os::raw::c_int;
            ret
        }
    }
}

ref_counted_ptr! {
    /// A request context provides request handling for a set of related browser or
    /// URL request objects. A request context can be specified when creating a new
    /// browser via the [BrowserHost] static factory functions or when creating
    /// a new URL request via the [URLRequest] static factory functions. Browser
    /// objects with different request contexts will never be hosted in the same
    /// render process. Browser objects with the same request context may or may not
    /// be hosted in the same render process depending on the process model. Browser
    /// objects created indirectly via the JavaScript window.open function or
    /// targeted links will share the same render process and the same request
    /// context as the source browser. When running in single-process mode there is
    /// only a single render process (the main process) and so all browsers created
    /// in single-process mode will share the same request context. This will be the
    /// first request context passed into a [BrowserHost] static factory
    /// function and all other request context objects will be ignored.
    pub struct RequestContext(*mut cef_request_context_t);
}

ref_counted_ptr!{
    struct ResolveCallback(*mut cef_resolve_callback_t);
}

struct ResolveCallbackWrapper(Mutex<Option<Box<dyn 'static + Send + FnOnce(ErrorCode, &[IpAddr])>>>);

impl ResolveCallback {
    fn new(callback: impl 'static + Send + FnOnce(ErrorCode, &[IpAddr])) -> ResolveCallback {
        unsafe{ ResolveCallback::from_ptr_unchecked(ResolveCallbackWrapper(Mutex::new(Some(Box::new(callback)))).wrap().into_raw()) }
    }
}

impl Wrapper for ResolveCallbackWrapper {
    type Cef = cef_resolve_callback_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_resolve_callback_t {
                base: unsafe { std::mem::zeroed() },
                on_resolve_completed: Some(Self::on_resolve_completed),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for ResolveCallbackWrapper: cef_resolve_callback_t {
        fn on_resolve_completed(
            &self,
            result: ErrorCode: cef_errorcode_t::Type,
            resolved_ips: ManuallyDrop<CefStringList>: cef_string_list_t,
        ) {
            let resolved_ips = (&*resolved_ips).into_iter().map(String::from).filter_map(|s| s.parse().ok()).collect::<Vec<IpAddr>>();
            self.0.lock().take().unwrap()(
                result,
                &resolved_ips,
            );
        }
    }
}


impl RequestContext {
    /// Returns the global context object.
    pub fn global() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_request_context_get_global_context()) }
    }
    /// Creates a new context object that shares storage with `other` and uses an
    /// optional `handler`.
    pub fn new_shared(
        other: RequestContext,
        handler: Option<Box<dyn RequestContextHandlerCallbacks>>,
    ) -> Self {
        let handler_ptr = if let Some(handler) = handler {
            RequestContextHandlerWrapper::new(handler).wrap().into_raw()
        } else {
            null_mut()
        };
        unsafe {
            Self::from_ptr_unchecked(cef_create_context_shared(other.into_raw(), handler_ptr))
        }
    }

    /// Returns `true` if this object is pointing to the same context as `that`
    /// object.
    pub fn is_same(&self, other: RequestContext) -> bool {
        unsafe{ self.0.is_same.unwrap()(self.as_ptr(), other.into_raw()) != 0 }
    }
    /// Returns `true` if this object is sharing the same storage as `that`
    /// object.
    pub fn is_sharing_with(&self, other: RequestContext) -> bool {
        unsafe{ self.0.is_sharing_with.unwrap()(self.as_ptr(), other.into_raw()) != 0 }
    }
    /// Returns `true` if this object is the global context. The global context
    /// is used by default when creating a browser or URL request with a `None`
    /// context argument.
    pub fn is_global(&self) -> bool {
        unsafe{ self.0.is_global.unwrap()(self.as_ptr()) != 0 }
    }
    /// Returns the handler for this context if any.
    pub fn get_handler(&self) -> Option<RequestContextHandler> {
        unsafe{ RequestContextHandler::from_ptr(self.0.get_handler.unwrap()(self.as_ptr())) }
    }
    /// Returns the cache path for this object. If `None` an "incognito mode" in-
    /// memory cache is being used.
    pub fn get_cache_path(&self) -> Option<String> {
        unsafe{ CefString::from_userfree(self.0.get_cache_path.unwrap()(self.as_ptr())).map(String::from) }
    }
    /// Returns the cookie manager for this object. `callback`
    /// will be executed asnychronously on the IO thread after the manager's
    /// storage has been initialized.
    pub fn get_cookie_manager(&self, callback: impl 'static + Send + FnOnce()) -> CookieManager {
        unsafe{ CookieManager::from_ptr_unchecked(self.0.get_cookie_manager.unwrap()(self.as_ptr(), CompletionCallback::new(callback).into_raw())) }
    }
    /// Register a scheme handler factory for the specified `scheme_name` and
    /// optional `domain_name`. An `None` `domain_name` value for a standard scheme
    /// will cause the factory to match all domain names. The `domain_name` value
    /// will be ignored for non-standard schemes. If `scheme_name` is a built-in
    /// scheme and no handler is returned by `factory` then the built-in scheme
    /// handler factory will be called. If `scheme_name` is a custom scheme then
    /// you must also implement the cef_app_t::on_register_custom_schemes()
    /// function in all processes. This function may be called multiple times to
    /// change or remove the factory that matches the specified `scheme_name` and
    /// optional `domain_name`. Returns `false` if an error occurs. This function
    /// may be called on any thread in the browser process.
    pub fn register_scheme_handler_factory(&self, scheme_name: &str, domain_name: Option<&str>, factory: SchemeHandlerFactory) -> bool {
        unsafe{
            self.0.register_scheme_handler_factory.unwrap()(
                self.as_ptr(),
                CefString::new(scheme_name).as_ptr(),
                domain_name.map(CefString::new).as_ref().map(CefString::as_ptr).unwrap_or(null()),
                factory.into_raw()
            ) != 0
        }
    }
    /// Clear all registered scheme handler factories. Returns `false` on error.
    /// This function may be called on any thread in the browser process.
    pub fn clear_scheme_handler_factories(&self) -> bool {
        unsafe{ self.0.clear_scheme_handler_factories.unwrap()(self.as_ptr()) != 0 }
    }
    /// Tells all renderer processes associated with this context to throw away
    /// their plugin list cache. If `reload_pages` is `true` they will also
    /// reload all pages with plugins.
    /// cef_request_tContextHandler::OnBeforePluginLoad may be called to rebuild
    /// the plugin list cache.
    pub fn purge_plugin_list_cache(&self, reload_pages: bool) {
        unsafe{ self.0.purge_plugin_list_cache.unwrap()(self.as_ptr(), reload_pages as c_int) }
    }
    /// Returns `true` if a preference with the specified `name` exists. This
    /// function must be called on the browser process UI thread.
    pub fn has_preference(&self, name: &str) -> bool {
        unsafe{
            self.0.has_preference.unwrap()(
                self.as_ptr(),
                CefString::new(name).as_ptr(),
            ) != 0
        }
    }
    /// Returns the value for the preference with the specified `name`. Returns
    /// `None` if the preference does not exist. The returned object contains a copy
    /// of the underlying preference value and modifications to the returned object
    /// will not modify the underlying preference value. This function must be
    /// called on the browser process UI thread.
    pub fn get_preference(&self, name: &str) -> Option<StoredValue> {
        unsafe{
            Value::from_ptr(self.0.get_preference.unwrap()(
                self.as_ptr(),
                CefString::new(name).as_ptr(),
            )).map(StoredValue::from)
        }
    }
    /// Returns all preferences as a dictionary. If `include_defaults` is `true`
    /// then preferences currently at their default value will be included. The
    /// returned object contains a copy of the underlying preference values and
    /// modifications to the returned object will not modify the underlying
    /// preference values. This function must be called on the browser process UI
    /// thread.
    pub fn get_all_preferences(&self, include_defaults: bool) -> DictionaryValue {
        unsafe{
            DictionaryValue::from_ptr_unchecked(self.0.get_all_preferences.unwrap()(
                self.as_ptr(),
                include_defaults as c_int,
            ))
        }
    }
    /// Returns `true` if the preference with the specified `name` can be
    /// modified using SetPreference. As one example preferences set via the
    /// command-line usually cannot be modified. This function must be called on
    /// the browser process UI thread.
    pub fn can_set_preference(&self, name: &str) -> bool {
        unsafe{
            self.0.can_set_preference.unwrap()(
                self.as_ptr(),
                CefString::new(name).as_ptr(),
            ) != 0
        }
    }
    /// Set the `value` associated with preference `name`. Returns `true` if the
    /// value is set successfully and `false` otherwise. If `value` is `None` the
    /// preference will be restored to its default value. If setting the preference
    /// fails then `error` will be populated with a detailed description of the
    /// problem. This function must be called on the browser process UI thread.
    pub fn set_preference(&self, name: &str, value: Option<StoredValue>) -> Result<(), String> {
        let mut error = CefString::null();
        let success = unsafe {
            self.0.set_preference.unwrap()(
                self.as_ptr(),
                CefString::new(name).as_ptr(),
                value.map(|v| Value::try_from(v).unwrap().into_raw()).unwrap_or(null_mut()),
                error.as_ptr_mut(),
            ) != 0
        };
        if success {
            Ok(())
        } else {
            Err(String::from(error))
        }
    }
    /// Clears all certificate exceptions that were added as part of handling
    /// cef_request_tHandler::on_certificate_error(). If you call this it is
    /// recommended that you also call close_all_connections() or you risk not
    /// being prompted again for server certificates if you reconnect quickly.
    /// `callback` will be executed on the UI thread after completion.
    pub fn clear_certificate_exceptions(&self, callback: impl 'static + Send + FnOnce()) {
        unsafe {
            self.0.clear_certificate_exceptions.unwrap()(
                self.as_ptr(),
                CompletionCallback::new(callback).into_raw(),
            );
        }
    }
    /// Clears all HTTP authentication credentials that were added as part of
    /// handling GetAuthCredentials. `callback` will be executed
    /// on the UI thread after completion.
    pub fn clear_http_auth_credentials(&self, callback: impl 'static + Send + FnOnce()) {
        unsafe {
            self.0.clear_http_auth_credentials.unwrap()(
                self.as_ptr(),
                CompletionCallback::new(callback).into_raw(),
            );
        }
    }
    /// Clears all active and idle connections that Chromium currently has. This is
    /// only recommended if you have released all other CEF objects but don't yet
    /// want to call Cefshutdown(). `callback` will be executed
    /// on the UI thread after completion.
    pub fn close_all_connections(&self, callback: impl 'static + Send + FnOnce()) {
        unsafe {
            self.0.close_all_connections.unwrap()(
                self.as_ptr(),
                CompletionCallback::new(callback).into_raw(),
            );
        }
    }
    /// Attempts to resolve `origin` to a list of associated IP addresses.
    /// `callback` will be executed on the UI thread after completion.
    pub fn resolve_host(&self, origin: &str, callback: impl 'static + Send + FnOnce(ErrorCode, &[IpAddr])) {
        unsafe {
            self.0.resolve_host.unwrap()(
                self.as_ptr(),
                CefString::new(origin).as_ptr(),
                ResolveCallback::new(callback).into_raw()
            )
        }
    }
    /// Load an extension.
    ///
    /// If extension resources will be read from disk using the default load
    /// implementation then `root_directory` should be the absolute path to the
    /// extension resources directory and `manifest` should be `None`. If extension
    /// resources will be provided by the client (e.g. via cef_request_tHandler
    /// and/or cef_extension_tHandler) then `root_directory` should be a path
    /// component unique to the extension (if not absolute this will be internally
    /// prefixed with the PK_DIR_RESOURCES path) and `manifest` should contain the
    /// contents that would otherwise be read from the "manifest.json" file on
    /// disk.
    ///
    /// The loaded extension will be accessible in all contexts sharing the same
    /// storage (HasExtension returns `true`). However, only the context on which
    /// this function was called is considered the loader (DidLoadExtension returns
    /// `true`) and only the loader will receive cef_request_tContextHandler
    /// callbacks for the extension.
    ///
    /// cef_extension_tHandler::OnExtensionLoaded will be called on load success or
    /// cef_extension_tHandler::OnExtensionLoadFailed will be called on load
    /// failure.
    ///
    /// If the extension specifies a background script via the "background"
    /// manifest key then cef_extension_tHandler::OnBeforeBackgroundBrowser will be
    /// called to create the background browser. See that function for additional
    /// information about background scripts.
    ///
    /// For visible extension views the client application should evaluate the
    /// manifest to determine the correct extension URL to load and then pass that
    /// URL to the cef_browser_host_t::CreateBrowser* function after the extension
    /// has loaded. For example, the client can look for the "browser_action"
    /// manifest key as documented at
    /// https://developer.chrome.com/extensions/browserAction. Extension URLs take
    /// the form "chrome-extension://<extension_id>/<path>".
    ///
    /// Browsers that host extensions differ from normal browsers as follows:
    ///  - Can access chrome.* JavaScript APIs if allowed by the manifest. Visit
    ///    chrome://extensions-support for the list of extension APIs currently
    ///    supported by CEF.
    ///  - Main frame navigation to non-extension content is blocked.
    ///  - Pinch-zooming is disabled.
    ///  - CefBrowserHost::GetExtension returns the hosted extension.
    ///  - CefBrowserHost::IsBackgroundHost returns true for background hosts.
    ///
    /// See https://developer.chrome.com/extensions for extension implementation
    /// and usage documentation.
    pub fn load_extension(&self, root_directory: &str, manifest: Option<DictionaryValue>, handler: ExtensionHandler) {
        unsafe {
            self.0.load_extension.unwrap()(
                self.as_ptr(),
                CefString::new(root_directory).as_ptr(),
                manifest.map(|m| m.into_raw()).unwrap_or(null_mut()),
                handler.into_raw(),
            )
        }
    }
    /// Returns `true` if this context was used to load the extension identified
    /// by `extension_id`. Other contexts sharing the same storage will also have
    /// access to the extension (see HasExtension). This function must be called on
    /// the browser process UI thread.
    pub fn did_load_extension(&self, extension_id: &str) -> bool {
        unsafe{
            self.0.did_load_extension.unwrap()(
                self.as_ptr(),
                CefString::new(extension_id).as_ptr(),
            ) != 0
        }
    }
    /// Returns `true` if this context has access to the extension identified by
    /// `extension_id`. This may not be the context that was used to load the
    /// extension (see DidLoadExtension). This function must be called on the
    /// browser process UI thread.
    pub fn has_extension(&self, extension_id: &str) -> bool {
        unsafe{
            self.0.has_extension.unwrap()(
                self.as_ptr(),
                CefString::new(extension_id).as_ptr(),
            ) != 0
        }
    }
    /// Retrieve the list of all extensions that this context has access to (see
    /// HasExtension). `extension_ids` will be populated with the list of extension
    /// ID values. Returns `true` on success. This function must be called on the
    /// browser process UI thread.
    pub fn get_extensions(&self) -> Option<Vec<String>> {
        unsafe {
            let mut string_list = CefStringList::new();
            let success = self.0.get_extensions.unwrap()(
                self.as_ptr(),
                string_list.as_mut_ptr()
            ) != 0;
            if success {
                Some(string_list.into_iter().map(String::from).collect())
            } else {
                None
            }
        }
    }
    /// Returns the extension matching `extension_id` or `None` if no matching
    /// extension is accessible in this context (see HasExtension). This function
    /// must be called on the browser process UI thread.
    pub fn get_extension(&self, extension_id: &str) -> Option<Extension> {
        unsafe{
            Extension::from_ptr(self.0.get_extension.unwrap()(
                self.as_ptr(),
                CefString::new(extension_id).as_ptr(),
            ))
        }
    }
}

/// Request context initialization settings.
pub struct RequestContextBuilder(
    Option<cef_request_context_settings_t>,
    Option<RequestContextHandler>,
);

impl RequestContextBuilder {
    pub fn new() -> Self {
        Self(None, None)
    }
    /// Creates a new context object with the specified `settings` and optional
    /// `handler`.
    pub fn build(self) -> RequestContext {
        let settings_ptr = self
            .0
            .map(|settings| &settings as *const _)
            .unwrap_or_else(null);
        let handler_ptr = if let Some(handler) = self.1 {
            handler.into_raw()
        } else {
            null()
        };
        unsafe {
            RequestContext::from_ptr_unchecked(cef_request_context_create_context(
                settings_ptr,
                handler_ptr as *mut _,
            ))
        }
    }

    fn get_settings(&mut self) -> &mut cef_request_context_settings_t {
        self.0
            .get_or_insert_with(|| cef_request_context_settings_t {
                size: std::mem::size_of::<cef_request_context_settings_t>(),
                cache_path: unsafe { std::mem::zeroed() },
                persist_session_cookies: 0,
                persist_user_preferences: 0,
                ignore_certificate_errors: 0,
                enable_net_security_expiration: 0,
                accept_language_list: unsafe { std::mem::zeroed() },
            })
    }

    /// Optionally supply a handler to the request context. See [RequestContextHandlerCallbacks].
    pub fn with_handler(mut self, handler: RequestContextHandler) -> Self {
        self.1.replace(handler);
        self
    }

    /// The location where cache data for this request context will be stored on
    /// disk. If non-empty this must be either equal to or a child directory of
    /// [CefSettings::root_cache_path]. If empty then browsers will be created in
    /// "incognito mode" where in-memory caches are used for storage and no data is
    /// persisted to disk. HTML5 databases such as localStorage will only persist
    /// across sessions if a cache path is specified. To share the global browser
    /// cache and related configuration set this value to match the
    /// [CefSettings::cache_path] value.
    pub fn with_cache_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path = path.as_ref().to_str().expect("Invalid UTF8");
        let settings = self.get_settings();
        let len = path.len();
        unsafe {
            cef_string_utf8_to_utf16(
                path.as_ptr() as *const std::os::raw::c_char,
                len,
                &mut settings.cache_path,
            );
        }
        self
    }

    /// To persist session cookies (cookies without an expiry date or validity
    /// interval) by default when using the global cookie manager set this value to
    /// true. Session cookies are generally intended to be transient and most
    /// Web browsers do not persist them. Can be set globally using the
    /// [CefSettings::persist_session_cookies] value. This value will be ignored if
    /// `cache_path` is empty or if it matches the [CefSettings::cache_path] value.
    pub fn persist_session_cookies(mut self, flag: bool) -> Self {
        let settings = self.get_settings();
        settings.persist_session_cookies = flag as i32;
        self
    }

    /// To persist user preferences as a JSON file in the cache path directory set
    /// this value to true. Can be set globally using the
    /// [CefSettings::persist_user_preferences] value. This value will be ignored if
    /// `cache_path` is empty or if it matches the [CefSettings::cache_path] value.
    pub fn persist_user_preferences(mut self, flag: bool) -> Self {
        let settings = self.get_settings();
        settings.persist_user_preferences = flag as i32;
        self
    }

    /// Set to true to ignore errors related to invalid SSL certificates.
    /// Enabling this setting can lead to potential security vulnerabilities like
    /// "man in the middle" attacks. Applications that load content from the
    /// internet should not enable this setting. Can be set globally using the
    /// [CefSettings::ignore_certificate_errors] value. This value will be ignored if
    /// `cache_path` matches the [CefSettings::cache_path] value.
    pub fn ignore_certificate_errors(mut self, flag: bool) -> Self {
        let settings = self.get_settings();
        settings.ignore_certificate_errors = flag as i32;
        self
    }

    /// Set to true to enable date-based expiration of built in network
    /// security information (i.e. certificate transparency logs, HSTS preloading
    /// and pinning information). Enabling this option improves network security
    /// but may cause HTTPS load failures when using CEF binaries built more than
    /// 10 weeks in the past. See https://www.certificate-transparency.org/ and
    /// https://www.chromium.org/hsts for details. Can be set globally using the
    /// [CefSettings::enable_net_security_expiration] value.
    pub fn enable_net_security_expiration(mut self, flag: bool) -> Self {
        let settings = self.get_settings();
        settings.enable_net_security_expiration = flag as i32;
        self
    }

    /// Comma delimited ordered list of language codes without any whitespace that
    /// will be used in the "Accept-Language" HTTP header. Can be set globally
    /// using the [CefSettings::accept_language_list] value or overridden on a per-
    /// browser basis using the [BrowserSettings::accept_language_list] value. If
    /// all values are empty then "en-US,en" will be used. This value will be
    /// ignored if `cache_path` matches the [CefSettings::cache_path] value.
    pub fn accept_language_list(mut self, list: &str) -> Self {
        let settings = self.get_settings();
        let len = list.len();
        unsafe {
            cef_string_utf8_to_utf16(
                list.as_ptr() as *const std::os::raw::c_char,
                len,
                &mut settings.accept_language_list,
            );
        }
        self
    }
}

impl Default for RequestContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
