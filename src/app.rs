use std::{
    sync::Arc,
    ptr::null_mut,
    mem::ManuallyDrop,
};
use cef_sys::{cef_app_t, cef_base_ref_counted_t, cef_resource_bundle_handler_t, cef_render_process_handler_t, cef_browser_process_handler_t};

use crate::{
    refcounted::{RefCounted, RefCounter},
    string::CefString,
    ptr_hash::Hashed,
    command_line::CommandLine,
    scheme_registrar::SchemeRegistrar,
    resource_bundle_handler::{ResourceBundleHandler, ResourceBundleHandlerWrapper},
    browser_process_handler::{BrowserProcessHandler, BrowserProcessHandlerWrapper},
    render_process_handler::{RenderProcessHandler, RenderProcessHandlerWrapper},
};

/// Implement this structure to provide handler implementations. Methods will be
/// called by the process and/or thread indicated.
pub trait AppCallbacks {
    /// Provides an opportunity to view and/or modify command-line arguments before
    /// processing by CEF and Chromium. The `process_type` value will be `None` for
    /// the browser process. Do not keep a reference to the CommandLine
    /// object passed to this function. The [CefSettings.command_line_args_disabled]
    /// value can be used to start with a None command-line object. Any values
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
    /// Return the handler for functionality specific to the render process. This
    /// function is called on the render process main thread.
    fn get_render_process_handler(&self) -> Option<Box<dyn RenderProcessHandler>> { None }
}

pub struct AppWrapper {
    delegate: Box<dyn AppCallbacks>,
    resource_bundle_handler: *mut <cef_resource_bundle_handler_t as RefCounter>::Wrapper,
    browser_process_handler: *mut <cef_browser_process_handler_t as RefCounter>::Wrapper,
    render_process_handler: *mut <cef_render_process_handler_t as RefCounter>::Wrapper,
}

/// Opaque reference to CEF's app struct
pub struct App(*mut cef_app_t);

unsafe impl Sync for AppWrapper {}
unsafe impl Send for AppWrapper {}

impl RefCounter for cef_app_t {
    type Wrapper = RefCounted<cef_app_t, AppWrapper>;
    fn set_base(&mut self, base: cef_base_ref_counted_t) {
        self.base = base;
    }
}

impl App {
    pub fn new(delegate: Box<dyn AppCallbacks>) -> Self {
        let rc = RefCounted::new(cef_app_t {
            on_before_command_line_processing: Some(AppWrapper::on_before_command_line_processing),
            on_register_custom_schemes:        Some(AppWrapper::on_register_custom_schemes),
            get_browser_process_handler:       Some(AppWrapper::get_browser_process_handler),
            get_resource_bundle_handler:       Some(AppWrapper::get_resource_bundle_handler),
            get_render_process_handler:        Some(AppWrapper::get_render_process_handler),
            ..Default::default()
        }, AppWrapper {
            delegate,
            resource_bundle_handler: null_mut(),
            browser_process_handler: null_mut(),
            render_process_handler: null_mut(),
        });
        let mut this = unsafe { <cef_app_t as RefCounter>::Wrapper::make_temp(rc as *mut _) };
        Self(this.get_cef())
    }
}

impl AppWrapper {
    extern "C" fn on_before_command_line_processing(self_: *mut cef_sys::cef_app_t, process_type: *const cef_sys::cef_string_t, command_line: *mut cef_sys::cef_command_line_t) {
        let this = unsafe { <cef_app_t as RefCounter>::Wrapper::make_temp(self_) };
        (**this).delegate.on_before_command_line_processing(CefString::copy_raw_to_string(process_type).as_ref().map(|s| &**s), &CommandLine::from(command_line));
    }
    extern "C" fn on_register_custom_schemes(self_: *mut cef_sys::cef_app_t, registrar: *mut cef_sys::cef_scheme_registrar_t) {
        let this = unsafe { <cef_app_t as RefCounter>::Wrapper::make_temp(self_) };
        (**this).delegate.on_register_custom_schemes(&SchemeRegistrar::from(registrar));
    }
    extern "C" fn get_resource_bundle_handler(self_: *mut cef_sys::cef_app_t) -> *mut cef_resource_bundle_handler_t {
        let mut this = unsafe { <cef_app_t as RefCounter>::Wrapper::make_temp(self_) };
        if let Some(handler) = (**this).delegate.get_resource_bundle_handler() {
            let wrapper = ResourceBundleHandlerWrapper::new(handler);
            (**this).resource_bundle_handler = wrapper;
            wrapper as *mut cef_resource_bundle_handler_t
        } else {
            if !(**this).resource_bundle_handler.is_null() {
                <cef_resource_bundle_handler_t as RefCounter>::Wrapper::release((*this).resource_bundle_handler as *mut cef_base_ref_counted_t);
                (**this).resource_bundle_handler = null_mut();
            }
            null_mut()
        }
    }
    extern "C" fn get_browser_process_handler(self_: *mut cef_sys::cef_app_t) -> *mut cef_sys::cef_browser_process_handler_t {
        let mut this = unsafe { <cef_app_t as RefCounter>::Wrapper::make_temp(self_) };
        if let Some(handler) = this.delegate.get_browser_process_handler() {
            let wrapper = BrowserProcessHandlerWrapper::new(handler);
            this.browser_process_handler = wrapper;
            wrapper as *mut cef_browser_process_handler_t
        } else {
            if !this.browser_process_handler.is_null() {
                <cef_browser_process_handler_t as RefCounter>::Wrapper::release((*this).browser_process_handler as *mut cef_base_ref_counted_t);
                this.browser_process_handler = null_mut();
            }
            null_mut()
        }
    }
    extern "C" fn get_render_process_handler(self_: *mut cef_sys::cef_app_t) -> *mut cef_render_process_handler_t {
        let mut this = unsafe { <cef_app_t as RefCounter>::Wrapper::make_temp(self_) };
        if let Some(handler) = this.delegate.get_render_process_handler() {
            let wrapper = RenderProcessHandlerWrapper::new(handler);
            this.render_process_handler = wrapper;
            wrapper as *mut cef_render_process_handler_t
        } else {
            if !this.render_process_handler.is_null() {
                <cef_render_process_handler_t as RefCounter>::Wrapper::release((*this).render_process_handler as *mut cef_base_ref_counted_t);
                this.render_process_handler = null_mut();
            }
            null_mut()
        }
    }
}

impl Drop for AppWrapper {
    fn drop(&mut self) {
        if !self.browser_process_handler.is_null() {
            <cef_browser_process_handler_t as RefCounter>::Wrapper::release(self.browser_process_handler as *mut cef_base_ref_counted_t);
        }
        if !self.render_process_handler.is_null() {
            <cef_render_process_handler_t as RefCounter>::Wrapper::release(self.render_process_handler as *mut cef_base_ref_counted_t);
        }
    }
}
