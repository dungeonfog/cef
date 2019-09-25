use cef_sys::{_cef_v8context_t, _cef_v8exception_t, _cef_v8stack_trace_t};

pub struct V8Context(*mut _cef_v8context_t);

impl V8Context {
}

impl From<*mut _cef_v8context_t> for V8Context {
    fn from(context: *mut _cef_v8context_t) -> Self {
        unsafe { ((*context).base.add_ref.unwrap())(&mut (*context).base); }
        Self(context)
    }
}

impl Drop for V8Context {
    fn drop(&mut self) {
        unsafe { ((*self.0).base.release.unwrap())(&mut (*self.0).base); }
    }
}

pub struct V8Exception(*mut _cef_v8exception_t);

impl V8Exception {
    /// Returns the exception message.
    pub fn get_message(&self) -> String {
        "".to_owned()
    }

}

impl From<*mut _cef_v8exception_t> for V8Exception {
    fn from(exception: *mut _cef_v8exception_t) -> Self {
        unsafe { ((*exception).base.add_ref.unwrap())(&mut (*exception).base); }
        Self(exception)
    }
}

impl Drop for V8Exception {
    fn drop(&mut self) {
        unsafe { ((*self.0).base.release.unwrap())(&mut (*self.0).base); }
    }
}

pub struct V8StackTrace(*mut _cef_v8stack_trace_t);

impl From<*mut _cef_v8stack_trace_t> for V8StackTrace {
    fn from(trace: *mut _cef_v8stack_trace_t) -> Self {
        unsafe { ((*trace).base.add_ref.unwrap())(&mut (*trace).base); }
        Self(trace)
    }
}

impl Drop for V8StackTrace {
    fn drop(&mut self) {
        unsafe { ((*self.0).base.release.unwrap())(&mut (*self.0).base); }
    }
}
