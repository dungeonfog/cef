use cef_sys::{cef_frame_t};


/// Structure used to represent a frame in the browser window. When used in the
/// browser process the functions of this structure may be called on any thread
/// unless otherwise indicated in the comments. When used in the render process
/// the functions of this structure may only be called on the main thread.
pub struct Frame(*mut cef_frame_t);

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
