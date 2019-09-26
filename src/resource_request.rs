use cef_sys::{cef_base_ref_counted_t, cef_resource_request_handler_t};

use crate::{
    refcounted::{RefCounter, RefCounted},
    browser::Browser,
    frame::Frame,
    request::Request,
    urlrequest::{CookieAccessFilter, RequestCallback, ResourceHandler, Response, ResponseFilter, URLRequestStatus},
    ReturnValue,
};

/// Implement this trait to handle events related to browser requests. The
/// functions of this trait will be called on the IO thread unless otherwise
/// indicated.
pub trait ResourceRequestHandler: Sync + Send {
    /// Called on the IO thread before a resource request is loaded. The `browser`
    /// and `frame` values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest]. To
    /// optionally filter cookies for the request return a
    /// [CookieAccessFilter] object.
    fn get_cookie_access_filter(&self, browser: Option<&Browser>, frame: Option<&Frame>, request: &Request) -> Option<Box<dyn CookieAccessFilter>> { None }
    /// Called on the IO thread before a resource request is loaded. The `browser`
    /// and `frame` values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest]. To redirect
    /// or change the resource load optionally modify `request`. Modification of
    /// the request URL will be treated as a redirect. Return RV_CONTINUE to
    /// continue the request immediately. Return RV_CONTINUE_ASYNC and call
    /// [RequestCallback::cont] at a later time to continue or cancel the
    /// request asynchronously. Return RV_CANCEL to cancel the request immediately.
    fn on_before_resource_load(&self, browser: Option<&Browser>, frame: Option<&Frame>, request: &Request, callback: RequestCallback) -> ReturnValue { ReturnValue::Cancel }
    /// Called on the IO thread before a resource is loaded. The `browser` and
    /// `frame` values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest]. To allow the
    /// resource to load using the default network loader return None. To specify a
    /// handler for the resource return a [ResourceHandler] object.
    fn get_resource_handler(&self, browser: Option<&Browser>, frame: Option<&Frame>, request: &Request) -> Option<Box<dyn ResourceHandler>> { None }
    /// Called on the IO thread when a resource load is redirected. The `browser`
    /// and `frame|`values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest]. The
    /// `request` parameter will contain the old URL and other request-related
    /// information. The `response` parameter will contain the response that
    /// resulted in the redirect. The `new_url` parameter will contain the new URL
    /// and can be changed if desired.
    fn on_resource_redirect(&self, browser: Option<&Browser>, frame: Option<&Frame>, request: &Request, response: &Response, new_url: &str) {}
    /// Called on the IO thread when a resource response is received. The `browser`
    /// and `frame` values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest]. To allow the
    /// resource load to proceed without modification return false. To redirect
    /// or retry the resource load optionally modify `request` and return true.
    /// Modification of the request URL will be treated as a redirect. Requests
    /// handled using the default network loader cannot be redirected in this
    /// callback. The `response` object cannot be modified in this callback.
    ///
    /// WARNING: Redirecting using this function is deprecated. Use
    /// OnBeforeResourceLoad or GetResourceHandler to perform redirects.
    fn on_resource_response(&self, browser: Option<&Browser>, frame: Option<&Frame>, request: &mut Request, response: &Response) -> bool { false }
    /// Called on the IO thread to optionally filter resource response content. The
    /// `browser` and `frame` values represent the source of the request, and may
    /// be None for requests originating from service workers or [URLRequest].
    fn get_resource_response_filter(&self, browser: Option<&Browser>, frame: Option<&Frame>, request: &Request, response: &Response) -> Option<Box<dyn ResponseFilter>> { None }
    /// Called on the IO thread when a resource load has completed. The `browser`
    /// and `frame` values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest].
    /// `status` indicates the load completion
    /// status. `received_content_length` is the number of response bytes actually
    /// read. This function will be called for all requests, including requests
    /// that are aborted due to CEF shutdown or destruction of the associated
    /// browser. In cases where the associated browser is destroyed this callback
    /// may arrive after the [LifeSpanHandler::on_before_close] callback for
    /// that browser. The [Frame::is_valid] function can be used to test for
    /// this situation, and care should be taken not to call `browser` or `frame`
    /// functions that modify state (like LoadURL, SendProcessMessage, etc.) if the
    /// frame is invalid.
    fn on_resource_load_complete(&self, browser: Option<&Browser>, frame: Option<&Frame>, request: &Request, response: &Response, status: URLRequestStatus, received_content_length: i64) {}
    /// Called on the IO thread to handle requests for URLs with an unknown
    /// protocol component. The `browser` and `frame` values represent the source
    /// of the request, and may be None for requests originating from service
    /// workers or [URLRequest].
    /// Set `allow_os_execution` to true to attempt execution via the
    /// registered OS protocol handler, if any.
    ///
    /// SECURITY WARNING: YOU SHOULD USE
    /// THIS METHOD TO ENFORCE RESTRICTIONS BASED ON SCHEME, HOST OR OTHER URL
    /// ANALYSIS BEFORE ALLOWING OS EXECUTION.
    fn on_protocol_execution(&self, browser: Option<&Browser>, frame: Option<&Frame>, request: &Request, response: &Response, allow_os_execution: &mut bool) {}
}

pub(crate) struct ResourceRequestHandlerWrapper {
    delegate: Box<dyn ResourceRequestHandler>,
    cookie_access_filter: Option<Box<dyn CookieAccessFilter>>,
    resource_handler: Option<Box<dyn ResourceHandler>>,
    response_filter: Option<Box<dyn ResponseFilter>>,
}

impl RefCounter for cef_resource_request_handler_t {
    type Wrapper = RefCounted<Self, ResourceRequestHandlerWrapper>;
    fn set_base(&mut self, base: cef_base_ref_counted_t) {
        self.base = base;
    }
}

impl ResourceRequestHandlerWrapper {
    pub(crate) fn wrap(delegate: Box<dyn ResourceRequestHandler>) -> *mut cef_resource_request_handler_t {
        let mut rc = RefCounted::new(cef_resource_request_handler_t {
            ..Default::default()
        }, Self {
            delegate,
            cookie_access_filter: None,
            resource_handler: None,
            response_filter: None,
        });
        unsafe { &mut *rc }.get_cef()
    }
}
