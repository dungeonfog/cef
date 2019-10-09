use cef_sys::{cef_v8context_t, cef_v8exception_t, cef_v8stack_trace_t};

pub struct V8Context(*mut cef_v8context_t);

impl V8Context {}

#[doc(hidden)]
impl From<*mut cef_v8context_t> for V8Context {
    fn from(context: *mut cef_v8context_t) -> Self {
        unsafe {
            ((*context).base.add_ref.unwrap())(&mut (*context).base);
        }
        Self(context)
    }
}

impl Drop for V8Context {
    fn drop(&mut self) {
        unsafe {
            ((*self.0).base.release.unwrap())(&mut (*self.0).base);
        }
    }
}

pub struct V8Exception(*mut cef_v8exception_t);

impl V8Exception {
    /// Returns the exception message.
    pub fn get_message(&self) -> String {
        "".to_owned()
    }
}

#[doc(hidden)]
impl From<*mut cef_v8exception_t> for V8Exception {
    fn from(exception: *mut cef_v8exception_t) -> Self {
        unsafe {
            ((*exception).base.add_ref.unwrap())(&mut (*exception).base);
        }
        Self(exception)
    }
}

impl Drop for V8Exception {
    fn drop(&mut self) {
        unsafe {
            ((*self.0).base.release.unwrap())(&mut (*self.0).base);
        }
    }
}

pub struct V8StackTrace(*mut cef_v8stack_trace_t);

#[doc(hidden)]
impl From<*mut cef_v8stack_trace_t> for V8StackTrace {
    fn from(trace: *mut cef_v8stack_trace_t) -> Self {
        unsafe {
            ((*trace).base.add_ref.unwrap())(&mut (*trace).base);
        }
        Self(trace)
    }
}

impl Drop for V8StackTrace {
    fn drop(&mut self) {
        unsafe {
            ((*self.0).base.release.unwrap())(&mut (*self.0).base);
        }
    }
}
