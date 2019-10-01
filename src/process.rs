use cef_sys::{cef_process_message_t, cef_process_id_t, cef_string_userfree_utf16_free};
use num_enum::UnsafeFromPrimitive;

use crate::{
    values::{StoredValue, ListValue},
    string::CefString,
};

/// Existing process IDs.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum ProcessId {
    /// Browser process.
    Browser = cef_process_id_t::PID_BROWSER as i32,
    /// Renderer process.
    Renderer = cef_process_id_t::PID_RENDERER as i32,
}

pub struct ProcessMessage(*mut cef_process_message_t);

impl ProcessMessage {
    pub fn is_valid(&self) -> bool {
        if let Some(is_valid) = unsafe { (*self.0).is_valid } {
            unsafe { is_valid(self.0) != 0 }
        } else { false }
    }
    pub fn is_read_only(&self) -> bool {
        if let Some(is_read_only) = unsafe { (*self.0).is_read_only } {
            unsafe { is_read_only(self.0) != 0 }
        } else { true }
    }
    pub fn get_name(&self) -> Option<String> {
        if let Some(get_name) = unsafe { (*self.0).get_name } {
            let name = unsafe { get_name(self.0) };
            let s = CefString::copy_raw_to_string(name);
            unsafe { cef_string_userfree_utf16_free(name); }
            s
        } else {
            None
        }
    }
    pub fn get_argument_list(&self) -> Vec<StoredValue> {
        ListValue::from(unsafe { (*self.0).get_argument_list.unwrap()(self.0) }).into()
    }
}

impl Clone for ProcessMessage {
    fn clone(&self) -> Self {
        Self(unsafe { ((*self.0).copy.unwrap())(self.0) })
    }
}

#[doc(hidden)]
impl From<*mut cef_process_message_t> for ProcessMessage {
    fn from(msg: *mut cef_process_message_t) -> Self {
        unsafe { ((*msg).base.add_ref.unwrap())(&mut (*msg).base); }
        Self(msg)
    }
}

impl Drop for ProcessMessage {
    fn drop(&mut self) {
        unsafe { ((*self.0).base.release.unwrap())(&mut (*self.0).base); }
    }
}
