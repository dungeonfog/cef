use crate::{
    browser_process_handler::{BrowserProcessHandler},
    command_line::CommandLine,
    refcounted::{RefCountedPtr, Wrapper},
    render_process_handler::{RenderProcessHandler},
    resource_bundle_handler::{ResourceBundleHandler},
    scheme_registrar::SchemeRegistrar,
    string::CefString,
};
use cef_sys::{
    cef_app_t, cef_render_process_handler_t,
    cef_resource_bundle_handler_t,
};
use std::{ptr::null_mut};

/// Implement this structure to provide handler implementations. Methods will be
/// called by the process and/or thread indicated.
pub trait AppCallbacks: 'static + Send + Sync {
    /// Provides an opportunity to view and/or modify command-line arguments before
    /// processing by CEF and Chromium. The `process_type` value will be `None` for
    /// the browser process. Do not keep a reference to the CommandLine
    /// object passed to this function. The [CefSettings.command_line_args_disabled]
    /// value can be used to start with a None command-line object. Any values
    /// specified in [CefSettings] that equate to command-line arguments will be set
    /// before this function is called. Be cautious when using this function to
    /// modify command-line arguments for non-browser processes as this may result
    /// in undefined behavior including crashes.
    fn on_before_command_line_processing(
        &self,
        process_type: Option<&str>,
        command_line: CommandLine,
    ) {
    }
    /// Provides an opportunity to register custom schemes. Do not keep a reference
    /// to the `registrar` object. This function is called on the main thread for
    /// each process and the registered schemes should be the same across all
    /// processes.
    fn on_register_custom_schemes(&self, registrar: SchemeRegistrar) {}
    /// Return the handler for resource bundle events. If
    /// [CefSettings.pack_loading_disabled] is true a handler must be returned.
    /// If no handler is returned resources will be loaded from pack files. This
    /// function is called by the browser and render processes on multiple threads.
    fn get_resource_bundle_handler(&self) -> Option<ResourceBundleHandler> {
        None
    }
    /// Return the handler for functionality specific to the browser process. This
    /// function is called on multiple threads in the browser process.
    fn get_browser_process_handler(&self) -> Option<BrowserProcessHandler> {
        None
    }
    /// Return the handler for functionality specific to the render process. This
    /// function is called on the render process main thread.
    fn get_render_process_handler(&self) -> Option<RenderProcessHandler> {
        None
    }
}

ref_counted_ptr! {
    /// Main entry point for using CEF
    pub struct App(*mut cef_app_t);
}

struct AppWrapper(Box<dyn AppCallbacks>);

impl Wrapper for AppWrapper {
    type Cef = cef_app_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_app_t {
                base: unsafe { std::mem::zeroed() },
                on_before_command_line_processing: Some(
                    AppWrapper::on_before_command_line_processing,
                ),
                on_register_custom_schemes: Some(AppWrapper::on_register_custom_schemes),
                get_browser_process_handler: Some(AppWrapper::get_browser_process_handler),
                get_resource_bundle_handler: Some(AppWrapper::get_resource_bundle_handler),
                get_render_process_handler: Some(AppWrapper::get_render_process_handler),
            },
            self,
        )
    }
}

impl App {
    pub fn new<C: AppCallbacks>(delegate: C) -> Self {
        App(AppWrapper::new(Box::new(delegate)).wrap())
    }
}

impl AppWrapper {
    pub fn new(delegate: Box<dyn AppCallbacks>) -> AppWrapper {
        AppWrapper(delegate)
    }
}

cef_callback_impl! {
    impl for AppWrapper: cef_app_t {
        fn on_before_command_line_processing(
            &self,
            process_type: Option<&CefString>: *const cef_sys::cef_string_t,
            command_line: CommandLine: *mut cef_sys::cef_command_line_t,
        ) {
            self.0.on_before_command_line_processing(
                process_type
                    .map(String::from)
                    .as_ref()
                    .map(|s| &**s),
                command_line
            );
        }
        fn on_register_custom_schemes(
            &self,
            registrar: SchemeRegistrar: *mut cef_sys::cef_scheme_registrar_t,
        ) {
            self.0.on_register_custom_schemes(registrar);
        }
        fn get_resource_bundle_handler(&self) -> *mut cef_resource_bundle_handler_t {
            self.0.get_resource_bundle_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_browser_process_handler(&self) -> *mut cef_sys::cef_browser_process_handler_t {
            self.0.get_browser_process_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
        fn get_render_process_handler(&self) -> *mut cef_render_process_handler_t {
            self.0.get_render_process_handler().map(|cef| cef.into_raw()).unwrap_or(null_mut())
        }
    }
}
