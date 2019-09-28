#![allow(unused)]
mod ptr_hash;
mod string;
pub use string::StringVisitor;
mod multimap;
mod refcounted;
mod values;
pub use values::StoredValue;
mod ids;
pub use ids::{PackResource, PackString};
mod scheme_registrar;
pub use scheme_registrar::{SchemeOptions, SchemeRegistrar};
mod resource_bundle_handler;
pub use resource_bundle_handler::{ScaleFactor, ResourceBundleHandler};
mod browser_process_handler;
pub use browser_process_handler::BrowserProcessHandler;
mod browser;
pub use browser::Browser;
mod browser_host;
pub use browser_host::BrowserHost;
mod frame;
pub use frame::Frame;
mod load_handler;
pub use load_handler::LoadHandler;
mod render_process_handler;
pub use render_process_handler::RenderProcessHandler;
mod v8context;
mod dom;
pub use dom::{DOMDocument, DOMVisitor};
mod process;
mod request;
pub use request::{Request, PostData, PostDataElement};
mod urlrequest;
pub use urlrequest::{URLRequest, URLRequestClient, AuthCallback};
mod request_context;
pub use request_context::{RequestContextHandler, RequestContext, RequestContextBuilder};
mod web_plugin;
pub use web_plugin::WebPluginInfo;
mod cookie;
pub use cookie::Cookie;
mod callback;
pub use callback::Callback;
mod resource_request;
pub use resource_request::ResourceRequestHandler;

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
pub use settings::Settings;
mod color;
pub use color::Color;
mod key_event;
pub use key_event::{KeyEventType, KeyEvent};

use num_enum::UnsafeFromPrimitive;

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
