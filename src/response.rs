use cef_sys::{cef_response_t, cef_response_create, cef_string_userfree_utf16_free};
use std::collections::HashMap;

use crate::{
    load_handler::ErrorCode,
    string::CefString,
    multimap::MultiMap,
};

ref_counted_ptr! {
    /// Structure used to represent a web response. The functions of this structure
    /// may be called on any thread.
    pub struct Response(*mut cef_response_t);
}

impl Response {
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .map(|is_read_only| unsafe { is_read_only(self.0.as_ptr()) != 0 })
            .unwrap_or(true)
        }
    pub fn get_error(&self) -> ErrorCode {
        self.0
            .get_error
            .map(|get_error| unsafe { ErrorCode::from_unchecked(get_error(self.0.as_ptr())) })
            .unwrap_or(ErrorCode::Failed)
    }
    pub fn set_error(&self, error: ErrorCode) {
        if let Some(set_error) = self.0.set_error {
            unsafe {
                set_error(self.0.as_ptr(), error as _);
            }
        }
    }
    pub fn get_status(&self) -> i32 {
        self.0
            .get_status
            .map(|get_status| unsafe { get_status(self.0.as_ptr()) } as _)
            .unwrap_or(0)
    }
    pub fn set_status(&self, status: i32) {
        if let Some(set_status) = self.0.set_status {
            unsafe {
                set_status(self.0.as_ptr(), status as _);
            }
        }
    }
    pub fn get_status_text(&self) -> String {
        self.0
            .get_status_text
            .and_then(|get_status_text| unsafe { get_status_text(self.as_ptr()).as_mut() })
            .map(|status_text| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(status_text));
                cef_string_userfree_utf16_free(status_text);
                s
            })
            .unwrap_or_default()
    }
    pub fn set_status_text(&self, status_text: &str) {
        if let Some(set_status_text) = self.0.set_status_text {
            unsafe {
                set_status_text(self.0.as_ptr(), CefString::new(status_text).as_ptr());
            }
        }
    }
    pub fn get_mime_type(&self) -> String {
        self.0
            .get_mime_type
            .and_then(|get_mime_type| unsafe { get_mime_type(self.as_ptr()).as_mut() })
            .map(|mime_type| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(mime_type));
                cef_string_userfree_utf16_free(mime_type);
                s
            })
            .unwrap_or_default()
    }
    pub fn set_mime_type(&self, mime_type: &str) {
        if let Some(set_mime_type) = self.0.set_mime_type {
            unsafe {
                set_mime_type(self.0.as_ptr(), CefString::new(mime_type).as_ptr());
            }
        }
    }
    pub fn get_charset(&self) -> String {
        self.0
            .get_charset
            .and_then(|get_charset| unsafe { get_charset(self.as_ptr()).as_mut() })
            .map(|charset| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(charset));
                cef_string_userfree_utf16_free(charset);
                s
            })
            .unwrap_or_default()
    }
    pub fn set_charset(&self, charset: &str) {
        if let Some(set_charset) = self.0.set_charset {
            unsafe {
                set_charset(self.0.as_ptr(), CefString::new(charset).as_ptr());
            }
        }
    }
    pub fn get_header_by_name(&self, name: &str) -> String {
        self.0
            .get_header_by_name
            .and_then(|get_header_by_name| unsafe { get_header_by_name(self.as_ptr(), CefString::new(name).as_ptr()).as_mut() })
            .map(|value| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(value));
                cef_string_userfree_utf16_free(value);
                s
            })
            .unwrap_or_default()
    }
    pub fn set_header_by_name(&self, name: &str, value: &str, overwrite: bool) {
        if let Some(set_header_by_name) = self.0.set_header_by_name {
            unsafe {
                set_header_by_name(self.0.as_ptr(), CefString::new(name).as_ptr(), CefString::new(value).as_ptr(), overwrite as _);
            }
        }
    }
    pub fn get_header_map(&self) -> HashMap<String, Vec<String>> {
        self.0
            .get_header_map
            .map(|get_header_map| {
                let map = MultiMap::new();
                unsafe { get_header_map(self.as_ptr(), map.as_ptr()) };
                <MultiMap as Into<HashMap<String, Vec<String>>>>::into(map)
            })
            .unwrap_or_else(HashMap::new)
    }
    pub fn set_header_map(&self, header_map: &HashMap<String, Vec<String>>) {
        let map: MultiMap = header_map.into();
        self.0
            .set_header_map
            .map(|set_header_map| unsafe { set_header_map(self.as_ptr(), map.as_ptr()); });
    }
    pub fn get_url(&self) -> String {
        self.0
            .get_url
            .and_then(|get_url| unsafe { get_url(self.as_ptr()).as_mut() })
            .map(|url| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(url));
                cef_string_userfree_utf16_free(url);
                s
            })
            .unwrap_or_default()
    }
    pub fn set_url(&self, url: &str) {
        if let Some(set_url) = self.0.set_url {
            unsafe {
                set_url(self.0.as_ptr(), CefString::new(url).as_ptr());
            }
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        unsafe {
            Self::from_ptr_unchecked(cef_response_create())
        }
    }
}
