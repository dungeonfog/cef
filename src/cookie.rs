use cef_sys::{cef_cookie_t};
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub httponly: bool,
    pub creation: SystemTime,
    pub last_access: SystemTime,
    pub expires: Option<SystemTime>,
}
