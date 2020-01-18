use cef_sys::{cef_process_id_t, cef_process_message_t, cef_string_userfree_utf16_free, cef_process_message_create};

use crate::{
    string::CefString,
    values::ListValue,
};

/// Existing process IDs.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessId {
    /// Browser process.
    Browser = cef_process_id_t::PID_BROWSER as isize,
    /// Renderer process.
    Renderer = cef_process_id_t::PID_RENDERER as isize,
}

impl ProcessId {
    pub unsafe fn from_unchecked(c: crate::CEnumType) -> Self {
        std::mem::transmute(c)
    }
}

ref_counted_ptr! {
    pub struct ProcessMessage(*mut cef_process_message_t);
}

/// Structure representing a message. Can be used on any process and thread.
impl ProcessMessage {
    pub fn new(name: &str) -> Self {
        unsafe {
            Self::from_ptr_unchecked(cef_process_message_create(CefString::from(name).as_ptr()))
        }
    }

    /// Returns true (1) if this object is valid. Do not call any other functions
    /// if this function returns false (0).
    pub fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.as_ptr()) } != 0)
            .unwrap_or(false)
    }
    /// Returns true if the values of this object are read-only. Some APIs may
    /// expose read-only objects.
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .map(|is_read_only| unsafe { is_read_only(self.as_ptr()) } != 0)
            .unwrap_or(true)
    }
    /// Returns the message name.
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
    /// Returns the list of arguments.
    pub fn get_argument_list(&self) -> ListValue {
        unsafe { ListValue::from_ptr_unchecked(self.0.get_argument_list.unwrap()(self.as_ptr())) }
    }
}

impl crate::cef_helper_traits::DeepClone for ProcessMessage {
    /// Returns a writable copy of this object.
    fn deep_clone(&self) -> Self {
        unsafe { Self::from_ptr_unchecked((self.0.copy.unwrap())(self.as_ptr())) }
    }
}
