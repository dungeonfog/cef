use cef_sys::{cef_callback_t};

pub struct Callback(*mut cef_callback_t);

impl Callback {
    pub fn cont(&self) {
        unsafe { (&*self.0).cont.unwrap()(self.0); }
    }
    pub fn cancel(&self) {
        unsafe { (&*self.0).cancel.unwrap()(self.0); }
    }
}

impl From<*mut cef_callback_t> for Callback {
    fn from(cb: *mut cef_callback_t) -> Self {
        unsafe { ((*cb).base.add_ref.unwrap())(&mut (*cb).base); }
        Self(cb)
    }
}

impl Drop for Callback {
    fn drop(&mut self) {
        unsafe { ((*self.0).base.release.unwrap())(&mut (*self.0).base); }
    }
}
