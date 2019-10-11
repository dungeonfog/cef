use cef_sys::{
    cef_base_ref_counted_t, cef_browser_t, cef_cookie_access_filter_t, cef_frame_t,
    cef_request_callback_t, cef_request_t, cef_resource_handler_t, cef_resource_request_handler_t,
    cef_response_filter_t, cef_response_t, cef_return_value_t, cef_string_t,
    cef_string_utf8_to_utf16, cef_urlrequest_status_t,
};
use std::ptr::null_mut;

use crate::{
    browser::Browser,
    frame::Frame,
    refcounted::{RefCounted, RefCountedPtr, RefCounter},
    request::Request,
    string::CefString,
    url_request::{
        CookieAccessFilter, CookieAccessFilterWrapper, RequestCallback, ResourceHandler,
        ResourceHandlerWrapper, Response, ResponseFilter, ResponseFilterWrapper, URLRequestStatus,
    },
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
    fn get_cookie_access_filter(
        &self,
        browser: Option<&Browser>,
        frame: Option<&Frame>,
        request: &Request,
    ) -> Option<Box<dyn CookieAccessFilter>> {
        None
    }
    /// Called on the IO thread before a resource request is loaded. The `browser`
    /// and `frame` values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest]. To redirect
    /// or change the resource load optionally modify `request`. Modification of
    /// the request URL will be treated as a redirect. Return [ReturnValue::Continue] to
    /// continue the request immediately. Return [ReturnValue::ContinueAsync] and call
    /// [RequestCallback::cont] at a later time to continue or cancel the
    /// request asynchronously. Return [ReturnValue::Cancel] to cancel the request immediately.
    fn on_before_resource_load(
        &self,
        browser: Option<&Browser>,
        frame: Option<&Frame>,
        request: &Request,
        callback: &RequestCallback,
    ) -> ReturnValue {
        ReturnValue::Cancel
    }
    /// Called on the IO thread before a resource is loaded. The `browser` and
    /// `frame` values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest]. To allow the
    /// resource to load using the default network loader return None. To specify a
    /// handler for the resource return a [ResourceHandler] object.
    fn get_resource_handler(
        &self,
        browser: Option<&Browser>,
        frame: Option<&Frame>,
        request: &Request,
    ) -> Option<Box<dyn ResourceHandler>> {
        None
    }
    /// Called on the IO thread when a resource load is redirected. The `browser`
    /// and `frame|`values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest]. The
    /// `request` parameter will contain the old URL and other request-related
    /// information. The `response` parameter will contain the response that
    /// resulted in the redirect. The `new_url` parameter will contain the new URL
    /// and can be changed if desired.
    fn on_resource_redirect(
        &self,
        browser: Option<&Browser>,
        frame: Option<&Frame>,
        request: &Request,
        response: &Response,
        new_url: &mut String,
    ) {
    }
    /// Called on the IO thread when a resource response is received. The `browser`
    /// and `frame` values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest].
    fn on_resource_response(
        &self,
        browser: Option<&Browser>,
        frame: Option<&Frame>,
        request: &Request,
        response: &Response,
    ) {
    }
    /// Called on the IO thread to optionally filter resource response content. The
    /// `browser` and `frame` values represent the source of the request, and may
    /// be None for requests originating from service workers or [URLRequest].
    fn get_resource_response_filter(
        &self,
        browser: Option<&Browser>,
        frame: Option<&Frame>,
        request: &Request,
        response: &Response,
    ) -> Option<Box<dyn ResponseFilter>> {
        None
    }
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
    fn on_resource_load_complete(
        &self,
        browser: Option<&Browser>,
        frame: Option<&Frame>,
        request: &Request,
        response: &Response,
        status: URLRequestStatus,
        received_content_length: i64,
    ) {
    }
    /// Called on the IO thread to handle requests for URLs with an unknown
    /// protocol component. The `browser` and `frame` values represent the source
    /// of the request, and may be None for requests originating from service
    /// workers or [URLRequest].
    /// Return true to attempt execution via the registered OS protocol handler, if any.
    ///
    /// SECURITY WARNING: YOU SHOULD USE
    /// THIS METHOD TO ENFORCE RESTRICTIONS BASED ON SCHEME, HOST OR OTHER URL
    /// ANALYSIS BEFORE ALLOWING OS EXECUTION.
    fn on_protocol_execution(
        &self,
        browser: Option<&Browser>,
        frame: Option<&Frame>,
        request: &Request,
    ) -> bool {
        false
    }
}

pub(crate) struct ResourceRequestHandlerWrapper {
    delegate: Box<dyn ResourceRequestHandler>,
    cookie_access_filter: Option<RefCountedPtr<cef_cookie_access_filter_t>>,
    resource_handler: Option<RefCountedPtr<cef_resource_handler_t>>,
    response_filter: Option<RefCountedPtr<cef_response_filter_t>>,
}

impl ResourceRequestHandlerWrapper {
    pub(crate) fn wrap(
        delegate: Box<dyn ResourceRequestHandler>,
    ) -> *mut cef_resource_request_handler_t {
        let mut rc = RefCounted::new(
            cef_resource_request_handler_t {
                base: unsafe { std::mem::zeroed() },
                get_cookie_access_filter: Some(Self::get_cookie_access_filter),
                on_before_resource_load: Some(Self::before_resource_load),
                get_resource_handler: Some(Self::get_resource_handler),
                on_resource_redirect: Some(Self::resource_redirect),
                on_resource_response: Some(Self::resource_response),
                get_resource_response_filter: Some(Self::get_resource_response_filter),
                on_resource_load_complete: Some(Self::resource_load_complete),
                on_protocol_execution: Some(Self::protocol_execution),
            },
            Self {
                delegate,
                cookie_access_filter: None,
                resource_handler: None,
                response_filter: None,
            },
        );
        unsafe { &mut *rc }.get_cef()
    }
}

cef_callback_impl! {
    impl ResourceRequestHandlerWrapper: cef_resource_request_handler_t {
        fn get_cookie_access_filter(
            &mut self,
            browser: Option<&Browser>: *mut cef_browser_t,
            frame  : Option<&Frame>  : *mut cef_frame_t,
            request: &Request        : *mut cef_request_t,
        ) -> *mut cef_cookie_access_filter_t
        {
            if let Some(filter) = self.delegate.get_cookie_access_filter(browser, frame, request) {
                self
                    .cookie_access_filter
                    .replace(CookieAccessFilterWrapper::wrap(filter));
                &mut **self.cookie_access_filter.as_mut().unwrap()
            } else {
                self.cookie_access_filter = None;
                null_mut()
            }
        }
        fn before_resource_load(
            &mut self,
            browser: Option<&Browser>: *mut cef_browser_t,
            frame: Option<&Frame>: *mut cef_frame_t,
            request: &Request: *mut cef_request_t,
            callback: &RequestCallback: *mut cef_request_callback_t,
        ) -> cef_return_value_t::Type {
            let result = self.delegate.on_before_resource_load(
                browser,
                frame,
                request,
                callback
            ) as i32;
            result as cef_return_value_t::Type
        }

        fn get_resource_handler(
            &mut self,
            browser: Option<&Browser>: *mut cef_browser_t,
            frame: Option<&Frame>: *mut cef_frame_t,
            request: &Request: *mut cef_request_t,
        ) -> *mut cef_resource_handler_t {
            if let Some(handler) = self.delegate.get_resource_handler(
                browser,
                frame,
                request,
            ) {
                self
                    .resource_handler
                    .replace(ResourceHandlerWrapper::wrap(handler));
                &mut **self.resource_handler.as_mut().unwrap()
            } else {
                self.resource_handler = None;
                null_mut()
            }
        }

        fn resource_redirect(
            &mut self,
            browser: Option<&Browser>: *mut cef_browser_t,
            frame: Option<&Frame>: *mut cef_frame_t,
            request: &Request: *mut cef_request_t,
            response: &Response: *mut cef_response_t,
            new_url: &mut CefString: *mut cef_string_t,
        ) {
            let mut new_url_rust = (&*new_url).into();
            self.delegate.on_resource_redirect(
                browser,
                frame,
                request,
                response,
                &mut new_url_rust,
            );
            new_url.set_string(&new_url_rust);
        }

        fn resource_response(
            &mut self,
            browser: Option<&Browser>: *mut cef_browser_t,
            frame: Option<&Frame>: *mut cef_frame_t,
            request: &Request: *mut cef_request_t,
            response: &Response: *mut cef_response_t,
        ) -> std::os::raw::c_int {
            self.delegate.on_resource_response(
                browser,
                frame,
                request,
                response,
            );
            0
        }

        fn get_resource_response_filter(
            &mut self,
            browser: Option<&Browser>: *mut cef_browser_t,
            frame: Option<&Frame>: *mut cef_frame_t,
            request: &Request: *mut cef_request_t,
            response: &Response: *mut cef_response_t,
        ) -> *mut cef_response_filter_t {
            if let Some(filter) = self.delegate.get_resource_response_filter(
                browser,
                frame,
                request,
                response,
            ) {
                self.response_filter
                    .replace(ResponseFilterWrapper::wrap(filter));
                &mut **self.response_filter.as_mut().unwrap()
            } else {
                self.response_filter = None;
                null_mut()
            }
        }

        fn resource_load_complete(
            &mut self,
            browser: Option<&Browser>: *mut cef_browser_t,
            frame: Option<&Frame>: *mut cef_frame_t,
            request: &Request: *mut cef_request_t,
            response: &Response: *mut cef_response_t,
            status: URLRequestStatus: cef_urlrequest_status_t::Type,
            received_content_length: i64: i64,
        ) {
            self.delegate.on_resource_load_complete(
                browser,
                frame,
                request,
                response,
                status,
                received_content_length,
            );
        }

        fn protocol_execution(
            &mut self,
            browser: Option<&Browser>: *mut cef_browser_t,
            frame: Option<&Frame>: *mut cef_frame_t,
            request: &Request: *mut cef_request_t,
            allow_os_execution: Option<&mut std::os::raw::c_int>: *mut std::os::raw::c_int,
        ) {
            if self.delegate.on_protocol_execution(
                browser,
                frame,
                request,
            ) {
                if let Some(allow_os_execution) = allow_os_execution {
                    *allow_os_execution = 1;
                }
            }
        }
    }
}
