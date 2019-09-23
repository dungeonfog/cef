use cef_sys::{cef_browser_host_t};

pub struct BrowserHost(*mut cef_browser_host_t);

impl std::convert::AsRef<cef_browser_host_t> for BrowserHost {
    fn as_ref(&self) -> &cef_browser_host_t {
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl From<*mut cef_browser_host_t> for BrowserHost {
    fn from(browser_host: *mut cef_browser_host_t) -> Self {
        unsafe { ((*browser_host).base.add_ref.unwrap())(&mut (*browser_host).base); }
        Self(browser_host)
    }
}

impl Drop for BrowserHost {
    fn drop(&mut self) {
        unsafe { (self.as_ref().base.release.unwrap())(&mut (*self.0).base); }
    }
}
