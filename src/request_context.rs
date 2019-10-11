use cef_sys::{
    cef_base_ref_counted_t, cef_browser_t, cef_create_context_shared, cef_frame_t,
    cef_plugin_policy_t, cef_request_context_create_context,
    cef_request_context_get_global_context, cef_request_context_handler_t,
    cef_request_context_settings_t, cef_request_context_t, cef_request_t,
    cef_resource_request_handler_t, cef_string_t, cef_string_utf8_to_utf16, cef_web_plugin_info_t,
};

use num_enum::UnsafeFromPrimitive;
use std::ptr::{null, null_mut};

use crate::{
    browser::Browser,
    frame::Frame,
    refcounted::{RefCounted, RefCounter},
    request::Request,
    resource_request_handler::{ResourceRequestHandler, ResourceRequestHandlerWrapper},
    string::CefString,
    web_plugin::WebPluginInfo,
};

#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum PluginPolicy {
    Allow = cef_plugin_policy_t::PLUGIN_POLICY_ALLOW as i32,
    DetectImportant = cef_plugin_policy_t::PLUGIN_POLICY_DETECT_IMPORTANT as i32,
    Block = cef_plugin_policy_t::PLUGIN_POLICY_BLOCK as i32,
    Disable = cef_plugin_policy_t::PLUGIN_POLICY_DISABLE as i32,
}

/// Implement this structure to provide handler implementations. The handler
/// instance will not be released until all objects related to the context have
/// been destroyed.
pub trait RequestContextHandler: Send + Sync {
    /// Called on the browser process UI thread immediately after the request
    /// context has been initialized.
    fn on_request_context_initialized(&self, request_context: &RequestContext) {}
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
    /// set at runtime using the `--plugin-policy=[allow|detect|block]` command-
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
    /// need to be handled via [ResourceRequestHandler::get_resource_handler]
    /// or it will be canceled. To allow the resource load to proceed with default
    /// handling return None. To specify a handler for the resource return a
    /// [ResourceRequestHandler] object. This function will not be called if
    /// the client associated with `browser` returns a non-None value from
    /// [RequestHandler::get_resource_request_handler] for the same request
    /// (identified by [Request::get_identifier]).
    fn get_resource_request_handler(
        &self,
        browser: Option<&Browser>,
        frame: Option<&Frame>,
        request: &Request,
        is_navigation: bool,
        is_download: bool,
        request_initiator: &str,
        disable_default_handling: &mut bool,
    ) -> Option<Box<dyn ResourceRequestHandler>> {
        None
    }
}

pub(crate) struct RequestContextHandlerWrapper {
    delegate: Box<dyn RequestContextHandler>,
    resource_request_handler: Option<*mut cef_resource_request_handler_t>,
}

impl RequestContextHandlerWrapper {
    pub(crate) fn wrap(
        delegate: Box<dyn RequestContextHandler>,
    ) -> *mut cef_request_context_handler_t {
        let mut rc = RefCounted::new(
            cef_request_context_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_request_context_initialized: Some(Self::request_context_initialized),
                on_before_plugin_load: Some(Self::before_plugin_load),
                get_resource_request_handler: Some(Self::get_resource_request_handler),
            },
            Self {
                delegate,
                resource_request_handler: None,
            },
        );
        unsafe { &mut *rc }.get_cef()
    }
    extern "C" fn request_context_initialized(
        self_: *mut cef_request_context_handler_t,
        request_context: *mut cef_request_context_t,
    ) {
        let this = unsafe { RefCounted::<cef_request_context_handler_t>::make_temp(self_) };
        this.delegate
            .on_request_context_initialized(&RequestContext(request_context));
    }
    extern "C" fn before_plugin_load(
        self_: *mut cef_request_context_handler_t,
        mime_type: *const cef_string_t,
        plugin_url: *const cef_string_t,
        is_main_frame: std::os::raw::c_int,
        top_origin_url: *const cef_string_t,
        plugin_info: *mut cef_web_plugin_info_t,
        plugin_policy: *mut cef_plugin_policy_t::Type,
    ) -> std::os::raw::c_int {
        let this = unsafe { RefCounted::<cef_request_context_handler_t>::make_temp(self_) };
        if let Some(policy) = this.delegate.on_before_plugin_load(
            &CefString::copy_raw_to_string(mime_type).unwrap(),
            CefString::copy_raw_to_string(plugin_url)
                .as_ref()
                .and_then(|s| Some(s.as_str())),
            is_main_frame != 0,
            CefString::copy_raw_to_string(top_origin_url)
                .as_ref()
                .and_then(|s| Some(s.as_str())),
            &WebPluginInfo::from(plugin_info),
            unsafe { PluginPolicy::from_unchecked(*plugin_policy) },
        ) {
            unsafe { (*plugin_policy) = policy as cef_plugin_policy_t::Type };
            1
        } else {
            0
        }
    }
    extern "C" fn get_resource_request_handler(
        self_: *mut cef_request_context_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        request: *mut cef_request_t,
        is_navigation: std::os::raw::c_int,
        is_download: std::os::raw::c_int,
        request_initiator: *const cef_string_t,
        disable_default_handling: *mut std::os::raw::c_int,
    ) -> *mut cef_resource_request_handler_t {
        let mut this = unsafe { RefCounted::<cef_request_context_handler_t>::make_temp(self_) };
        let mut local_disable_default_handling = false;
        if let Some(resource_request_handler) = this.delegate.get_resource_request_handler(
            unsafe { Browser::from_ptr(browser).as_ref() },
            unsafe { Frame::from_ptr(frame).as_ref() },
            unsafe { &Request::from_ptr_unchecked(request) },
            is_navigation != 0,
            is_download != 0,
            CefString::copy_raw_to_string(request_initiator)
                .as_ref()
                .and_then(|s| Some(s.as_str()))
                .unwrap_or(""),
            &mut local_disable_default_handling,
        ) {
            if local_disable_default_handling {
                unsafe { (*disable_default_handling) = 1 };
            }
            if let Some(resource_request_handler) =
                this.resource_request_handler
                    .replace(ResourceRequestHandlerWrapper::wrap(
                        resource_request_handler,
                    ))
            {
                RefCounted::<cef_resource_request_handler_t>::release(
                    resource_request_handler as *mut cef_base_ref_counted_t,
                );
            }
            this.resource_request_handler.unwrap()
        } else {
            if local_disable_default_handling {
                unsafe { (*disable_default_handling) = 1 };
            }
            if let Some(resource_request_handler) = this.resource_request_handler.take() {
                RefCounted::<cef_resource_request_handler_t>::release(
                    resource_request_handler as *mut cef_base_ref_counted_t,
                );
            }
            null_mut() as *mut cef_resource_request_handler_t
        }
    }
}

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

impl RequestContext {
    /// Returns the global context object.
    pub fn global() -> Self {
        Self(unsafe { cef_request_context_get_global_context() })
    }
    /// Creates a new context object that shares storage with `other` and uses an
    /// optional `handler`.
    pub fn new_shared(
        other: &RequestContext,
        handler: Option<Box<dyn RequestContextHandler>>,
    ) -> Self {
        let handler_ptr = if let Some(handler) = handler {
            RequestContextHandlerWrapper::wrap(handler)
        } else {
            null_mut()
        };
        Self(unsafe { cef_create_context_shared(other.0, handler_ptr) })
    }

    pub(crate) fn as_ptr(&self) -> *mut cef_request_context_t {
        self.0
    }
}

/// Request context initialization settings.
pub struct RequestContextBuilder(
    Option<cef_request_context_settings_t>,
    Option<Box<dyn RequestContextHandler>>,
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
            .and_then(|settings| Some(&settings as *const _))
            .unwrap_or_else(null);
        let handler_ptr = if let Some(handler) = self.1 {
            RequestContextHandlerWrapper::wrap(handler)
        } else {
            null()
        };
        RequestContext(unsafe {
            cef_request_context_create_context(settings_ptr, handler_ptr as *mut _)
        })
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

    /// Optionally supply a handler to the request context. See [RequestContextHandler].
    pub fn with_handler(mut self, handler: Box<dyn RequestContextHandler>) -> Self {
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
    pub fn with_cache_path(mut self, path: &str) -> Self {
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
