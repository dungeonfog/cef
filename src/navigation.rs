use cef_sys::cef_navigation_entry_t;
use std::time::SystemTime;

use crate::{
    load_handler::TransitionType,
};

ref_counted_ptr! {
    /// Structure used to represent an entry in navigation history.
    pub struct NavigationEntry(*mut cef_navigation_entry_t);
}

impl NavigationEntry {
    pub fn is_valid(&self) -> bool {
        unimplemented!()
    }
    pub fn get_url(&self) -> String {
        unimplemented!()
    }
    pub fn get_display_url(&self) -> String {
        unimplemented!()
    }
    pub fn get_original_url(&self) -> String {
        unimplemented!()
    }
    pub fn get_title(&self) -> String {
        unimplemented!()
    }
    pub fn get_transition_type(&self) -> TransitionType {
        unimplemented!()
    }
    pub fn has_post_data(&self) -> bool {
        unimplemented!()
    }
    pub fn get_completion_time(&self) -> SystemTime {
        unimplemented!()
    }
    pub fn get_http_status_code(&self) -> u16 {
        unimplemented!()
    }
    // pub fn get_sslstatus(&self) -> SSLStatus {
    //     unimplemented!()
    // }
}
