use parking_lot::Mutex;
use cef_sys::{cef_scheme_handler_factory_t, cef_browser_t, cef_frame_t, cef_string_t, cef_request_t, cef_resource_handler_t, cef_register_scheme_handler_factory, cef_clear_scheme_handler_factories};
use crate::{
    browser::Browser,
    frame::Frame,
    request::Request,
    refcounted::{RefCountedPtr, Wrapper},
    url_request::ResourceHandler,
    string::CefString,
};
use std::ptr;

ref_counted_ptr!{
    /// Structure that creates [`ResourceHandler`] instances for handling scheme
    /// requests. The functions of this structure will always be called on the IO
    /// thread.
    pub struct SchemeHandlerFactory(*mut cef_scheme_handler_factory_t);
}

pub enum SchemeName<'a> {
    Custom { name: &'a str },
    HTTP   { domain: Option<&'a str> },
    HTTPS  { domain: Option<&'a str> },
    FILE   { domain: Option<&'a str> },
    FTP    { domain: Option<&'a str> },
    ABOUT  { domain: Option<&'a str> },
    DATA   { domain: Option<&'a str> },
}

impl SchemeHandlerFactory {
    pub fn new<C: SchemeHandlerFactoryCallbacks>(callbacks: C) -> SchemeHandlerFactory {
        unsafe{ SchemeHandlerFactory::from_ptr_unchecked(SchemeHandlerFactoryWrapper(Mutex::new(Box::new(callbacks))).wrap().into_raw()) }
    }

    /// Register a scheme handler factory with the global request context. A None
    /// `domain_name.domain` value for a standard scheme will cause the factory to match all
    /// domain names. If `scheme_name` is [SchemeName::Custom] then you must also implement the
    /// [App::on_register_custom_schemes] function in all processes. This
    /// function may be called multiple times to change or remove the factory that
    /// matches the specified `scheme_name` and optional `scheme_name.domain`. Returns false
    /// if an error occurs. This function may be called on any thread in the
    /// browser process. Using this function is equivalent to calling
    /// [RequestContext::request_context_get_global_context]().register_scheme_handler_factory()
    pub fn register<'a>(&self, scheme_name: SchemeName<'a>) -> bool {
        let (scheme_name, domain_name) = match scheme_name {
            SchemeName::Custom { name }   => (name, None),
            SchemeName::HTTP   { domain } => ("http",  domain),
            SchemeName::HTTPS  { domain } => ("https", domain),
            SchemeName::FILE   { domain } => ("file",  domain),
            SchemeName::FTP    { domain } => ("ftp",   domain),
            SchemeName::ABOUT  { domain } => ("about", domain),
            SchemeName::DATA   { domain } => ("data",  domain),
        };
        let scheme_name = CefString::from(scheme_name);
        let domain_name = domain_name.map(CefString::from);
        unsafe {
            cef_register_scheme_handler_factory(scheme_name.as_ptr(), domain_name.map(|s| s.as_ptr()).unwrap_or_else(ptr::null), self.as_ptr()) != 0
        }
    }
    /// Clear all scheme handler factories registered with the global request
    /// context. Returns false on error. This function may be called on any
    /// thread in the browser process. Using this function is equivalent to calling
    /// [RequestContext::request_context_get_global_context]()->clear_scheme_handler_factories().
    pub fn clear() -> bool {
        unsafe {
            cef_clear_scheme_handler_factories() != 0
        }
    }
}

pub trait SchemeHandlerFactoryCallbacks: 'static + Send {
    /// Return a new resource handler instance to handle the request or an `None`
    /// reference to allow default handling of the request. `browser` and `frame`
    /// will be the browser window and frame respectively that originated the
    /// request or `None` if the request did not originate from a browser window (for
    /// example, if the request came from cef_urlrequest_t). The `request` object
    /// passed to this function cannot be modified.
    fn create(
        &self,
        browser: Browser,
        frame: Frame,
        scheme_name: &str,
        request: Request,
    ) -> Option<ResourceHandler>;
}

struct SchemeHandlerFactoryWrapper(Mutex<Box<dyn SchemeHandlerFactoryCallbacks>>);
impl Wrapper for SchemeHandlerFactoryWrapper {
    type Cef = cef_scheme_handler_factory_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_scheme_handler_factory_t {
                base: unsafe { std::mem::zeroed() },
                create: Some(Self::create),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for SchemeHandlerFactoryWrapper: cef_scheme_handler_factory_t {
        fn create(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t,
            scheme_name: String: *const cef_string_t,
            request: Request: *mut cef_request_t
        ) -> *mut cef_resource_handler_t {
            self.0.lock().create(
                browser,
                frame,
                &scheme_name,
                request,
            ).map(|h| h.into_raw()).unwrap_or(ptr::null_mut())
        }
    }
}
