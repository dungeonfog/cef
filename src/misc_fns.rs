//! Free-floating functions that aren't associated with any particular CEF type.
//!
//! These get re-exported in the crate root, and this module should not be visible
//! to the end user.

use crate::{
    app::App,
    main_args::MainArgs,
    sandbox::SandboxInfo,
    settings::Settings,
};
use cef_sys::{
    cef_do_message_loop_work, cef_execute_process,
    cef_initialize, cef_quit_message_loop, cef_run_message_loop, cef_shutdown,
};
use std::{
    ptr::null_mut,
    sync::atomic::{AtomicBool, Ordering},
};


/// Call during process startup to enable High-DPI support on Windows 7 or newer.
///
/// Older versions of Windows should be left DPI-unaware because they do not
/// support DirectWrite and GDI fonts are kerned very badly.
fn enable_highdpi_support() {
    #[cfg(target_os = "windows")]
    {
        static ENABLED: AtomicBool = AtomicBool::new(false);
        if !ENABLED.swap(true, Ordering::SeqCst) {
            use winapi::um::{winbase, winnt};
            unsafe {
                let windows_7_or_greater = {
                    let mut version_info = winnt::OSVERSIONINFOEXA {
                        dwOSVersionInfoSize: std::mem::size_of::<winnt::OSVERSIONINFOEXA>() as _,
                        // yes this is 6.1 for Windows 7. The windows headers have the following definition:
                        // #define _WIN32_WINNT_WIN7 0x0601 // Windows 7
                        dwMajorVersion: 6,
                        dwMinorVersion: 1,
                        dwBuildNumber: 0,
                        dwPlatformId: 0,
                        szCSDVersion: [0; 128],
                        wServicePackMajor: 0,
                        wServicePackMinor: 0,
                        wSuiteMask: 0,
                        wProductType: 0,
                        wReserved: 0,
                    };
                    let c = winnt::VerSetConditionMask(0, winnt::VER_MAJORVERSION, winnt::VER_GREATER_EQUAL);
                    let c = winnt::VerSetConditionMask(c, winnt::VER_MINORVERSION, winnt::VER_GREATER_EQUAL);
                    let c = winnt::VerSetConditionMask(c, winnt::VER_SERVICEPACKMAJOR, winnt::VER_GREATER_EQUAL);
                    winbase::VerifyVersionInfoA(&mut version_info, winnt::VER_MAJORVERSION | winnt::VER_MINORVERSION | winnt::VER_SERVICEPACKMAJOR, c) != 0
                };
                if windows_7_or_greater {
                    cef_sys::cef_enable_highdpi_support();
                }
            }
        }
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
/// `sandbox_info` parameter may be None (see [SandboxInfo] for details).
pub fn execute_process(
    application: Option<App>,
    sandbox_info: Option<&SandboxInfo>,
) -> i32 {
    let args = MainArgs::new();
    enable_highdpi_support();
    unsafe {
        cef_execute_process(
            args.get(),
            application.map(|app| app.into_raw()).unwrap_or_else(null_mut),
            sandbox_info
                .map(|wsi| wsi.get())
                .unwrap_or_else(null_mut),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProcessType {
    Browser,
    Renderer,
    Gpu,
    Utility,
    Other,
}

pub fn process_type() -> ProcessType {
    let flag = "--type=";
    let arg = std::env::args().find(|s| s.starts_with(flag));
    let arg = arg.as_ref().map(|s| &s[flag.len()..]);
    match arg {
        None => ProcessType::Browser,
        Some("renderer") => ProcessType::Renderer,
        Some("gpu-process") => ProcessType::Gpu,
        Some("utility") => ProcessType::Utility,
        _ => ProcessType::Other,
    }
}

pub struct Context(());

impl !Send for Context {}
impl !Sync for Context {}
static CONTEXT_INITIALIZED: AtomicBool = AtomicBool::new(false);

impl Context {
    /// This function should be called on the main application thread to initialize
    /// the CEF browser process.
    ///
    /// The `application` parameter may be None. A return
    /// value of true indicates that it succeeded and false indicates that it
    /// failed. The `windows_sandbox_info` parameter is only used on Windows and may
    /// be None (see [SandboxInfo] for details).
    pub fn initialize(
        settings: &Settings,
        application: Option<App>,
        sandbox_info: Option<&SandboxInfo>,
    ) -> Result<Context, std::io::Error> {
        let args = MainArgs::new();
        enable_highdpi_support();
        CONTEXT_INITIALIZED.swap(true, Ordering::SeqCst);
        unsafe {
            let settings = settings.to_cef(sandbox_info.is_some())?;
            let worked = cef_initialize(
                args.get(),
                &settings,
                application.map(|app| app.into_raw()).unwrap_or_else(null_mut),
                sandbox_info
                    .map(|wsi| wsi.get())
                    .unwrap_or_else(null_mut),
            ) != 0;
            crate::settings::drop_settings(settings);
            match worked {
                true => Ok(Context(())),
                false => Err(std::io::Error::new(std::io::ErrorKind::Other, "context creation failed!")),
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
        CONTEXT_INITIALIZED.swap(false, Ordering::SeqCst);
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
        cef_sys::cef_set_osmodal_loop(os_modal_loop as i32);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuitMessageLoopError {
    NotOnMainThread,
    ContextNotInitialized,
}

/// Quit the CEF message loop that was started by calling [App::run_message_loop].
pub fn quit_message_loop() -> Result<(), QuitMessageLoopError> {
    if !crate::task::TaskRunner::currently_on(crate::task::ThreadId::UI) {
        return Err(QuitMessageLoopError::NotOnMainThread);
    }
    if !CONTEXT_INITIALIZED.load(Ordering::SeqCst) {
        return Err(QuitMessageLoopError::ContextNotInitialized);
    }
    unsafe {
        cef_quit_message_loop();
    }
    Ok(())
}

