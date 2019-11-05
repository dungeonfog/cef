//! Free-floating functions that aren't associated with any particular CEF type.
//!
//! These get re-exported in the crate root, and this module should not be visible
//! to the end user.

use crate::{
    app::App,
    main_args::MainArgs,
    settings::Settings,
};
use cef_sys::{
    cef_do_message_loop_work, cef_execute_process,
    cef_initialize, cef_quit_message_loop, cef_run_message_loop, cef_set_osmodal_loop, cef_shutdown,
};
use std::{ptr::null_mut};

#[cfg(target_os = "windows")]
use crate::sandbox::SandboxInfo;


/// Call during process startup to enable High-DPI support on Windows 7 or newer.
///
/// Older versions of Windows should be left DPI-unaware because they do not
/// support DirectWrite and GDI fonts are kerned very badly.
#[cfg(target_os = "windows")]
pub fn enable_highdpi_support() {
    unsafe {
        cef_sys::cef_enable_highdpi_support();
    }
}
/// This function should be called from the application entry point function to
/// execute a secondary process.
///
/// It can be used to run secondary processes from
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
    application: Option<App>,
) -> i32 {
    unsafe {
        cef_execute_process(
            args.get(),
            application.map(|app| app.into_raw()).unwrap_or_else(null_mut),
            null_mut(),
            // windows_sandbox_info
            //     .map(|wsi| wsi.get())
            //     .unwrap_or_else(null_mut),
        )
    }
}
/// This function should be called from the application entry point function to
/// execute a secondary process.
///
/// It can be used to run secondary processes from
/// the browser client executable (default behavior) or from a separate
/// executable specified by the [CefSettings::browser_subprocess_path] value. If
/// called for the browser process (identified by no "type" command-line value)
/// it will return immediately with a value of -1. If called for a recognized
/// secondary process it will block until the process should exit and then return
/// the process exit code. The `application` parameter may be None.
#[cfg(not(target_os = "windows"))]
pub fn execute_process(args: &MainArgs, application: Option<App>) -> i32 {
    unsafe {
        cef_execute_process(
            args.get(),
            application
                .and_then(|app| Some(app.into_raw()))
                .unwrap_or_else(null_mut),
            null_mut(),
        )
    }
}


pub struct Context(());

impl Context {
    /// This function should be called on the main application thread to initialize
    /// the CEF browser process.
    ///
    /// The `application` parameter may be None. A return
    /// value of true indicates that it succeeded and false indicates that it
    /// failed. The `windows_sandbox_info` parameter is only used on Windows and may
    /// be None (see [SandboxInfo] for details).
    #[cfg(target_os = "windows")]
    pub fn initialize(
        args: &MainArgs,
        settings: &Settings,
        application: Option<App>,
    ) -> Option<Context> {
        unsafe {
            let worked = cef_initialize(
                args.get(),
                settings.get(),
                application.map(|app| app.into_raw()).unwrap_or_else(null_mut),
                null_mut(),
                // windows_sandbox_info
                //     .map(|wsi| wsi.get())
                //     .unwrap_or_else(null_mut),
            ) != 0;
            match worked {
                true => Some(Context(())),
                false => None,
            }
        }
    }

    /// Perform a single iteration of CEF message loop processing.
    ///
    /// This function is
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
    pub fn do_message_loop_work(&self) {
        unsafe {
            cef_do_message_loop_work();
        }
    }
    /// Run the CEF message loop.
    ///
    /// Use this function instead of an application-
    /// provided message loop to get the best balance between performance and CPU
    /// usage. This function should only be called on the main application thread and
    /// only if [App::initialize] is called with a
    /// [CefSettings::multi_threaded_message_loop] value of false. This function
    /// will block until a quit message is received by the system.
    pub fn run_message_loop(&self) {
        unsafe {
            cef_run_message_loop();
        }
    }
    /// Quit the CEF message loop that was started by calling [App::run_message_loop].
    ///
    /// This function should only be called on the main application thread and only
    /// if [App::run_message_loop] was used.
    pub fn quit_message_loop(&self) {
        unsafe {
            cef_quit_message_loop();
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            println!("drop context");
            cef_shutdown();
        }
    }
}

/// Set to true before calling Windows APIs like TrackPopupMenu that enter a
/// modal message loop.
///
/// Set to false after exiting the modal message loop.
#[cfg(target_os = "windows")]
pub fn set_osmodal_loop(os_modal_loop: bool) {
    unsafe {
        cef_set_osmodal_loop(os_modal_loop as i32);
    }
}
