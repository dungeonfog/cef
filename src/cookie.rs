use cef_sys::{cef_cookie_t, cef_time_to_doublet};
use std::time::{Duration, SystemTime};

use crate::string::CefString;

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
    pub creation: SystemTime,
    /// The cookie last access date. This is automatically populated by the system
    /// on access.
    pub last_access: SystemTime,
    /// The cookie expiration date.
    pub expires: Option<SystemTime>,
}

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
        let mut creation = 0.0;
        cef_time_to_doublet(&cookie.creation, &mut creation);
        let mut last_access = 0.0;
        cef_time_to_doublet(&cookie.last_access, &mut last_access);
        let mut expires = 0.0;
        if cookie.has_expires != 0 {
            cef_time_to_doublet(&cookie.expires, &mut expires);
        }

        Self {
            name,
            value,
            domain,
            path,
            secure: cookie.secure != 0,
            httponly: cookie.httponly != 0,
            creation: SystemTime::UNIX_EPOCH + Duration::from_secs_f64(creation),
            last_access: SystemTime::UNIX_EPOCH + Duration::from_secs_f64(last_access),
            expires: if cookie.has_expires != 0 {
                Some(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(expires))
            } else {
                None
            },
        }
    }
}
