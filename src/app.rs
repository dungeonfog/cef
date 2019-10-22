use crate::{
    browser_process_handler::{BrowserProcessHandler},
    command_line::CommandLine,
    main_args::MainArgs,
    refcounted::{RefCountedPtr, Wrapper},
    render_process_handler::{RenderProcessHandler},
    resource_bundle_handler::{ResourceBundleHandler},
    scheme_registrar::SchemeRegistrar,
    settings::Settings,
    string::CefString,
};
use cef_sys::{
    cef_app_t, cef_do_message_loop_work, cef_enable_highdpi_support, cef_execute_process,
    cef_initialize, cef_quit_message_loop, cef_render_process_handler_t,
    cef_resource_bundle_handler_t, cef_run_message_loop, cef_set_osmodal_loop, cef_shutdown,
};
use std::{ptr::null_mut};

#[cfg(target_os = "windows")]
use crate::sandbox::SandboxInfo;

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

pub struct AppWrapper(Box<dyn AppCallbacks>);

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
    pub fn new(delegate: Box<dyn AppCallbacks>) -> Self {
        App(AppWrapper::new(delegate).wrap())
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
                application.map(|app| app.as_ptr()).unwrap_or_else(null_mut),
                windows_sandbox_info
                    .map(|wsi| wsi.get())
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
                application.map(|app| app.as_ptr()).unwrap_or_else(null_mut),
                windows_sandbox_info
                    .map(|wsi| wsi.get())
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
                application.map(|app| app.0).unwrap_or_else(null_mut),
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
    /// [BrowserProcessHandlerCallbacks::on_schedule_message_pump_work] callbacks can
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
