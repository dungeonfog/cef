#![allow(dead_code, unused_variables)]
#![feature(fn_traits, unboxed_closures, trait_alias)]
#[macro_use]
mod refcounted;
#[macro_use]
mod extern_callback_helpers;
mod cef_helper_traits;
mod ptr_hash;
mod misc_fns;
pub use misc_fns::*;

pub mod string;
pub mod multimap;
pub mod values;
pub mod scheme_registrar;
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

#[cfg(target_os = "windows")]
pub mod sandbox;
#[cfg(target_os = "windows")]
pub mod main_args;
pub mod settings;
pub mod color;
pub mod events;
pub mod drag;
pub mod file_dialog;
pub mod printing;
pub mod window;
use num_enum::UnsafeFromPrimitive;
pub mod ime;
pub mod navigation;
pub mod extension;
pub mod request_handler;
pub mod ssl;
pub mod task;

/// Return value types.
#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum ReturnValue {
    /// Cancel immediately.
    Cancel = cef_sys::cef_return_value_t::RV_CANCEL as i32,
    /// Continue immediately.
    Continue = cef_sys::cef_return_value_t::RV_CONTINUE as i32,
    /// Continue asynchronously (usually via a callback).
    ContinueAsync = cef_sys::cef_return_value_t::RV_CONTINUE_ASYNC as i32,
}


