use cef_sys::{cef_browser_t};

use crate::{
    browser_host::BrowserHost,
    frame::Frame,
    string::{CefString, CefStringList},
};

/// Structure used to represent a browser window. When used in the browser
/// process the functions of this structure may be called on any thread unless
/// otherwise indicated in the comments. When used in the render process the
/// functions of this structure may only be called on the main thread.
#[derive(Eq)]
pub struct Browser(*mut cef_browser_t);

unsafe impl Send for Browser {}
unsafe impl Sync for Browser {}

impl Browser {
    /// Returns the browser host object. This function can only be called in the
    /// browser process.
    pub fn get_host(&self) -> BrowserHost {
        BrowserHost::from(unsafe { (self.as_ref().get_host.unwrap())(self.0) })
    }
    /// Returns true if the browser can navigate backwards.
    pub fn can_go_back(&self) -> bool {
        unsafe { (self.as_ref().can_go_back.unwrap())(self.0) != 0 }
    }
    /// Navigate backwards.
    pub fn go_back(&mut self) {
        unsafe { (self.as_ref().go_back.unwrap())(self.0); }
    }
    /// Returns true if the browser can navigate forwards.
    pub fn can_go_forward(&self) -> bool {
        unsafe { (self.as_ref().can_go_forward.unwrap())(self.0) != 0 }
    }
    /// Navigate forwards.
    pub fn go_forward(&mut self) {
        unsafe { (self.as_ref().go_forward.unwrap())(self.0); }
    }
    /// Returns true if the browser is currently loading.
    pub fn is_loading(&self) -> bool {
        unsafe { (self.as_ref().is_loading.unwrap())(self.0) != 0 }
    }
    /// Reload the current page, optionally ignoring any cached data.
    pub fn reload(&mut self, ignore_cache: bool) {
        if ignore_cache {
            unsafe { (self.as_ref().reload.unwrap())(self.0); }
        } else {
            unsafe { (self.as_ref().reload_ignore_cache.unwrap())(self.0); }
        }
    }
    /// Stop loading the page.
    pub fn stop_load(&mut self) {
        unsafe { (self.as_ref().stop_load.unwrap())(self.0); }
    }
    /// Returns the globally unique identifier for this browser. This value is also
    /// used as the tabId for extension APIs.
    pub fn get_identifier(&self) -> i32 {
        unsafe { (self.as_ref().get_identifier.unwrap())(self.0) }
    }
    /// Returns true if the window is a popup window.
    pub fn is_popup(&self) -> bool {
        unsafe { (self.as_ref().is_popup.unwrap())(self.0) != 0 }
    }
    /// Returns true if a document has been loaded in the browser.
    pub fn has_document(&self) -> bool {
        unsafe { (self.as_ref().has_document.unwrap())(self.0) != 0 }
    }
    /// Returns the main (top-level) frame for the browser window.
    pub fn get_main_frame(&self) -> Frame {
        Frame::from(unsafe { (self.as_ref().get_main_frame.unwrap())(self.0) })
    }
    /// Returns the focused frame for the browser window.
    pub fn get_focused_frame(&self) -> Option<Frame> {
        let frame = unsafe { (self.as_ref().get_focused_frame.unwrap())(self.0) };
        if frame.is_null() {
            None
        } else {
            Some(Frame::from(frame))
        }
    }
    /// Returns the frame with the specified identifier, or None if not found.
    pub fn get_frame_byident(&self, identifier: i64) -> Option<Frame> {
        let frame = unsafe { (self.as_ref().get_frame_byident.unwrap())(self.0, identifier) };
        if frame.is_null() {
            None
        } else {
            Some(Frame::from(frame))
        }
    }
    /// Returns the frame with the specified name, or None if not found.
    pub fn get_frame(&self, name: &str) -> Option<Frame> {
        let frame = unsafe { (self.as_ref().get_frame.unwrap())(self.0, CefString::new(name).as_ref()) };
        if frame.is_null() {
            None
        } else {
            Some(Frame::from(frame))
        }
    }
    /// Returns the number of frames that currently exist.
    pub fn get_frame_count(&self) -> usize {
        unsafe { (self.as_ref().get_frame_count.unwrap())(self.0) }
    }
    /// Returns the identifiers of all existing frames.
    pub fn get_frame_identifiers(&self) -> Vec<i64> {
        let mut count = self.get_frame_count();
        let mut result = vec![0; count];
        unsafe { (self.as_ref().get_frame_identifiers.unwrap())(self.0, &mut count, result.as_mut_ptr()); }
        result
    }
    /// Returns the names of all existing frames.
    pub fn get_frame_names(&self) -> Vec<String> {
        let mut list = CefStringList::default();
        unsafe { (self.as_ref().get_frame_names.unwrap())(self.0, list.get()); }
        list.into()
    }
}

impl std::convert::AsRef<cef_browser_t> for Browser {
    fn as_ref(&self) -> &cef_browser_t {
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl From<*mut cef_browser_t> for Browser {
    fn from(browser: *mut cef_browser_t) -> Self {
        unsafe { ((*browser).base.add_ref.unwrap())(&mut (*browser).base); }
        Self(browser)
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        unsafe { (self.as_ref().base.release.unwrap())(&mut (*self.0).base); }
    }
}

impl PartialEq for Browser {
    /// Returns true if this object is pointing to the same handle as `that`
    /// object.
    fn eq(&self, that: &Self) -> bool {
        unsafe { (self.as_ref().is_same.unwrap())(self.0, that.0) != 0 }
    }
}
