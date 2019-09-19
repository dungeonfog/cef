use std::{
    sync::Arc,
    ptr::null_mut,
};
use cef_sys::cef_app_t;

use crate::{
    reference,
    refcounted::RefCounted,
    string::CefString,
    ptr_hash::Hashed,
    command_line::CommandLine,
    scheme_registrar::SchemeRegistrar,
    resource_bundle_handler::ResourceBundleHandler,
    browser_process_handler::BrowserProcessHandler,
};

/// Implement this structure to provide handler implementations. Methods will be
/// called by the process and/or thread indicated.
pub trait AppCallbacks {
    /// Provides an opportunity to view and/or modify command-line arguments before
    /// processing by CEF and Chromium. The `process_type` value will be `None` for
    /// the browser process. Do not keep a reference to the CommandLine
    /// object passed to this function. The [CefSettings.command_line_args_disabled]
    /// value can be used to start with an NULL command-line object. Any values
    /// specified in [CefSettings] that equate to command-line arguments will be set
    /// before this function is called. Be cautious when using this function to
    /// modify command-line arguments for non-browser processes as this may result
    /// in undefined behavior including crashes.
    fn on_before_command_line_processing(&self, process_type: Option<&str>, command_line: &CommandLine) {}
    /// Provides an opportunity to register custom schemes. Do not keep a reference
    /// to the `registrar` object. This function is called on the main thread for
    /// each process and the registered schemes should be the same across all
    /// processes.
    fn on_register_custom_schemes(&self, registrar: &SchemeRegistrar) {}
    /// Return the handler for resource bundle events. If
    /// [CefSettings.pack_loading_disabled] is true a handler must be returned.
    /// If no handler is returned resources will be loaded from pack files. This
    /// function is called by the browser and render processes on multiple threads.
    fn get_resource_bundle_handler(&self) -> Option<Box<dyn ResourceBundleHandler>> { None }
    /// Return the handler for functionality specific to the browser process. This
    /// function is called on multiple threads in the browser process.
    fn get_browser_process_handler(&self) -> Option<Box<dyn BrowserProcessHandler>> { None }

    // / Return the handler for functionality specific to the render process. This
    // / function is called on the render process main thread.
    // fn get_render_process_handler(&self) -> Option<Box<dyn RenderProcessHandler>> { None }
}

pub struct App {
    app: Box<cef_app_t>,
    delegate: Box<dyn AppCallbacks>,
}

unsafe impl Sync for App {}
unsafe impl Send for App {}

impl App {
    pub fn new(delegate: impl AppCallbacks + 'static + Send + Sync) -> Arc<Self> {
        let app = RefCounted::wrap(cef_app_t {
            on_before_command_line_processing: Some(Self::on_before_command_line_processing),
            on_register_custom_schemes: Some(Self::on_register_custom_schemes),
            get_browser_process_handler: Some(Self::get_browser_process_handler),
            get_resource_bundle_handler: Some(Self::get_resource_bundle_handler),
            get_render_process_handler: Some(Self::get_render_process_handler),
            ..Default::default()
        });
        let instance = Arc::new(Self {
            app,
            delegate: Box::new(delegate),
            // render_process_handler: CefRenderProcessHandler::new(rss_feed),
        });

        reference::register(Hashed::from(&instance.app), &instance);

        instance
    }

    extern "C" fn on_before_command_line_processing(self_: *mut cef_sys::cef_app_t, process_type: *const cef_sys::cef_string_t, command_line: *mut cef_sys::cef_command_line_t) {
        if let Some(this) = reference::get::<Self>(Hashed::from(self_)) {
            this.delegate.on_before_command_line_processing(CefString::copy_raw_to_string(process_type).as_ref().map(|s| &**s), &CommandLine::from(command_line));
        }
    }
    extern "C" fn on_register_custom_schemes(self_: *mut cef_sys::cef_app_t, registrar: *mut cef_sys::cef_scheme_registrar_t) {
        if let Some(this) = reference::get::<Self>(Hashed::from(self_)) {
            this.delegate.on_register_custom_schemes(&SchemeRegistrar::from(registrar));
        }
    }
    extern "C" fn get_resource_bundle_handler(_self: *mut cef_sys::cef_app_t) -> *mut cef_sys::cef_resource_bundle_handler_t {
        null_mut()
    }
    extern "C" fn get_browser_process_handler(_self: *mut cef_sys::cef_app_t) -> *mut cef_sys::cef_browser_process_handler_t {
        null_mut()
    }
    extern "C" fn get_render_process_handler(self_: *mut cef_sys::cef_app_t) -> *mut cef_sys::cef_render_process_handler_t {
        if let Some(_this) = reference::get::<Self>(Hashed::from(&self_)) {
            // return this.render_process_handler.as_ptr() as *mut cef_sys::cef_render_process_handler_t;
            null_mut()
        } else {
            null_mut()
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        reference::unregister::<Self>(Hashed::from(&self.app));
    }
}
