use cef_sys::{cef_process_id_t, cef_process_message_t, cef_string_userfree_utf16_free};
use num_enum::UnsafeFromPrimitive;

use crate::{
    string::CefString,
    values::{ListValue, StoredValue},
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

ref_counted_ptr! {
    pub struct ProcessMessage(*mut cef_process_message_t);
}

impl ProcessMessage {
    pub fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.as_ptr()) } != 0)
            .unwrap_or(false)
    }
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .map(|is_read_only| unsafe { is_read_only(self.as_ptr()) } != 0)
            .unwrap_or(true)
    }
    pub fn get_name(&self) -> Option<String> {
        self.0
            .get_name
            .and_then(|get_name| unsafe { get_name(self.as_ptr()).as_mut() })
            .map(|cef_string| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(cef_string));
                cef_string_userfree_utf16_free(cef_string);
                s
            })
    }
    pub fn get_argument_list(&self) -> Vec<StoredValue> {
        unsafe { ListValue::from_ptr_unchecked(self.0.get_argument_list.unwrap()(self.as_ptr())) }
            .into()
    }
}

impl crate::cef_helper_traits::DeepClone for ProcessMessage {
    fn deep_clone(&self) -> Self {
        unsafe { Self::from_ptr_unchecked((self.0.copy.unwrap())(self.as_ptr())) }
    }
}
