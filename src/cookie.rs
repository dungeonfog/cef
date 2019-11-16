use crate::refcounted::Wrapper;
use crate::refcounted::RefCountedPtr;
use parking_lot::Mutex;
use crate::string::CefStringList;
use std::os::raw::c_int;
use cef_sys::cef_cookie_visitor_t;
use cef_sys::cef_set_cookie_callback_t;
use cef_sys::cef_delete_cookies_callback_t;
use cef_sys::{cef_cookie_t, cef_cookie_manager_t};
use chrono::{DateTime, Utc};

use crate::{
    callback::CompletionCallback,
    string::CefString,
};

/// Cookie information.
#[derive(Clone, Debug)]
pub struct Cookie {
    /// The cookie name.
    pub name: String,
    /// The cookie value.
    pub value: String,
    /// If `domain` is empty a host cookie will be created instead of a domain
    /// cookie. Domain cookies are stored with a leading "." and are visible to
    /// sub-domains whereas host cookies are not.
    pub domain: String,
    /// If `path` is non-empty only URLs at or below the path will get the cookie
    /// value.
    pub path: String,
    /// If `secure` is true the cookie will only be sent for HTTPS requests.
    pub secure: bool,
    /// If `httponly` is true the cookie will only be sent for HTTP requests.
    pub httponly: bool,
    /// The cookie creation date. This is automatically populated by the system on
    /// cookie creation.
    pub creation: DateTime<Utc>,
    /// The cookie last access date. This is automatically populated by the system
    /// on access.
    pub last_access: DateTime<Utc>,
    /// The cookie expiration date.
    pub expires: Option<DateTime<Utc>>,
}

pub struct CookieVisit<'a> {
    pub cookie: Cookie,
    /// 0-based index of current cookie.
    pub index: usize,
    /// Total number of cookies.
    pub len: usize,
    /// Set to `true` to delete the cookie.
    pub delete_cookie: &'a mut bool
}

/// Function for visiting cookie values. This always gets called on the UI thread.
///
/// Return `true` to continue visiting cookies, and `false` to stop.
pub trait CookieVisitorFn = 'static + Send + for<'a> FnMut(CookieVisit<'a>) -> bool;

impl Cookie {
    pub(crate) unsafe fn new(cookie: *const cef_cookie_t) -> Self {
        let cookie = cookie.as_ref().unwrap();
        let name = CefString::from_ptr(&cookie.name)
            .map(String::from)
            .unwrap_or_default();
        let value = CefString::from_ptr(&cookie.value)
            .map(String::from)
            .unwrap_or_default();
        let domain = CefString::from_ptr(&cookie.domain)
            .map(String::from)
            .unwrap_or_default();
        let path = CefString::from_ptr(&cookie.path)
            .map(String::from)
            .unwrap_or_default();
        Self {
            name,
            value,
            domain,
            path,
            secure: cookie.secure != 0,
            httponly: cookie.httponly != 0,
            creation: crate::values::cef_time_to_date_time(cookie.creation),
            last_access: crate::values::cef_time_to_date_time(cookie.last_access),
            expires: if cookie.has_expires != 0 {
                Some(crate::values::cef_time_to_date_time(cookie.expires))
            } else {
                None
            },
        }
    }
}

impl From<&'_ Cookie> for cef_cookie_t {
    fn from(cookie: &Cookie) -> cef_cookie_t {
        cef_cookie_t {
            name: CefString::new(&cookie.name).into_raw(),
            value: CefString::new(&cookie.value).into_raw(),
            domain: CefString::new(&cookie.domain).into_raw(),
            path: CefString::new(&cookie.path).into_raw(),
            secure: cookie.secure as c_int,
            httponly: cookie.httponly as c_int,
            creation: crate::values::date_time_to_cef_time(cookie.creation),
            last_access: crate::values::date_time_to_cef_time(cookie.last_access),
            has_expires: cookie.expires.is_some() as c_int,
            expires: cookie.expires.map(crate::values::date_time_to_cef_time).unwrap_or(unsafe{ std::mem::zeroed() }),
        }
    }
}

ref_counted_ptr!{
    pub struct CookieManager(*mut cef_cookie_manager_t);
}

ref_counted_ptr!{
    struct CookieVisitor(*mut cef_cookie_visitor_t);
}

ref_counted_ptr!{
    struct SetCookieCallback(*mut cef_set_cookie_callback_t);
}

ref_counted_ptr!{
    struct DeleteCookiesCallback(*mut cef_delete_cookies_callback_t);
}

impl CookieManager {
    /// Set the schemes supported by this manager. If `include_defaults` is `true`
    /// the default schemes ("http", "https", "ws" and "wss") will also be
    /// supported. Calling this function with an NULL `schemes` value and
    /// `include_defaults` set to `false` will disable all loading and saving of
    /// cookies for this manager. If `callback` is non-NULL it will be executed
    /// asnychronously on the UI thread after the change has been applied. Must be
    /// called before any cookies are accessed.
    pub fn set_supported_schemes<'a>(
        &self,
        schemes: impl IntoIterator<Item=&'a str>,
        include_defaults: bool,
        on_completion: impl 'static + Send + FnOnce()
    ) {
        let mut string_list = schemes.into_iter().collect::<CefStringList>();
        unsafe {
            self.0.set_supported_schemes.unwrap()(
                self.as_ptr(),
                string_list.as_mut_ptr(),
                include_defaults as c_int,
                CompletionCallback::new(on_completion).into_raw()
            )
        }
    }
    /// Visit all cookies on the UI thread. The returned cookies are ordered by
    /// longest path, then by earliest creation date. Returns `false` if cookies
    /// cannot be accessed.
    pub fn visit_all_cookies(
        &self,
        visitor: impl CookieVisitorFn,
    ) -> bool {
        unsafe {
            self.0.visit_all_cookies.unwrap()(
                self.as_ptr(),
                CookieVisitor::new(visitor).into_raw()
            ) != 0
        }
    }
    /// Visit a subset of cookies on the UI thread. The results are filtered by the
    /// given url scheme, host, domain and path. If `include_http_only` is `true`
    /// HTTP-only cookies will also be included in the results. The returned
    /// cookies are ordered by longest path, then by earliest creation date.
    /// Returns `false` if cookies cannot be accessed.
    pub fn visit_url_cookies(
        &self,
        url: &str,
        include_http_only: bool,
        visitor: impl CookieVisitorFn,
    ) -> bool {
        unsafe {
            self.0.visit_url_cookies.unwrap()(
                self.as_ptr(),
                CefString::new(url).as_ptr(),
                include_http_only as c_int,
                CookieVisitor::new(visitor).into_raw()
            ) != 0
        }
    }
    /// Sets a cookie given a valid URL and explicit user-provided cookie
    /// attributes. This function expects each attribute to be well-formed. It will
    /// check for disallowed characters (e.g. the ';' character is disallowed
    /// within the cookie value attribute) and fail without setting the cookie if
    /// such characters are found. If `callback` is non-NULL it will be executed
    /// asnychronously on the UI thread after the cookie has been set. Returns
    /// `false` if an invalid URL is specified or if cookies cannot be accessed.
    pub fn set_cookie(
        &self,
        url: &str,
        cookie: &Cookie,
        callback: impl 'static + Send + FnOnce(bool)
    ) -> bool {
        unsafe {
            self.0.set_cookie.unwrap()(
                self.as_ptr(),
                CefString::new(url).as_ptr(),
                &cookie.into(),
                SetCookieCallback::new(callback).into_raw(),
            ) != 0
        }
    }
    /// Delete all cookies that match the specified parameters. If both `url` and
    /// `cookie_name` values are specified all host and domain cookies matching
    /// both will be deleted. If only `url` is specified all host cookies (but not
    /// domain cookies) irrespective of path will be deleted. If `url` is NULL all
    /// cookies for all hosts and domains will be deleted. If `callback` is non-
    /// NULL it will be executed asnychronously on the UI thread after the cookies
    /// have been deleted. Returns `false` if a non-NULL invalid URL is specified
    /// or if cookies cannot be accessed. Cookies can alternately be deleted using
    /// the Visit*Cookies() functions.
    pub fn delete_cookies(
        &self,
        url: &str,
        cookie_name: &str,
        callback: impl 'static + Send + FnOnce(usize),
    ) -> bool {
        unsafe {
            self.0.delete_cookies.unwrap()(
                self.as_ptr(),
                CefString::new(url).as_ptr(),
                CefString::new(cookie_name).as_ptr(),
                DeleteCookiesCallback::new(callback).into_raw()
            ) != 0
        }
    }
    /// Flush the backing store (if any) to disk. If `callback` is non-NULL it will
    /// be executed asnychronously on the UI thread after the flush is complete.
    /// Returns `false` if cookies cannot be accessed.
    pub fn flush_store(
        &self,
        on_completion: impl 'static + Send + FnOnce()
    ) -> bool {
        unsafe {
            self.0.flush_store.unwrap()(
                self.as_ptr(),
                CompletionCallback::new(on_completion).into_raw()
            ) != 0
        }
    }
}

impl CookieVisitor {
    pub fn new(f: impl CookieVisitorFn) -> CookieVisitor {
        unsafe{ CookieVisitor::from_ptr_unchecked(CookieVisitorWrapper(Mutex::new(Some(Box::new(f)))).wrap().into_raw()) }
    }
}

struct CookieVisitorWrapper(Mutex<Option<Box<dyn CookieVisitorFn>>>);

impl Wrapper for CookieVisitorWrapper {
    type Cef = cef_cookie_visitor_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_cookie_visitor_t {
                base: unsafe { std::mem::zeroed() },
                visit: Some(Self::visit),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for CookieVisitorWrapper: cef_cookie_visitor_t {
        fn visit(
            &self,
            cookie: *const cef_cookie_t: *const cef_cookie_t,
            count: c_int: c_int,
            total: c_int: c_int,
            delete_cookie: &mut c_int: *mut c_int,
        ) -> c_int {
            let cookie = unsafe{ Cookie::new(cookie) };
            let mut delete_cookie_rs = *delete_cookie != 0;
            let ret = self.0.lock().take().unwrap()(CookieVisit {
                cookie,
                index: count as usize,
                len: total as usize,
                delete_cookie: &mut delete_cookie_rs
            }) as c_int;
            *delete_cookie = delete_cookie_rs as c_int;
            ret
        }
    }
}

impl SetCookieCallback {
    pub fn new(f: impl 'static + Send + FnOnce(bool)) -> SetCookieCallback {
        unsafe{ SetCookieCallback::from_ptr_unchecked(SetCookieCallbackWrapper(Mutex::new(Some(Box::new(f)))).wrap().into_raw()) }
    }
}

struct SetCookieCallbackWrapper(Mutex<Option<Box<dyn 'static + Send + FnOnce(bool)>>>);

impl Wrapper for SetCookieCallbackWrapper {
    type Cef = cef_set_cookie_callback_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_set_cookie_callback_t {
                base: unsafe { std::mem::zeroed() },
                on_complete: Some(Self::on_complete),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for SetCookieCallbackWrapper: cef_set_cookie_callback_t {
        fn on_complete(
            &self,
            success: bool: c_int,
        ) {
            self.0.lock().take().unwrap()(success)
        }
    }
}

impl DeleteCookiesCallback {
    pub fn new(f: impl 'static + Send + FnOnce(usize)) -> DeleteCookiesCallback {
        unsafe{ DeleteCookiesCallback::from_ptr_unchecked(DeleteCookiesCallbackWrapper(Mutex::new(Some(Box::new(f)))).wrap().into_raw()) }
    }
}

struct DeleteCookiesCallbackWrapper(Mutex<Option<Box<dyn 'static + Send + FnOnce(usize)>>>);

impl Wrapper for DeleteCookiesCallbackWrapper {
    type Cef = cef_delete_cookies_callback_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_delete_cookies_callback_t {
                base: unsafe { std::mem::zeroed() },
                on_complete: Some(Self::on_complete),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for DeleteCookiesCallbackWrapper: cef_delete_cookies_callback_t {
        fn on_complete(
            &self,
            num_deleted: c_int: c_int,
        ) {
            self.0.lock().take().unwrap()(num_deleted as usize)
        }
    }
}
