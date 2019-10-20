use cef_sys::{
    cef_auth_callback_t, cef_browser_t, cef_callback_t,
    cef_cookie_access_filter_t, cef_cookie_t, cef_frame_t, cef_request_callback_t, cef_request_t, cef_resource_handler_t, cef_resource_read_callback_t,
    cef_resource_skip_callback_t, cef_response_filter_status_t, cef_response_filter_t,
    cef_response_t, cef_string_t, cef_urlrequest_client_t, cef_urlrequest_create,
    cef_urlrequest_status_t, cef_urlrequest_t,
};
use num_enum::UnsafeFromPrimitive;
use std::{
    ptr::null_mut,
    sync::Arc,
};

use crate::{
    browser::Browser,
    callback::Callback,
    cookie::Cookie,
    frame::Frame,
    load_handler::ErrorCode,
    refcounted::{RefCountedPtr, Wrapper},
    request::Request,
    request_context::RequestContext,
    string::CefString,
};

/// Flags that represent [URLRequest] status.
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum URLRequestStatus {
    /// Unknown status.
    Unknown = cef_urlrequest_status_t::UR_UNKNOWN as i32,
    /// Request succeeded.
    Success = cef_urlrequest_status_t::UR_SUCCESS as i32,
    /// An IO request is pending, and the caller will be informed when it is
    /// completed.
    IOPending = cef_urlrequest_status_t::UR_IO_PENDING as i32,
    /// Request was canceled programatically.
    Canceled = cef_urlrequest_status_t::UR_CANCELED as i32,
    /// Request failed for some reason.
    Failed = cef_urlrequest_status_t::UR_FAILED as i32,
}

ref_counted_ptr! {
    /// Structure used to make a URL request. URL requests are not associated with a
    /// browser instance so no [Client] callbacks will be executed. URL requests
    /// can be created on any valid CEF thread in either the browser or render
    /// process. Once created the functions of the URL request object must be
    /// accessed on the same thread that created it.
    pub struct URLRequest(*mut cef_urlrequest_t);
}

impl URLRequest {
    /// Create a new URL request that is not associated with a specific browser or
    /// frame. Use [Frame::create_urlrequest] instead if you want the request to
    /// have this association, in which case it may be handled differently (see
    /// documentation on that function). Requests may originate from the both browser
    /// process and the render process.
    ///
    /// For requests originating from the browser process:
    ///   - It may be intercepted by the client via [ResourceRequestHandler] or
    ///     [SchemeHandlerFactory].
    ///   - POST data may only contain only a single element of type [PostDataElementType::File]
    ///     or [PostDataElementType::Bytes].
    ///   - If `request_context` is empty the global request context will be used.
    /// For requests originating from the render process:
    ///   - It cannot be intercepted by the client so only http(s) and blob schemes
    ///     are supported.
    ///   - POST data may only contain a single element of type [PostDataElementType::Bytes].
    ///   - The `request_context` parameter must be None.
    ///
    /// The `request` object will be marked as read-only after calling this function.
    pub fn new(
        request: &mut Request,
        client: Arc<dyn URLRequestClient>,
        request_context: Option<&RequestContext>,
    ) -> Self {
        unsafe {
            Self::from_ptr_unchecked(cef_urlrequest_create(
                request.as_ptr(),
                URLRequestClientWrapper::new(client).wrap().into_raw(),
                request_context
                    .and_then(|ctx| Some(ctx.as_ptr()))
                    .unwrap_or_else(null_mut),
            ))
        }
    }
    /// Returns the request object used to create this URL request. The returned
    /// object is read-only and should not be modified.
    pub fn get_request(&self) -> Request {
        unsafe { Request::from_ptr_unchecked(self.0.get_request.unwrap()(self.as_ptr())) }
    }
    /// Returns the request status.
    pub fn get_request_status(&self) -> URLRequestStatus {
        unsafe {
            URLRequestStatus::from_unchecked(
                self.0.get_request_status.unwrap()(self.as_ptr()) as i32
            )
        }
    }
    /// Returns the request error if status is [URLRequestStatus::Canceled] or [URLRequestStatus::Failed], or [ErrorCode::None]
    /// otherwise.
    pub fn get_request_error(&self) -> ErrorCode {
        unsafe {
            ErrorCode::from_unchecked(self.0.get_request_error.unwrap()(self.as_ptr()) as i32)
        }
    }
    /// Returns the response, or None if no response information is available.
    /// Response information will only be available after the upload has completed.
    /// The returned object is read-only and should not be modified.
    pub fn get_response(&self) -> Option<Response> {
        unsafe { Response::from_ptr(self.0.get_response.unwrap()(self.as_ptr())) }
    }
    /// Returns true if the response body was served from the cache. This
    /// includes responses for which revalidation was required.
    pub fn response_was_cached(&self) -> bool {
        unsafe { self.0.response_was_cached.unwrap()(self.as_ptr()) != 0 }
    }
    /// Cancel the request.
    pub fn cancel(&self) {
        unsafe { self.0.cancel.unwrap()(self.as_ptr()) }
    }
}

ref_counted_ptr! {
    /// Callback structure used for asynchronous continuation of authentication
    /// requests.
    pub struct AuthCallback(*mut cef_auth_callback_t);
}

impl AuthCallback {
    /// Continue the authentication request.
    pub fn cont(&self, username: &str, password: &str) {
        if let Some(cont) = self.0.cont {
            unsafe {
                cont(
                    self.as_ptr(),
                    CefString::new(username).as_ptr(),
                    CefString::new(password).as_ptr(),
                );
            }
        }
        // TODO: WHY IS THIS HERE?
        // unsafe { ((&*self.0).base.release.unwrap())(&mut (&mut *self.0).base) };
    }
    /// Cancel the authentication request.
    pub fn cancel(&self) {
        if let Some(cancel) = self.0.cancel {
            unsafe {
                cancel(self.as_ptr());
            }
        }
        // TODO: WHY IS THIS HERE?
        // unsafe { ((&*self.0).base.release.unwrap())(&mut (&mut *self.0).base) };
    }
}

/// Trait that should be implemented by the [URLRequest] client. The
/// functions of this trait will be called on the same thread that created
/// the request unless otherwise documented.
pub trait URLRequestClient: Send + Sync {
    /// Notifies the client that the request has completed. Use the
    /// [URLRequest::get_request_status] function to determine if the request was
    /// successful or not.
    fn on_request_complete(&self, request: URLRequest) {}
    /// Notifies the client of upload progress. `current` denotes the number of
    /// bytes sent so far and `total` is the total size of uploading data (or -1 if
    /// chunked upload is enabled). This function will only be called if the
    /// [URLRequestFlags::ReportUploadProgress] flag is set on the request.
    fn on_upload_progress(&self, request: URLRequest, current: i64, total: i64) {}
    /// Notifies the client of download progress. `current` denotes the number of
    /// bytes received up to the call and `total` is the expected total size of the
    /// response (or -1 if not determined).
    fn on_download_progress(&self, request: URLRequest, current: i64, total: i64) {}
    /// Called when some part of the response is read. `data` contains the current
    /// bytes received since the last call. This function will not be called if the
    /// [URLRequestFlags::NoDownloadData] flag is set on the request.
    fn on_download_data(&self, request: URLRequest, data: &[u8]) {}
    /// Called on the IO thread when the browser needs credentials from the user.
    /// `is_proxy` indicates whether the host is a proxy server. `host` contains the
    /// hostname and `port` contains the port number. Return true to continue
    /// the request and call [AuthCallback::cont] when the authentication
    /// information is available. If the request has an associated browser/frame
    /// then returning false will result in a call to [RequestHandler::GetAuthCredentials] on the
    /// [RequestHandler] associated with that browser, if any. Otherwise,
    /// returning false will cancel the request immediately. This function will
    /// only be called for requests initiated from the browser process.
    fn get_auth_credentials(
        &self,
        is_proxy: bool,
        host: &str,
        port: u16,
        realm: &str,
        scheme: &str,
        callback: AuthCallback,
    ) -> bool {
        false
    }
}

pub(crate) struct URLRequestClientWrapper {
    delegate: Arc<dyn URLRequestClient>,
}

impl std::borrow::Borrow<Arc<dyn URLRequestClient>> for URLRequestClientWrapper {
    fn borrow(&self) -> &Arc<dyn URLRequestClient> {
        &self.delegate
    }
}

impl Wrapper for URLRequestClientWrapper {
    type Cef = cef_urlrequest_client_t;
    type Inner = dyn URLRequestClient;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_urlrequest_client_t {
                base: unsafe { std::mem::zeroed() },
                on_request_complete: Some(Self::request_complete),
                on_upload_progress: Some(Self::upload_progress),
                on_download_progress: Some(Self::download_progress),
                on_download_data: Some(Self::download_data),
                get_auth_credentials: Some(Self::get_auth_credentials),
            },
            self,
        )
    }
}

impl URLRequestClientWrapper {
    pub(crate) fn new(delegate: Arc<dyn URLRequestClient>) -> URLRequestClientWrapper {
        URLRequestClientWrapper {
            delegate,
        }
    }
}

cef_callback_impl!{
    impl for URLRequestClientWrapper: cef_urlrequest_client_t {
        fn request_complete(
            &self,
            request: URLRequest: *mut cef_urlrequest_t,
        ) {
            self.delegate.on_request_complete(request);
        }
        fn upload_progress(
            &self,
            request: URLRequest: *mut cef_urlrequest_t,
            current: i64: i64,
            total: i64: i64,
        ) {
            self.delegate.on_upload_progress(
                request,
                current,
                total,
            );
        }
        fn download_progress(
            &self,
            request: URLRequest: *mut cef_urlrequest_t,
            current: i64: i64,
            total: i64: i64,
        ) {
            self.delegate.on_download_progress(
                request,
                current,
                total,
            );
        }
        fn download_data(
            &self,
            request: URLRequest: *mut cef_urlrequest_t,
            data: *const std::os::raw::c_void: *const std::os::raw::c_void,
            data_length: usize: usize,
        ) {
            self.delegate.on_download_data(
                request,
                unsafe { std::slice::from_raw_parts(data as *const u8, data_length) },
            );
        }
        fn get_auth_credentials(
            &self,
            is_proxy: bool: std::os::raw::c_int,
            host: &CefString: *const cef_string_t,
            port: std::os::raw::c_int: std::os::raw::c_int,
            realm: &CefString: *const cef_string_t,
            scheme: &CefString: *const cef_string_t,
            callback: AuthCallback: *mut cef_auth_callback_t,
        ) -> i32 {
            self.delegate.get_auth_credentials(
                is_proxy,
                &String::from(host),
                port as u16,
                &String::from(realm),
                &String::from(scheme),
                callback,
            ) as i32
        }
    }
}

ref_counted_ptr! {
    /// Structure used to represent a web response. The functions of this structure
    /// may be called on any thread.
    pub struct Response(*mut cef_response_t);
}

ref_counted_ptr! {
    /// Callback structure used for asynchronous continuation of url requests.
    pub struct RequestCallback(*mut cef_request_callback_t);
}

impl RequestCallback {
    /// Continue the url request. If `allow` is true the request will be
    /// continued. Otherwise, the request will be canceled.
    pub fn cont(&self, allow: bool) {
        unsafe {
            self.0.cont.unwrap()(self.0.as_ptr(), allow as i32);
        }
    }
    /// Cancel the url request.
    pub fn cancel(&self) {
        unsafe {
            self.0.cancel.unwrap()(self.0.as_ptr());
        }
    }
}

/// Implement this trait to filter cookies that may be sent or received from
/// resource requests. The functions of this trait will be called on the IO
/// thread unless otherwise indicated.
pub trait CookieAccessFilter: Sync + Send {
    /// Called on the IO thread before a resource request is sent. The `browser`
    /// and `frame` values represent the source of the request, and may be None for
    /// requests originating from service workers or [URLRequest].
    /// Return true if the specified  cookie can be sent with the request or false otherwise.
    fn can_send_cookie(
        &self,
        browser: Option<Browser>,
        frame: Option<Frame>,
        request: Request,
        cookie: Cookie,
    ) -> bool {
        false
    }
    /// Called on the IO thread after a resource response is received. The
    /// `browser` and `frame` values represent the source of the request, and may
    /// be None for requests originating from service workers or [URLRequest].
    /// Return true if the
    /// specified cookie returned with the response can be saved or false
    /// otherwise.
    fn can_save_cookie(
        &self,
        browser: Option<Browser>,
        frame: Option<Frame>,
        request: Request,
        response: Response,
        cookie: Cookie,
    ) -> bool {
        false
    }
}

pub(crate) struct CookieAccessFilterWrapper {
    delegate: Arc<dyn CookieAccessFilter>,
}

impl std::borrow::Borrow<Arc<dyn CookieAccessFilter>> for CookieAccessFilterWrapper {
    fn borrow(&self) -> &Arc<dyn CookieAccessFilter> {
        &self.delegate
    }
}

impl Wrapper for CookieAccessFilterWrapper {
    type Cef = cef_cookie_access_filter_t;
    type Inner = dyn CookieAccessFilter;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_cookie_access_filter_t {
                base: unsafe { std::mem::zeroed() },
                can_send_cookie: Some(Self::can_send_cookie),
                can_save_cookie: Some(Self::can_save_cookie),
            },
            self,
        )
    }
}

impl CookieAccessFilterWrapper {
    pub(crate) fn new(
        delegate: Arc<dyn CookieAccessFilter>,
    ) -> CookieAccessFilterWrapper {
        CookieAccessFilterWrapper {
            delegate
        }
    }
}

cef_callback_impl!{
    impl for CookieAccessFilterWrapper: cef_cookie_access_filter_t {
        fn can_send_cookie(
            &self,
            browser: Option<Browser>: *mut cef_browser_t,
            frame: Option<Frame>: *mut cef_frame_t,
            request: Request: *mut cef_request_t,
            cookie: Cookie: *const cef_cookie_t,
        ) -> std::os::raw::c_int {
            self.delegate.can_send_cookie(
                browser,
                frame,
                request,
                cookie,
            ) as std::os::raw::c_int
        }

        fn can_save_cookie(
            &self,
            browser: Option<Browser>: *mut cef_browser_t,
            frame: Option<Frame>: *mut cef_frame_t,
            request: Request: *mut cef_request_t,
            response: Response: *mut cef_response_t,
            cookie: Cookie: *const cef_cookie_t,
        ) -> std::os::raw::c_int {
            self.delegate.can_save_cookie(
                browser,
                frame,
                request,
                response,
                cookie,
            ) as std::os::raw::c_int
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, UnsafeFromPrimitive)]
pub enum ResponseFilterStatus {
    NeedMoreData = cef_response_filter_status_t::RESPONSE_FILTER_NEED_MORE_DATA as i32,
    Done = cef_response_filter_status_t::RESPONSE_FILTER_DONE as i32,
    Error = cef_response_filter_status_t::RESPONSE_FILTER_ERROR as i32,
}

/// Implement this trait to filter resource response content. The functions
/// of this trait will be called on the browser process IO thread.
pub trait ResponseFilter: Send + Sync {
    /// Initialize the response filter. Will only be called a single time. The
    /// filter will not be installed if this function returns false.
    fn init_filter(&self) -> bool {
        false
    }
    /// Called to filter a chunk of data. Expected usage is as follows:
    ///
    ///  A. Read input data from `data_in` and set `data_in_read` to the number of
    ///     bytes that were read up to a maximum of `data_in.len()`. `data_in` can
    ///     be size 0.
    ///  B. Write filtered output data to `data_out` and set `data_out_written` to
    ///     the number of bytes that were written up to a maximum of
    ///     `data_out.len()`. If no output data was written then all data must be
    ///     read from `data_in` (user must set `data_in_read` = `data_in.len()`).
    ///  C. Return [ResponseFilterStatus::Done] if all output data was written or
    ///     [ResponseFilterStatus::NeedMoreData] if output data is still pending.
    ///
    /// This function will be called repeatedly until the input slice has been
    /// fully read (user sets `data_in_read` = `data_in.len()`) and there is no more
    /// input data to filter (the resource response is complete). This function may
    /// then be called an additional time with an zero-length input slice if the user
    /// filled the output slice (set `data_out_written` = `data_out.len()`) and
    /// returned [ResponseFilterStatus::NeedMoreData] to indicate that output data is
    /// still pending.
    ///
    /// Calls to this function will stop when one of the following conditions is
    /// met:
    ///
    ///  A. There is no more input data to filter (the resource response is
    ///     complete) and the user sets `data_out_written` = 0 or returns
    ///     [ResponseFilterStatus::Done] to indicate that all data has been written, or;
    ///  B. The user returns [ResponseFilterStatus::Error] to indicate an error.
    fn filter(
        &self,
        data_in: &[u8],
        data_in_read: &mut usize,
        data_out: &mut [u8],
        data_out_written: &mut usize,
    ) -> ResponseFilterStatus {
        ResponseFilterStatus::Error
    }
}

pub(crate) struct ResponseFilterWrapper {
    delegate: Arc<dyn ResponseFilter>,
}

impl std::borrow::Borrow<Arc<dyn ResponseFilter>> for ResponseFilterWrapper {
    fn borrow(&self) -> &Arc<dyn ResponseFilter> {
        &self.delegate
    }
}

impl Wrapper for ResponseFilterWrapper {
    type Cef = cef_response_filter_t;
    type Inner = dyn ResponseFilter;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_response_filter_t {
                base: unsafe { std::mem::zeroed() },
                init_filter: Some(Self::init_filter),
                filter: Some(Self::filter),
            },
            self,
        )
    }
}

impl ResponseFilterWrapper {
    pub(crate) fn new(delegate: Arc<dyn ResponseFilter>) -> ResponseFilterWrapper {
        ResponseFilterWrapper { delegate }
    }
}
cef_callback_impl!{
    impl for ResponseFilterWrapper: cef_response_filter_t {
        fn init_filter(&self) -> std::os::raw::c_int {
            self.delegate.init_filter() as std::os::raw::c_int
        }
        fn filter(
            &self,
            data_in: *mut std::os::raw::c_void: *mut std::os::raw::c_void,
            data_in_size: usize: usize,
            data_in_read: &mut usize: *mut usize,
            data_out: *mut std::os::raw::c_void: *mut std::os::raw::c_void,
            data_out_size: usize: usize,
            data_out_written: &mut usize: *mut usize,
        ) -> cef_response_filter_status_t::Type {
            self.delegate.filter(
                unsafe { (data_in as *const u8).as_ref().map(|data_in| std::slice::from_raw_parts(data_in, data_in_size)).unwrap_or(&[]) },
                data_in_read,
                unsafe { std::slice::from_raw_parts_mut(data_out as *mut u8, data_out_size) },
                data_out_written
            ) as cef_response_filter_status_t::Type
        }
    }
}

/// Structure used to implement a custom request handler structure. The functions
/// of this structure will be called on the IO thread unless otherwise indicated.
pub trait ResourceHandler: Send + Sync {
    /// Open the response stream. To handle the request immediately set
    /// `handle_request` to true and return true. To decide at a later time
    /// set `handle_request` to false, return true, and execute `callback`
    /// to continue or cancel the request. To cancel the request immediately set
    /// `handle_request` to true and return false. This function will be
    /// called in sequence but not from a dedicated thread.
    fn open(&self, request: &Request, handle_request: &mut bool, callback: Callback) -> bool {
        (*handle_request) = true;
        false
    }
    /// Retrieve response header information. If the response length is not known
    /// set `response_length` to -1 and [ResourceHandler::read_response] will be called until it
    /// returns false. If the response length is known set `response_length` to
    /// a positive value and [ResourceHandler::read_response] will be called until it returns false
    /// or the specified number of bytes have been read. Use the `response`
    /// object to set the mime type, http status code and other optional header
    /// values. To redirect the request to a new URL set `redirect_url` to the new
    /// URL. `redirect_url` can be either a relative or fully qualified URL. It is
    /// also possible to set `response` to a redirect http status code and pass the
    /// new URL via a Location header. Likewise with `redirect_url` it is valid to
    /// set a relative or fully qualified URL as the Location header value. If an
    /// error occured while setting up the request you can call [Response::set_error] on
    /// `response` to indicate the error condition.
    fn get_response_headers(
        &self,
        response: &mut Response,
        response_length: &mut i64,
        redirect_url: &mut String,
    ) {
    }
    /// Skip response data when requested by a Range header. Skip over and discard
    /// `bytes_to_skip` bytes of response data. If data is available immediately
    /// set `bytes_skipped` to the number of bytes skipped and return true. To
    /// read the data at a later time set `bytes_skipped` to 0, return true and
    /// execute `callback` when the data is available. To indicate failure set
    /// `bytes_skipped` to < 0 (e.g. -2 for [ErrorCode::Failed]) and return false. This
    /// function will be called in sequence but not from a dedicated thread.
    fn skip(&self, bytes_to_skip: i64, bytes_skipped: &mut i64, callback: Callback) -> bool {
        false
    }
    /// Read response data. If data is available immediately copy up to
    /// the slice len into `data_out`, set `bytes_read` to the number of
    /// bytes copied, and return true. To read the data at a later time keep a
    /// reference to `data_out`, set `bytes_read` to 0, return true and execute
    /// `callback` when the data is available (`data_out` will remain valid until
    /// the callback is executed). To indicate response completion set `bytes_read`
    /// to 0 and return false. To indicate failure set `bytes_read` to < 0
    /// (e.g. -2 for [ErrorCode::Failed]) and return false. This function will be called
    /// in sequence but not from a dedicated thread.
    fn read(&self, data_out: &mut [u8], bytes_read: &mut i32, callback: Callback) -> bool {
        false
    }
    /// Request processing has been canceled.
    fn cancel(&self) {}
}

pub(crate) struct ResourceHandlerWrapper {
    delegate: Arc<dyn ResourceHandler>,
}

impl std::borrow::Borrow<Arc<dyn ResourceHandler>> for ResourceHandlerWrapper {
    fn borrow(&self) -> &Arc<dyn ResourceHandler> {
        &self.delegate
    }
}

impl Wrapper for ResourceHandlerWrapper {
    type Cef = cef_resource_handler_t;
    type Inner = dyn ResourceHandler;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_resource_handler_t {
                base: unsafe { std::mem::zeroed() },
                open: Some(Self::open),
                get_response_headers: Some(Self::get_response_headers),
                skip: Some(Self::skip),
                read: Some(Self::read),
                cancel: Some(Self::cancel),
                // deprecated callbacks:
                process_request: None,
                read_response: None,
            },
            self,
        )
    }
}

impl ResourceHandlerWrapper {
    pub(crate) fn new(delegate: Arc<dyn ResourceHandler>) -> ResourceHandlerWrapper {
        ResourceHandlerWrapper { delegate }
    }
    // TODO: move to cef_callback_impl
    extern "C" fn open(
        self_: *mut cef_resource_handler_t,
        request: *mut cef_request_t,
        handle_request: *mut std::os::raw::c_int,
        callback: *mut cef_callback_t,
    ) -> std::os::raw::c_int {
        unimplemented!()
    }
    extern "C" fn get_response_headers(
        self_: *mut cef_resource_handler_t,
        response: *mut cef_response_t,
        response_length: *mut i64,
        redirect_url: *mut cef_string_t,
    ) {
        unimplemented!()
    }
    extern "C" fn skip(
        self_: *mut cef_resource_handler_t,
        bytes_to_skip: i64,
        bytes_skipped: *mut i64,
        callback: *mut cef_resource_skip_callback_t,
    ) -> std::os::raw::c_int {
        unimplemented!()
    }
    extern "C" fn read(
        self_: *mut cef_resource_handler_t,
        data_out: *mut std::os::raw::c_void,
        bytes_to_read: std::os::raw::c_int,
        bytes_read: *mut std::os::raw::c_int,
        callback: *mut cef_resource_read_callback_t,
    ) -> std::os::raw::c_int {
        unimplemented!()
    }
    extern "C" fn cancel(self_: *mut cef_resource_handler_t) {
        unimplemented!()
    }
}
