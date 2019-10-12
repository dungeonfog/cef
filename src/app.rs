use cef_sys::{
    cef_app_t, cef_base_ref_counted_t, cef_browser_process_handler_t, cef_do_message_loop_work,
    cef_enable_highdpi_support, cef_execute_process, cef_initialize, cef_quit_message_loop,
    cef_render_process_handler_t, cef_resource_bundle_handler_t, cef_run_message_loop,
    cef_set_osmodal_loop, cef_shutdown,
};
use std::{ptr::null_mut};


use crate::{
    browser_process_handler::{BrowserProcessHandler, BrowserProcessHandlerWrapper},
    command_line::CommandLine,
    main_args::MainArgs,
    refcounted::{RefCounted, RefCountedPtr},
    render_process_handler::{RenderProcessHandler, RenderProcessHandlerWrapper},
    resource_bundle_handler::{ResourceBundleHandler, ResourceBundleHandlerWrapper},
    scheme_registrar::SchemeRegistrar,
    settings::Settings,
    string::CefString,
};

#[cfg(target_os = "windows")]
use crate::sandbox::SandboxInfo;

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
    fn on_before_command_line_processing(
        &self,
        process_type: Option<&str>,
        command_line: &CommandLine,
    ) {
    }
    /// Provides an opportunity to register custom schemes. Do not keep a reference
    /// to the `registrar` object. This function is called on the main thread for
    /// each process and the registered schemes should be the same across all
    /// processes.
    fn on_register_custom_schemes(&self, registrar: &SchemeRegistrar) {}
    /// Return the handler for resource bundle events. If
    /// [CefSettings.pack_loading_disabled] is true a handler must be returned.
    /// If no handler is returned resources will be loaded from pack files. This
    /// function is called by the browser and render processes on multiple threads.
    fn get_resource_bundle_handler(&self) -> Option<Box<dyn ResourceBundleHandler>> {
        None
    }
    /// Return the handler for functionality specific to the browser process. This
    /// function is called on multiple threads in the browser process.
    fn get_browser_process_handler(&self) -> Option<Box<dyn BrowserProcessHandler>> {
        None
    }
    /// Return the handler for functionality specific to the render process. This
    /// function is called on the render process main thread.
    fn get_render_process_handler(&self) -> Option<Box<dyn RenderProcessHandler>> {
        None
    }
}

pub struct AppWrapper {
    delegate: Box<dyn AppCallbacks>,
    resource_bundle_handler: *mut RefCounted<cef_resource_bundle_handler_t>,
    browser_process_handler: *mut RefCounted<cef_browser_process_handler_t>,
    render_process_handler: *mut RefCounted<cef_render_process_handler_t>,
}

ref_counted_ptr! {
    /// Main entry point for using CEF
    pub struct App(*mut cef_app_t);
}

unsafe impl Sync for AppWrapper {}
unsafe impl Send for AppWrapper {}

impl App {
    pub fn new<T: AppCallbacks + 'static>(delegate: T) -> Self {
        App(RefCountedPtr::wrap(
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
            AppWrapper {
                delegate: Box::new(delegate),
                resource_bundle_handler: null_mut(),
                browser_process_handler: null_mut(),
                render_process_handler: null_mut(),
            },
        ))
    }
    /// Call during process startup to enable High-DPI support on Windows 7 or newer.
    /// Older versions of Windows should be left DPI-unaware because they do not
    /// support DirectWrite and GDI fonts are kerned very badly.
    #[cfg(target_os = "windows")]
    pub fn enable_highdpi_support() {
        unsafe {
            cef_enable_highdpi_support();
        }
    }
    /// This function should be called from the application entry point function to
    /// execute a secondary process. It can be used to run secondary processes from
    /// the browser client executable (default behavior) or from a separate
    /// executable specified by the [CefSettings::browser_subprocess_path] value. If
    /// called for the browser process (identified by no "type" command-line value)
    /// it will return immediately with a value of -1. If called for a recognized
    /// secondary process it will block until the process should exit and then return
    /// the process exit code. The `application` parameter may be None. The
    /// `windows_sandbox_info` parameter may be None (see [SandboxInfo] for details).
    #[cfg(target_os = "windows")]
    pub fn execute_process(
        args: &MainArgs,
        application: Option<&App>,
        windows_sandbox_info: Option<&SandboxInfo>,
    ) -> i32 {
        unsafe {
            cef_execute_process(
                args.get(),
                application
                    .and_then(|app| Some(app.as_ptr()))
                    .unwrap_or_else(null_mut),
                windows_sandbox_info
                    .and_then(|wsi| Some(wsi.get()))
                    .unwrap_or_else(null_mut),
            )
        }
    }
    /// This function should be called from the application entry point function to
    /// execute a secondary process. It can be used to run secondary processes from
    /// the browser client executable (default behavior) or from a separate
    /// executable specified by the [CefSettings::browser_subprocess_path] value. If
    /// called for the browser process (identified by no "type" command-line value)
    /// it will return immediately with a value of -1. If called for a recognized
    /// secondary process it will block until the process should exit and then return
    /// the process exit code. The `application` parameter may be None.
    #[cfg(not(target_os = "windows"))]
    pub fn execute_process(args: &[&str], application: Option<&App>) -> i32 {
        unsafe {
            cef_execute_process(
                args.get(),
                application
                    .and_then(|app| Some(app.0))
                    .unwrap_or_else(null_mut),
                null_mut(),
            )
        }
    }
    /// This function should be called on the main application thread to initialize
    /// the CEF browser process. The `application` parameter may be None. A return
    /// value of true indicates that it succeeded and false indicates that it
    /// failed. The `windows_sandbox_info` parameter is only used on Windows and may
    /// be None (see [SandboxInfo] for details).
    #[cfg(target_os = "windows")]
    pub fn initialize(
        args: &MainArgs,
        settings: &Settings,
        application: Option<&App>,
        windows_sandbox_info: Option<&SandboxInfo>,
    ) -> bool {
        unsafe {
            cef_initialize(
                args.get(),
                settings.get(),
                application
                    .and_then(|app| Some(app.as_ptr()))
                    .unwrap_or_else(null_mut),
                windows_sandbox_info
                    .and_then(|wsi| Some(wsi.get()))
                    .unwrap_or_else(null_mut),
            ) != 0
        }
    }
    /// This function should be called on the main application thread to initialize
    /// the CEF browser process. The `application` parameter may be None. A return
    /// value of true indicates that it succeeded and false indicates that it
    /// failed.
    #[cfg(not(target_os = "windows"))]
    pub fn initialize(args: &[&str], settings: &Settings, application: Option<&App>) -> bool {
        unsafe {
            cef_initialize(
                args.get(),
                settings.get(),
                application
                    .and_then(|app| Some(app.0))
                    .unwrap_or_else(null_mut),
                null_mut(),
            ) != 0
        }
    }
    /// This function should be called on the main application thread to shut down
    /// the CEF browser process before the application exits.
    pub fn shutdown() {
        unsafe {
            cef_shutdown();
        }
    }
    /// Perform a single iteration of CEF message loop processing. This function is
    /// provided for cases where the CEF message loop must be integrated into an
    /// existing application message loop. Use of this function is not recommended
    /// for most users; use either the [App::run_message_loop] function or
    /// [Settings::multi_threaded_message_loop] if possible. When using this function
    /// care must be taken to balance performance against excessive CPU usage. It is
    /// recommended to enable the [Settings::external_message_pump] option when using
    /// this function so that
    /// [BrowserProcessHandler::on_schedule_message_pump_work] callbacks can
    /// facilitate the scheduling process. This function should only be called on the
    /// main application thread and only if [App::initialize] is called with a
    /// [Settings::multi_threaded_message_loop] value of false. This function
    /// will not block.
    pub fn do_message_loop_work() {
        unsafe {
            cef_do_message_loop_work();
        }
    }
    /// Run the CEF message loop. Use this function instead of an application-
    /// provided message loop to get the best balance between performance and CPU
    /// usage. This function should only be called on the main application thread and
    /// only if [App::initialize] is called with a
    /// [CefSettings::multi_threaded_message_loop] value of false. This function
    /// will block until a quit message is received by the system.
    pub fn run_message_loop() {
        unsafe {
            cef_run_message_loop();
        }
    }
    /// Quit the CEF message loop that was started by calling [App::run_message_loop].
    /// This function should only be called on the main application thread and only
    /// if [App::run_message_loop] was used.
    pub fn quit_message_loop() {
        unsafe {
            cef_quit_message_loop();
        }
    }
    /// Set to true before calling Windows APIs like TrackPopupMenu that enter a
    /// modal message loop. Set to false after exiting the modal message loop.
    #[cfg(target_os = "windows")]
    pub fn set_osmodal_loop(os_modal_loop: bool) {
        unsafe {
            cef_set_osmodal_loop(os_modal_loop as i32);
        }
    }
}

impl AppWrapper {
    extern "C" fn on_before_command_line_processing(
        self_: *mut cef_sys::cef_app_t,
        process_type: *const cef_sys::cef_string_t,
        command_line: *mut cef_sys::cef_command_line_t,
    ) {
        let this = unsafe { RefCounted::<cef_app_t>::make_temp(self_) };
        (**this).delegate.on_before_command_line_processing(
            unsafe { CefString::copy_raw_to_string(process_type) }
                .as_ref()
                .map(|s| &**s),
            unsafe { &CommandLine::from_ptr_unchecked(command_line) },
        );
    }
    extern "C" fn on_register_custom_schemes(
        self_: *mut cef_sys::cef_app_t,
        registrar: *mut cef_sys::cef_scheme_registrar_t,
    ) {
        let this = unsafe { RefCounted::<cef_app_t>::make_temp(self_) };
        (**this)
            .delegate
            .on_register_custom_schemes(&SchemeRegistrar::from(registrar));
    }
    extern "C" fn get_resource_bundle_handler(
        self_: *mut cef_sys::cef_app_t,
    ) -> *mut cef_resource_bundle_handler_t {
        let mut this = unsafe { RefCounted::<cef_app_t>::make_temp(self_) };
        if let Some(handler) = (**this).delegate.get_resource_bundle_handler() {
            let wrapper = ResourceBundleHandlerWrapper::new(handler);
            (**this).resource_bundle_handler = wrapper;
            wrapper as *mut cef_resource_bundle_handler_t
        } else {
            if !(**this).resource_bundle_handler.is_null() {
                RefCounted::<cef_resource_bundle_handler_t>::release(
                    (*this).resource_bundle_handler as *mut cef_base_ref_counted_t,
                );
                (**this).resource_bundle_handler = null_mut();
            }
            null_mut()
        }
    }
    extern "C" fn get_browser_process_handler(
        self_: *mut cef_sys::cef_app_t,
    ) -> *mut cef_sys::cef_browser_process_handler_t {
        let mut this = unsafe { RefCounted::<cef_app_t>::make_temp(self_) };
        if let Some(handler) = this.delegate.get_browser_process_handler() {
            let wrapper = BrowserProcessHandlerWrapper::new(handler);
            this.browser_process_handler = wrapper;
            wrapper as *mut cef_browser_process_handler_t
        } else {
            if !this.browser_process_handler.is_null() {
                RefCounted::<cef_browser_process_handler_t>::release(
                    (*this).browser_process_handler as *mut cef_base_ref_counted_t,
                );
                this.browser_process_handler = null_mut();
            }
            null_mut()
        }
    }
    extern "C" fn get_render_process_handler(
        self_: *mut cef_sys::cef_app_t,
    ) -> *mut cef_render_process_handler_t {
        let mut this = unsafe { RefCounted::<cef_app_t>::make_temp(self_) };
        if let Some(handler) = this.delegate.get_render_process_handler() {
            let wrapper = RenderProcessHandlerWrapper::new(handler);
            this.render_process_handler = wrapper;
            wrapper as *mut cef_render_process_handler_t
        } else {
            if !this.render_process_handler.is_null() {
                RefCounted::<cef_render_process_handler_t>::release(
                    (*this).render_process_handler as *mut cef_base_ref_counted_t,
                );
                this.render_process_handler = null_mut();
            }
            null_mut()
        }
    }
}

impl Drop for AppWrapper {
    fn drop(&mut self) {
        if !self.browser_process_handler.is_null() {
            RefCounted::<cef_browser_process_handler_t>::release(
                self.browser_process_handler as *mut cef_base_ref_counted_t,
            );
        }
        if !self.render_process_handler.is_null() {
            RefCounted::<cef_render_process_handler_t>::release(
                self.render_process_handler as *mut cef_base_ref_counted_t,
            );
        }
    }
}
