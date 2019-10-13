use cef_sys::{cef_v8context_t, cef_v8exception_t, cef_v8stack_trace_t};

use crate::{
    client::Client,
};

ref_counted_ptr! {
    pub struct V8Context<C: Client>(*mut cef_v8context_t);
}

impl<C> V8Context<C> where C: Client {}

ref_counted_ptr! {
    pub struct V8Exception(*mut cef_v8exception_t);
}

impl V8Exception {
    /// Returns the exception message.
    pub fn get_message(&self) -> String {
        "".to_owned()
    }
}

ref_counted_ptr! {
    pub struct V8StackTrace(*mut cef_v8stack_trace_t);
}
