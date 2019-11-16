use cef_sys::{cef_navigation_entry_t};
use std::convert::TryInto;
use crate::{
    load_handler::TransitionType,
    ssl::SSLStatus,
    string::CefString,
};
use chrono::{DateTime, Utc};

ref_counted_ptr! {
    /// Structure used to represent an entry in navigation history.
    pub struct NavigationEntry(*mut cef_navigation_entry_t);
}

impl NavigationEntry {
    pub fn is_valid(&self) -> bool {
        unsafe{ self.0.is_valid.unwrap()(self.as_ptr()) != 0 }
    }
    pub fn get_url(&self) -> String {
        unsafe{ CefString::from_userfree_unchecked(self.0.get_url.unwrap()(self.as_ptr())).into() }
    }
    pub fn get_display_url(&self) -> String {
        unsafe{ CefString::from_userfree_unchecked(self.0.get_display_url.unwrap()(self.as_ptr())).into() }
    }
    pub fn get_original_url(&self) -> String {
        unsafe{ CefString::from_userfree_unchecked(self.0.get_original_url.unwrap()(self.as_ptr())).into() }
    }
    pub fn get_title(&self) -> String {
        unsafe{ CefString::from_userfree_unchecked(self.0.get_title.unwrap()(self.as_ptr())).into() }
    }
    pub fn get_transition_type(&self) -> TransitionType {
        unsafe{ self.0.get_transition_type.unwrap()(self.as_ptr()).0.try_into().unwrap() }
    }
    pub fn has_post_data(&self) -> bool {
        unsafe{ self.0.has_post_data.unwrap()(self.as_ptr()) != 0 }
    }
    pub fn get_completion_time(&self) -> DateTime<Utc> {
        crate::values::cef_time_to_date_time(unsafe{ self.0.get_completion_time.unwrap()(self.as_ptr()) })
    }
    pub fn get_http_status_code(&self) -> u16 {
        unsafe{ self.0.get_http_status_code.unwrap()(self.as_ptr()) as u16 }
    }
    pub fn get_ssl_status(&self) -> SSLStatus {
        unsafe{ SSLStatus::from_ptr_unchecked(self.0.get_sslstatus.unwrap()(self.as_ptr())) }
    }
}
