use parking_lot::Mutex;
use cef_sys::{cef_callback_t, cef_completion_callback_t};
use crate::refcounted::{RefCountedPtr, Wrapper};

ref_counted_ptr! {
    /// Generic callback structure used for asynchronous continuation.
    pub struct Callback(*mut cef_callback_t);
}

ref_counted_ptr! {
    /// Generic callback structure used for asynchronous continuation.
    pub(crate) struct CompletionCallback(*mut cef_completion_callback_t);
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

impl CompletionCallback {
    pub fn new(f: impl 'static + Send + FnOnce()) -> CompletionCallback {
        unsafe{ CompletionCallback::from_ptr_unchecked(CompletionCallbackWrapper(Mutex::new(Some(Box::new(f)))).wrap().into_raw()) }
    }
    pub fn on_complete(&self) {
        unsafe {
            self.0.on_complete.unwrap()(self.as_ptr());
        }
    }
}

struct CompletionCallbackWrapper(Mutex<Option<Box<dyn 'static + Send + FnOnce()>>>);

impl Wrapper for CompletionCallbackWrapper {
    type Cef = cef_completion_callback_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_completion_callback_t {
                base: unsafe { std::mem::zeroed() },
                on_complete: Some(Self::on_complete),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for CompletionCallbackWrapper: cef_completion_callback_t {
        fn on_complete(&self) {
            self.0.lock().take().unwrap()()
        }
    }
}
