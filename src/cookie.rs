use cef_sys::{cef_cookie_t, cef_time_to_doublet};
use std::time::{SystemTime, Duration};

use crate::{
    string::CefString,
};

#[derive(Clone, Debug)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub httponly: bool,
    pub creation: SystemTime,
    pub last_access: SystemTime,
    pub expires: Option<SystemTime>,
}

impl From<*const cef_cookie_t> for Cookie {
    fn from(cookie: *const cef_cookie_t) -> Self {
        let cookie = unsafe { cookie.as_ref() }.unwrap();
        let name = CefString::copy_raw_to_string(&cookie.name);
        let value = CefString::copy_raw_to_string(&cookie.value);
        let domain = CefString::copy_raw_to_string(&cookie.domain);
        let path = CefString::copy_raw_to_string(&cookie.path);
        let mut creation = 0.0;
        unsafe { cef_time_to_doublet(&cookie.creation, &mut creation) };
        let mut last_access = 0.0;
        unsafe { cef_time_to_doublet(&cookie.last_access, &mut last_access) };
        let mut expires = 0.0;
        if cookie.has_expires != 0 {
            unsafe { cef_time_to_doublet(&cookie.expires, &mut expires) };
        }

        Self {
            name: name.unwrap_or_default(),
            value: value.unwrap_or_default(),
            domain: domain.unwrap_or_default(),
            path: path.unwrap_or_default(),
            secure: cookie.secure != 0,
            httponly: cookie.httponly != 0,
            creation: SystemTime::UNIX_EPOCH + Duration::from_secs_f64(creation),
            last_access: SystemTime::UNIX_EPOCH + Duration::from_secs_f64(last_access),
            expires: if cookie.has_expires != 0 { Some(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(expires)) } else { None },
        }
    }
}
