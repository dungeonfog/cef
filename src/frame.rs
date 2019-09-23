use cef_sys::{cef_frame_t};

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
