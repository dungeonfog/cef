use cef_sys::cef_callback_t;

/// Generic callback structure used for asynchronous continuation.
pub struct Callback(*mut cef_callback_t);

impl Callback {
    /// Continue processing.
    pub fn cont(&self) {
        unsafe {
            (&*self.0).cont.unwrap()(self.0);
        }
    }
    /// Cancel processing.
    pub fn cancel(&self) {
        unsafe {
            (&*self.0).cancel.unwrap()(self.0);
        }
    }
}

#[doc(hidden)]
impl From<*mut cef_callback_t> for Callback {
    fn from(cb: *mut cef_callback_t) -> Self {
        unsafe {
            ((*cb).base.add_ref.unwrap())(&mut (*cb).base);
        }
        Self(cb)
    }
}

impl Drop for Callback {
    fn drop(&mut self) {
        unsafe {
            ((*self.0).base.release.unwrap())(&mut (*self.0).base);
        }
    }
}
