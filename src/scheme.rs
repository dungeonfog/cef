use parking_lot::Mutex;
use cef_sys::{cef_scheme_handler_factory_t, cef_browser_t, cef_frame_t, cef_string_t, cef_request_t, cef_resource_handler_t};
use crate::{
    browser::Browser,
    frame::Frame,
    request::Request,
    refcounted::{RefCountedPtr, Wrapper},
    url_request::ResourceHandler,
};
use std::ptr;

ref_counted_ptr!{
    /// Structure that creates [`ResourceHandler`] instances for handling scheme
    /// requests. The functions of this structure will always be called on the IO
    /// thread.
    pub struct SchemeHandlerFactory(*mut cef_scheme_handler_factory_t);
}

impl SchemeHandlerFactory {
    pub fn new<C: SchemeHandlerFactoryCallbacks>(callbacks: C) -> SchemeHandlerFactory {
        unsafe{ SchemeHandlerFactory::from_ptr_unchecked(SchemeHandlerFactoryWrapper(Mutex::new(Box::new(callbacks))).wrap().into_raw()) }
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
