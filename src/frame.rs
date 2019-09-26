use cef_sys::{cef_frame_t};

use crate::{
    string::{StringVisitor, StringVisitorWrapper},
    request::Request,
};

/// Structure used to represent a frame in the browser window. When used in the
/// browser process the functions of this structure may be called on any thread
/// unless otherwise indicated in the comments. When used in the render process
/// the functions of this structure may only be called on the main thread.
pub struct Frame(*mut cef_frame_t);

unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl std::convert::AsRef<cef_frame_t> for Frame {
    fn as_ref(&self) -> &cef_frame_t {
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl From<*mut cef_frame_t> for Frame {
    fn from(frame: *mut cef_frame_t) -> Self {
        unsafe { ((*frame).base.add_ref.unwrap())(&mut (*frame).base); }
        Self(frame)
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe { (self.as_ref().base.release.unwrap())(&mut (*self.0).base); }
    }
}

impl Frame {
    /// True if this object is currently attached to a valid frame.
    pub fn is_valid(&self) -> bool {
        self.as_ref().is_valid.and_then(|is_valid| Some(unsafe { is_valid(self.0) } != 0)).unwrap_or(false)
    }
    /// Execute undo in this frame.
    pub fn undo(&self) {
        if let Some(undo) = self.as_ref().undo {
            unsafe { undo(self.0); }
        }
    }
    /// Execute redo in this frame.
    pub fn redo(&self) {
        if let Some(redo) = self.as_ref().redo {
            unsafe { redo(self.0); }
        }
    }
    /// Execute cut in this frame.
    pub fn cut(&self) {
        if let Some(cut) = self.as_ref().cut {
            unsafe { cut(self.0); }
        }
    }
    /// Execute copy in this frame.
    pub fn copy(&self) {
        if let Some(copy) = self.as_ref().copy {
            unsafe { copy(self.0); }
        }
    }
    /// Execute paste in this frame.
    pub fn paste(&self) {
        if let Some(paste) = self.as_ref().paste {
            unsafe { paste(self.0); }
        }
    }
    /// Execute delete in this frame.
    pub fn del(&self) {
        if let Some(del) = self.as_ref().del {
            unsafe { del(self.0); }
        }
    }
    /// Execute select all in this frame.
    pub fn select_all(&self) {
        if let Some(select_all) = self.as_ref().select_all {
            unsafe { select_all(self.0); }
        }
    }
    /// Save this frame's HTML source to a temporary file and open it in the
    /// default text viewing application. This function can only be called from the
    /// browser process.
    pub fn view_source(&self) {
        if let Some(view_source) = self.as_ref().view_source {
            unsafe { view_source(self.0); }
        }
    }
    /// Retrieve this frame's HTML source as a string sent to the specified
    /// visitor.
    pub fn get_source(&self, visitor: Box<dyn StringVisitor>) {
        if let Some(get_source) = self.as_ref().get_source {
            let visitor = StringVisitorWrapper::wrap(visitor);
            unsafe { get_source(self.0, visitor); }
        }
    }
    /// Retrieve this frame's display text as a string sent to the specified
    /// visitor.
    pub fn get_text(&self, visitor: Box<dyn StringVisitor>) {
        if let Some(get_text) = self.as_ref().get_text {
            let visitor = StringVisitorWrapper::wrap(visitor);
            unsafe { get_text(self.0, visitor); }
        }
    }
    /// Load the request represented by the |request| object.
    pub fn load_request(&self, request: &Request) {
        if let Some(load_request) = self.as_ref().load_request {
            unsafe { load_request(self.0, request.as_ptr()); }
        }
    }

    // TODO: to be continued…
}
