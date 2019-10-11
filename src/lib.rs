#![allow(unused)]
#![feature(fn_traits)]
#[macro_use]
mod refcounted;
#[macro_use]
mod extern_callback_helpers;
mod ptr_hash;
mod string;
pub use string::StringVisitor;
mod multimap;
mod values;
pub use values::StoredValue;
mod scheme_registrar;
pub use scheme_registrar::{SchemeOptions, SchemeRegistrar};
mod resource_bundle_handler;
pub use resource_bundle_handler::{ResourceBundleHandler, ScaleFactor};
mod browser_process_handler;
pub use browser_process_handler::BrowserProcessHandler;
mod browser;
pub use browser::{Browser, BrowserSettings, State};
mod browser_host;
pub use browser_host::{BrowserHost, WindowHandle};
mod frame;
pub use frame::Frame;
mod load_handler;
pub use load_handler::LoadHandler;
mod render_process_handler;
pub use render_process_handler::RenderProcessHandler;
mod dom;
mod v8context;
pub use dom::{DOMDocument, DOMVisitor};
mod process;
pub use process::{ProcessId, ProcessMessage};
mod request;
pub use request::{PostData, PostDataElement, Request};
mod url_request;
pub use url_request::{AuthCallback, URLRequest, URLRequestClient};
mod request_context;
pub use request_context::{RequestContext, RequestContextBuilder, RequestContextHandler};
mod web_plugin;
pub use web_plugin::WebPluginInfo;
mod cookie;
pub use cookie::Cookie;
mod callback;
pub use callback::Callback;
mod resource_request_handler;
pub use resource_request_handler::ResourceRequestHandler;
mod client;
pub use client::Client;
mod image;
pub use image::Image;

mod command_line;
pub use command_line::CommandLine;
mod app;
pub use app::{App, AppCallbacks};

#[cfg(target_os = "windows")]
mod sandbox;
#[cfg(target_os = "windows")]
pub use sandbox::SandboxInfo;
mod main_args;
pub use main_args::MainArgs;
mod settings;
pub use settings::{LogSeverity, Settings};
mod color;
pub use color::Color;
mod events;
pub use events::{
    KeyEvent, KeyEventType, MouseButtonType, MouseEvent, PointerType, TouchEvent, TouchEventType,
};
mod drag;
pub use drag::{DragData, DragOperation};
mod file_dialog;
mod printing;
pub use printing::PDFPrintSettings;
mod window;
use num_enum::UnsafeFromPrimitive;
pub use window::WindowInfo;
mod ime;
pub use ime::CompositionUnderline;
mod navigation;
pub use navigation::NavigationEntry;

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
