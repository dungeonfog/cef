use cef_sys::cef_callback_t;

ref_counted_ptr! {
    /// Generic callback structure used for asynchronous continuation.
    pub struct Callback(*mut cef_callback_t);
}

impl Callback {
    /// Continue processing.
    pub fn cont(&self) {
        unsafe {
            self.0.cont.unwrap()(self.as_ptr());
        }
    }
    /// Cancel processing.
    pub fn cancel(&self) {
        unsafe {
            self.0.cancel.unwrap()(self.as_ptr());
        }
    }
}
