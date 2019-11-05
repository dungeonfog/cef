use crate::color::Color;
use cef_sys::{cef_log_severity_t, cef_settings_t, cef_string_utf8_to_utf16};
use num_enum::UnsafeFromPrimitive;
use std::path::Path;

/// Log severity levels.
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, UnsafeFromPrimitive)]
pub enum LogSeverity {
    /// Default logging (currently INFO logging).
    Default = cef_log_severity_t::LOGSEVERITY_DEFAULT,
    /// Verbose logging (same as DEBUG logging).
    Verbose = cef_log_severity_t::LOGSEVERITY_VERBOSE,
    /// INFO logging.
    Info = cef_log_severity_t::LOGSEVERITY_INFO,
    /// WARNING logging.
    Warning = cef_log_severity_t::LOGSEVERITY_WARNING,
    /// ERROR logging.
    Error = cef_log_severity_t::LOGSEVERITY_ERROR,
    /// FATAL logging.
    Fatal = cef_log_severity_t::LOGSEVERITY_FATAL,
    /// Disable logging to file for all messages, and to stderr for messages with
    /// severity less than FATAL.
    Disable = cef_log_severity_t::LOGSEVERITY_DISABLE,
}

pub struct Settings(cef_settings_t);

impl Settings {
    pub fn new() -> Self {
        Self(cef_settings_t {
            size: std::mem::size_of::<cef_settings_t>(),
            no_sandbox: 0,
            browser_subprocess_path: unsafe { std::mem::zeroed() },
            framework_dir_path: unsafe { std::mem::zeroed() },
            main_bundle_path: unsafe { std::mem::zeroed() },
            multi_threaded_message_loop: 0,
            external_message_pump: 0,
            windowless_rendering_enabled: 0,
            command_line_args_disabled: 0,
            cache_path: unsafe { std::mem::zeroed() },
            root_cache_path: unsafe { std::mem::zeroed() },
            user_data_path: unsafe { std::mem::zeroed() },
            persist_session_cookies: 0,
            persist_user_preferences: 0,
            user_agent: unsafe { std::mem::zeroed() },
            product_version: unsafe { std::mem::zeroed() },
            locale: unsafe { std::mem::zeroed() },
            log_file: unsafe { std::mem::zeroed() },
            log_severity: cef_log_severity_t::LOGSEVERITY_DEFAULT,
            javascript_flags: unsafe { std::mem::zeroed() },
            resources_dir_path: unsafe { std::mem::zeroed() },
            locales_dir_path: unsafe { std::mem::zeroed() },
            pack_loading_disabled: 0,
            remote_debugging_port: 0,
            uncaught_exception_stack_size: 0,
            ignore_certificate_errors: 0,
            enable_net_security_expiration: 0,
            background_color: 0,
            accept_language_list: unsafe { std::mem::zeroed() },
            application_client_id_for_file_scanning: unsafe { std::mem::zeroed() },
        })
    }
    pub(crate) fn get(&self) -> *const cef_settings_t {
        &self.0
    }
    pub(crate) fn get_mut(&mut self) -> *mut cef_settings_t {
        &mut self.0
    }

    /// Call to disable the sandbox for sub-processes. See
    /// [sandbox::SandboxInfo] for requirements to enable the sandbox on Windows. Also
    /// configurable using the "no-sandbox" command-line switch.
    pub fn disable_sandbox(&mut self) {
        self.0.no_sandbox = 1;
    }
    /// Set the path to a separate executable that will be launched for sub-processes.
    /// If this value is empty on Windows or Linux then the main process executable
    /// will be used. If this value is empty on macOS then a helper executable must
    /// exist at "Contents/Frameworks/<app> Helper.app/Contents/MacOS/<app> Helper"
    /// in the top-level app bundle. See the comments on [App::execute_process] for
    /// details. Also configurable using the "browser-subprocess-path" command-line
    /// switch.
    pub fn set_browser_subprocess_path<P: AsRef<Path>>(&mut self, path: P) {
        unsafe {
            let path = path.as_ref().to_str().expect("Invalid UTF8");
            cef_string_utf8_to_utf16(
                path.as_ptr() as *const std::os::raw::c_char,
                path.len(),
                &mut self.0.browser_subprocess_path,
            );
        }
    }
    /// The path to the CEF framework directory on macOS. If this value is empty
    /// then the framework must exist at "Contents/Frameworks/Chromium Embedded
    /// Framework.framework" in the top-level app bundle. Also configurable using
    /// the "framework-dir-path" command-line switch.
    #[cfg(target_os = "macos")]
    pub fn set_framework_dir_path<P: AsRef<Path>>(&mut self, path: P) {
        unsafe {
            let path = path.as_ref().to_str().expect("Invalid UTF8");
            cef_string_utf8_to_utf16(
                path.as_ptr() as *const std::os::raw::c_char,
                path.len(),
                &mut self.0.framework_dir_path,
            );
        }
    }
    /// The path to the main bundle on macOS. If this value is empty then it
    /// defaults to the top-level app bundle. Also configurable using
    /// the "main-bundle-path" command-line switch.
    #[cfg(target_os = "macos")]
    pub fn set_main_bundle_path<P: AsRef<Path>>(&mut self, path: P) {
        unsafe {
            let path = path.as_ref().to_str().expect("Invalid UTF8");
            cef_string_utf8_to_utf16(
                path.as_ptr() as *const std::os::raw::c_char,
                path.len(),
                &mut self.0.main_bundle_path,
            );
        }
    }
    /// Call to have the browser process message loop run in a separate
    /// thread. If this is not set, the [App::do_message_loop_work] function must be
    /// called from your application message loop.
    #[cfg(not(target_os = "macos"))]
    pub fn enable_multi_threaded_message_loop(&mut self) {
        self.0.multi_threaded_message_loop = 1;
    }
    /// Call to control browser process main (UI) thread message pump
    /// scheduling via the [BrowserProcessHandlerCallbacks::on_schedule_message_pump_work]
    /// callback. This option is recommended for use in combination with the
    /// [App::do_message_loop_work] function in cases where the CEF message loop must be
    /// integrated into an existing application message loop (see additional
    /// comments and warnings on [App::do_message_loop_work]). Enabling this option is not
    /// recommended for most users; leave this option disabled and use either the
    /// [App::run_message_loop] function or [Settings::enable_multi_threaded_message_loop] if possible.
    pub fn enable_external_message_pump(&mut self) {
        self.0.external_message_pump = 1;
    }
    /// Call to enable windowless (off-screen) rendering support. Do not
    /// enable this value if the application does not use windowless rendering as
    /// it may reduce rendering performance on some systems.
    pub fn enable_windowless_rendering(&mut self) {
        self.0.windowless_rendering_enabled = 1;
    }
    /// Call to disable configuration of browser process features using
    /// standard CEF and Chromium command-line arguments. Configuration can still
    /// be specified using CEF data structures or via the
    /// [App::on_before_command_line_processing] function.
    pub fn disable_command_line_args(&mut self) {
        self.0.command_line_args_disabled = 1;
    }
    /// The location where data for the global browser cache will be stored on
    /// disk. If non-empty this must be either equal to or a child directory of
    /// [Settings::set_root_cache_path]. If empty then browsers will be created in
    /// "incognito mode" where in-memory caches are used for storage and no data is
    /// persisted to disk. HTML5 databases such as localStorage will only persist
    /// across sessions if a cache path is specified. Can be overridden for
    /// individual [RequestContext] instances via the
    /// [RequestContextSettings::cache_path] value.
    pub fn set_cache_path<P: AsRef<Path>>(&mut self, path: P) {
        unsafe {
            let path = path.as_ref().to_str().expect("Invalid UTF8");
            cef_string_utf8_to_utf16(
                path.as_ptr() as *const std::os::raw::c_char,
                path.len(),
                &mut self.0.cache_path,
            );
        }
    }
    /// The root directory that all [Settings::set_cache_path] and
    /// [RequestContextSettings::cache_path] values must have in common. If this
    /// value is empty and [Settings::set_cache_path] is non-empty then this value will
    /// default to the [Settings::set_cache_path] value. Failure to set this value
    /// correctly may result in the sandbox blocking read/write access to the
    /// cache_path directory.
    pub fn set_root_cache_path<P: AsRef<Path>>(&mut self, path: P) {
        unsafe {
            let path = path.as_ref().to_str().expect("Invalid UTF8");
            cef_string_utf8_to_utf16(
                path.as_ptr() as *const std::os::raw::c_char,
                path.len(),
                &mut self.0.root_cache_path,
            );
        }
    }
    /// The location where user data such as spell checking dictionary files will
    /// be stored on disk. If empty then the default platform-specific user data
    /// directory will be used ("~/.cef_user_data" directory on Linux,
    /// "~/Library/Application Support/CEF/User Data" directory on Mac OS X,
    /// "Local Settings\Application Data\CEF\User Data" directory under the user
    /// profile directory on Windows).
    pub fn set_user_data_path<P: AsRef<Path>>(&mut self, path: P) {
        unsafe {
            let path = path.as_ref().to_str().expect("Invalid UTF8");
            cef_string_utf8_to_utf16(
                path.as_ptr() as *const std::os::raw::c_char,
                path.len(),
                &mut self.0.user_data_path,
            );
        }
    }
    /// To persist session cookies (cookies without an expiry date or validity
    /// interval) by default when using the global cookie manager call this function.
    /// Session cookies are generally intended to be transient and most
    /// Web browsers do not persist them. A [Settings::set_cache_path] value must also be
    /// specified to enable this feature. Also configurable using the
    /// "persist-session-cookies" command-line switch. Can be overridden for
    /// individual [RequestContext] instances via the
    /// [RequestContextSettings::persist_session_cookies] value.
    pub fn persist_session_cookies(&mut self) {
        self.0.persist_session_cookies = 1;
    }
    /// To persist user preferences as a JSON file in the cache path directory call
    /// this function. A [Settings::set_cache_path] value must also be specified
    /// to enable this feature. Also configurable using the
    /// "persist-user-preferences" command-line switch. Can be overridden for
    /// individual CefRequestContext instances via the
    /// [RequestContextSettings::persist_user_preferences] value.
    pub fn persist_user_preferences(&mut self) {
        self.0.persist_user_preferences = 1;
    }
    /// Value that will be returned as the User-Agent HTTP header. If empty the
    /// default User-Agent string will be used. Also configurable using the
    /// "user-agent" command-line switch.
    pub fn set_user_agent(&mut self, agent: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                agent.as_ptr() as *const std::os::raw::c_char,
                agent.len(),
                &mut self.0.user_agent,
            );
        }
    }
    /// Value that will be inserted as the product portion of the default
    /// User-Agent string. If empty the Chromium product version will be used. If
    /// [Settings::set_user_agent] is specified this value will be ignored. Also configurable
    /// using the "product-version" command-line switch.
    pub fn set_product_version(&mut self, version: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                version.as_ptr() as *const std::os::raw::c_char,
                version.len(),
                &mut self.0.product_version,
            );
        }
    }
    /// The locale string that will be passed to WebKit. If empty the default
    /// locale of "en-US" will be used. This value is ignored on Linux where locale
    /// is determined using environment variable parsing with the precedence order:
    /// LANGUAGE, LC_ALL, LC_MESSAGES and LANG. Also configurable using the "lang"
    /// command-line switch.
    pub fn set_locale(&mut self, locale: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                locale.as_ptr() as *const std::os::raw::c_char,
                locale.len(),
                &mut self.0.locale,
            );
        }
    }
    /// The directory and file name to use for the debug log. If empty a default
    /// log file name and location will be used. On Windows and Linux a "debug.log"
    /// file will be written in the main executable directory. On Mac OS X a
    /// "~/Library/Logs/<app name>_debug.log" file will be written where <app name>
    /// is the name of the main app executable. Also configurable using the
    /// "log-file" command-line switch.
    pub fn set_log_file(&mut self, file: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                file.as_ptr() as *const std::os::raw::c_char,
                file.len(),
                &mut self.0.log_file,
            );
        }
    }
    /// The log severity. Only messages of this severity level or higher will be
    /// logged. When set to DISABLE no messages will be written to the log file,
    /// but FATAL messages will still be output to stderr. Also configurable using
    /// the "log-severity" command-line switch with a value of "verbose", "info",
    /// "warning", "error", "fatal" or "disable".
    pub fn set_log_severity(&mut self, severity: LogSeverity) {
        self.0.log_severity = severity as cef_log_severity_t::Type;
    }
    /// Custom flags that will be used when initializing the V8 JavaScript engine.
    /// The consequences of using custom flags may not be well tested. Also
    /// configurable using the "js-flags" command-line switch.
    pub fn set_javascript_flags(&mut self, flags: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                flags.as_ptr() as *const std::os::raw::c_char,
                flags.len(),
                &mut self.0.javascript_flags,
            );
        }
    }
    /// The fully qualified path for the resources directory. If this value is
    /// empty the cef.pak and/or devtools_resources.pak files must be located in
    /// the module directory on Windows/Linux or the app bundle Resources directory
    /// on Mac OS X. Also configurable using the "resources-dir-path" command-line
    /// switch.
    pub fn set_resources_dir_path<P: AsRef<Path>>(&mut self, path: P) {
        unsafe {
            let path = path.as_ref().to_str().expect("Invalid UTF8");
            cef_string_utf8_to_utf16(
                path.as_ptr() as *const std::os::raw::c_char,
                path.len(),
                &mut self.0.resources_dir_path,
            );
        }
    }
    /// The fully qualified path for the locales directory. If this value is empty
    /// the locales directory must be located in the module directory. This value
    /// is ignored on Mac OS X where pack files are always loaded from the app
    /// bundle Resources directory. Also configurable using the "locales-dir-path"
    /// command-line switch.
    pub fn set_locales_dir_path<P: AsRef<Path>>(&mut self, path: P) {
        unsafe {
            let path = path.as_ref().to_str().expect("Invalid UTF8");
            cef_string_utf8_to_utf16(
                path.as_ptr() as *const std::os::raw::c_char,
                path.len(),
                &mut self.0.locales_dir_path,
            );
        }
    }
    /// Call to disable loading of pack files for resources and locales.
    /// A resource bundle handler must be provided for the browser and render
    /// processes via [App::get_resource_bundle_handler] if loading of pack files
    /// is disabled. Also configurable using the "disable-pack-loading" command-
    /// line switch.
    pub fn disable_pack_loading(&mut self) {
        self.0.pack_loading_disabled = 1;
    }
    /// Set to a value between 1024 and 65535 to enable remote debugging on the
    /// specified port. For example, if 8080 is specified the remote debugging URL
    /// will be http://localhost:8080. CEF can be remotely debugged from any CEF or
    /// Chrome browser window. Also configurable using the "remote-debugging-port"
    /// command-line switch.
    pub fn set_remote_debugging_port(&mut self, port: u16) {
        self.0.remote_debugging_port = port as i32;
    }
    /// The number of stack trace frames to capture for uncaught exceptions.
    /// Specify a positive value to enable the [RenderProcessHandlerCallbacks::on_uncaught_exception] callback. Specify 0 (default value) and
    /// [RenderProcessHandlerCallbacks::on_uncaught_exception] will not be called. Also configurable using the
    /// "uncaught-exception-stack-size" command-line switch.
    pub fn set_uncaught_exception_stack_size(&mut self, stack_size: i32) {
        self.0.uncaught_exception_stack_size = stack_size;
    }
    /// Call to ignore errors related to invalid SSL certificates.
    /// Enabling this setting can lead to potential security vulnerabilities like
    /// "man in the middle" attacks. Applications that load content from the
    /// internet should not enable this setting. Also configurable using the
    /// "ignore-certificate-errors" command-line switch. Can be overridden for
    /// individual [RequestContext] instances via the
    /// [RequestContextSettings::ignore_certificate_errors] value.
    pub fn ignore_certificate_errors(&mut self) {
        self.0.ignore_certificate_errors = 1;
    }
    /// Call to enable date-based expiration of built in network
    /// security information (i.e. certificate transparency logs, HSTS preloading
    /// and pinning information). Enabling this option improves network security
    /// but may cause HTTPS load failures when using CEF binaries built more than
    /// 10 weeks in the past. See https://www.certificate-transparency.org/ and
    /// https://www.chromium.org/hsts for details. Also configurable using the
    /// "enable-net-security-expiration" command-line switch. Can be overridden for
    /// individual [RequestContext] instances via the
    /// [RequestContextSettings::enable_net_security_expiration] value.
    pub fn enable_net_security_expiration(&mut self) {
        self.0.enable_net_security_expiration = 1;
    }
    /// Background color used for the browser before a document is loaded and when
    /// no document color is specified. The alpha component must be either fully
    /// opaque (0xFF) or fully transparent (0x00). If the alpha component is fully
    /// opaque then the RGB components will be used as the background color. If the
    /// alpha component is fully transparent for a windowed browser then the
    /// default value of opaque white be used. If the alpha component is fully
    /// transparent for a windowless (off-screen) browser then transparent painting
    /// will be enabled.
    pub fn set_background_color(&mut self, color: Color) {
        self.0.background_color = color.get();
    }
    /// Comma delimited ordered list of language codes without any whitespace that
    /// will be used in the "Accept-Language" HTTP header. May be overridden on a
    /// per-browser basis using the [BrowserSettings::accept_language_list] value.
    /// If both values are empty then "en-US,en" will be used. Can be overridden
    /// for individual [RequestContext] instances via the
    /// [RequestContextSettings::accept_language_list] value.
    pub fn set_accept_language_list(&mut self, list: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                list.as_ptr() as *const std::os::raw::c_char,
                list.len(),
                &mut self.0.accept_language_list,
            );
        }
    }
    /// GUID string used for identifying the application. This is passed to the
    /// system AV function for scanning downloaded files. By default, the GUID
    /// will be an empty string and the file will be treated as an untrusted
    /// file when the GUID is empty.
    pub fn set_application_client_id_for_file_scanning(&mut self, guid: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                guid.as_ptr() as *const std::os::raw::c_char,
                guid.len(),
                &mut self.0.application_client_id_for_file_scanning,
            );
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        let settings = &self.0;
        for cefstr in &[
            &settings.browser_subprocess_path,
            &settings.framework_dir_path,
            &settings.main_bundle_path,
            &settings.cache_path,
            &settings.root_cache_path,
            &settings.user_data_path,
            &settings.user_agent,
            &settings.product_version,
            &settings.locale,
            &settings.log_file,
            &settings.javascript_flags,
            &settings.resources_dir_path,
            &settings.locales_dir_path,
            &settings.accept_language_list,
            &settings.application_client_id_for_file_scanning,
        ] {
            if let Some(dtor) = cefstr.dtor {
                unsafe {
                    dtor(cefstr.str);
                }
            }
        }
    }
}
