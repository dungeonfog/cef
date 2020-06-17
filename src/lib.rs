#![allow(dead_code, unused_variables)]
#![feature(fn_traits, unboxed_closures, trait_alias, optin_builtin_traits)]
#[macro_use]
mod refcounted;
#[macro_use]
mod extern_callback_helpers;
pub mod helper_traits;
mod ptr_hash;
mod misc_fns;
pub use misc_fns::*;

pub mod accessibility_handler;
pub mod string;
pub mod multimap;
pub mod values;
pub mod scheme_registrar;
pub mod resource_bundle;
pub mod resource_bundle_handler;
pub mod browser_process_handler;
pub mod browser;
pub mod browser_host;
pub mod frame;
pub mod load_handler;
pub mod render_process_handler;
pub mod dom;
pub mod v8context;
pub mod process;
pub mod request;
pub mod response;
pub mod url_request;
pub mod request_context;
pub mod web_plugin;
pub mod cookie;
pub mod callback;
pub mod resource_request_handler;
pub mod client;
pub mod image;

pub mod command_line;
pub mod app;

pub mod sandbox;
mod main_args;
pub mod scheme;
pub mod settings;
pub mod color;
pub mod events;
pub mod drag;
pub mod file_dialog;
pub mod printing;
pub mod window;
pub mod x509_certificate;
pub mod ime;
pub mod navigation;
pub mod extension;
pub mod stream;
pub mod ssl;
pub mod task;
pub mod logging;
mod send_protector;

/// Return value types.
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ReturnValue {
    /// Cancel immediately.
    Cancel = cef_sys::cef_return_value_t::RV_CANCEL as isize,
    /// Continue immediately.
    Continue = cef_sys::cef_return_value_t::RV_CONTINUE as isize,
    /// Continue asynchronously (usually via a callback).
    ContinueAsync = cef_sys::cef_return_value_t::RV_CONTINUE_ASYNC as isize,
}

impl ReturnValue {
    pub unsafe fn from_unchecked(c: crate::CEnumType) -> Self {
        std::mem::transmute(c)
    }
}

#[cfg(target_os = "windows")]
pub type CEnumType = i32;
#[cfg(target_os = "linux")]
pub type CEnumType = u32;
#[cfg(target_os = "macos")]
pub type CEnumType = u32;
