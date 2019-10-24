#![allow(dead_code, unused_variables)]
#![feature(fn_traits, unboxed_closures, trait_alias)]
#[macro_use]
mod refcounted;
#[macro_use]
mod extern_callback_helpers;
mod cef_helper_traits;
mod ptr_hash;
pub mod string;
pub use string::StringVisitorCallback;
pub mod multimap;
pub mod values;
pub use values::StoredValue;
pub mod scheme_registrar;
pub use scheme_registrar::{SchemeOptions, SchemeRegistrar};
pub mod resource_bundle_handler;
pub use resource_bundle_handler::{ResourceBundleHandlerCallbacks, ScaleFactor};
pub mod browser_process_handler;
pub use browser_process_handler::BrowserProcessHandlerCallbacks;
pub mod browser;
pub use browser::{Browser, BrowserSettings, State};
pub mod browser_host;
pub use browser_host::{BrowserHost, WindowHandle};
pub mod frame;
pub use frame::Frame;
pub mod load_handler;
pub use load_handler::LoadHandlerCallbacks;
pub mod render_process_handler;
pub use render_process_handler::RenderProcessHandlerCallbacks;
pub mod dom;
pub mod v8context;
pub use dom::{DOMDocument, DOMVisitorCallback};
pub mod process;
pub use process::{ProcessId, ProcessMessage};
pub mod request;
pub use request::{PostData, PostDataElement, Request};
pub mod url_request;
pub use url_request::{AuthCallback, URLRequest, URLRequestClientCallbacks};
pub mod request_context;
pub use request_context::{RequestContext, RequestContextBuilder, RequestContextHandlerCallbacks};
pub mod web_plugin;
pub use web_plugin::WebPluginInfo;
pub mod cookie;
pub use cookie::Cookie;
pub mod callback;
pub use callback::Callback;
pub mod resource_request_handler;
pub use resource_request_handler::ResourceRequestHandlerCallbacks;
pub mod client;
pub use client::ClientCallbacks;
pub mod image;
pub use image::{AlphaType, BinaryImage, ColorType, Image, RepresentationInfo};

pub mod command_line;
pub use command_line::CommandLine;
pub mod app;
pub use app::{App, AppCallbacks};

#[cfg(target_os = "windows")]
pub mod sandbox;
#[cfg(target_os = "windows")]
pub use sandbox::SandboxInfo;
pub mod main_args;
pub use main_args::MainArgs;
pub mod settings;
pub use settings::{LogSeverity, Settings};
pub mod color;
pub use color::Color;
pub mod events;
pub use events::{
    KeyEvent, KeyEventType, MouseButtonType, MouseEvent, PointerType, TouchEvent, TouchEventType,
};
pub mod drag;
pub use drag::{DragData, DragOperation};
pub mod file_dialog;
pub mod printing;
pub use printing::PDFPrintSettings;
pub mod window;
use num_enum::UnsafeFromPrimitive;
pub use window::WindowInfo;
pub mod ime;
pub use ime::CompositionUnderline;
pub mod navigation;
pub use navigation::NavigationEntry;
pub mod extension;
pub use extension::Extension;
pub mod request_handler;
pub use request_handler::RequestHandlerCallbacks;
pub mod ssl;
pub use ssl::{CertStatus, SSLInfo, X509Certificate};
pub mod task;
pub use task::TaskRunner;

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
