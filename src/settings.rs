use std::os::raw::c_int;
use crate::{
    color::Color,
    string::CefString,
};
use cef_sys::{cef_log_severity_t, cef_settings_t};
use std::path::PathBuf;
use uuid::Uuid;

/// Log severity levels.
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LogSeverity {
    /// Default logging (currently INFO logging).
    Default = cef_log_severity_t::LOGSEVERITY_DEFAULT as isize,
    /// Verbose logging (same as DEBUG logging).
    Verbose = cef_log_severity_t::LOGSEVERITY_VERBOSE as isize,
    /// INFO logging.
    Info = cef_log_severity_t::LOGSEVERITY_INFO as isize,
    /// WARNING logging.
    Warning = cef_log_severity_t::LOGSEVERITY_WARNING as isize,
    /// ERROR logging.
    Error = cef_log_severity_t::LOGSEVERITY_ERROR as isize,
    /// FATAL logging.
    Fatal = cef_log_severity_t::LOGSEVERITY_FATAL as isize,
    /// Disable logging to file for all messages, and to stderr for messages with
    /// severity less than FATAL.
    Disable = cef_log_severity_t::LOGSEVERITY_DISABLE as isize,
}

impl LogSeverity {
    pub unsafe fn from_unchecked(c: crate::CEnumType) -> Self {
        std::mem::transmute(c)
    }
}

pub struct Settings {
    /// Set the path to a separate executable that will be launched for sub-processes.
    /// If this value is empty on Windows or Linux then the main process executable
    /// will be used. If this value is empty on macOS then a helper executable must
    /// exist at "Contents/Frameworks/<app> Helper.app/Contents/MacOS/<app> Helper"
    /// in the top-level app bundle. See the comments on [App::execute_process] for
    /// details. Also configurable using the "browser-subprocess-path" command-line
    /// switch.
    pub browser_subprocess_path: Option<PathBuf>,
    /// The path to the CEF framework directory on macOS. If this value is empty
    /// then the framework must exist at "Contents/Frameworks/Chromium Embedded
    /// Framework.framework" in the top-level app bundle. Also configurable using
    /// the "framework-dir-path" command-line switch.
    ///
    /// Only applies on macOS.
    pub framework_dir_path: Option<PathBuf>,
    /// The path to the main bundle on macOS. If this value is empty then it
    /// defaults to the top-level app bundle. Also configurable using
    /// the "main-bundle-path" command-line switch.
    ///
    /// Only applies on macOS.
    pub main_bundle_path: Option<PathBuf>,
    /// Call to have the browser process message loop run in a separate
    /// thread. If this is not set, the [App::do_message_loop_work] function must be
    /// called from your application message loop.
    ///
    /// Doesn't apply on macOS.
    pub multi_threaded_message_loop: bool,
    /// Call to control browser process main (UI) thread message pump
    /// scheduling via the [BrowserProcessHandlerCallbacks::on_schedule_message_pump_work]
    /// callback. This option is recommended for use in combination with the
    /// [App::do_message_loop_work] function in cases where the CEF message loop must be
    /// integrated into an existing application message loop (see additional
    /// comments and warnings on [App::do_message_loop_work]). Enabling this option is not
    /// recommended for most users; leave this option disabled and use either the
    /// [App::run_message_loop] function or [Settings::enable_multi_threaded_message_loop] if possible.
    pub external_message_pump: bool,
    /// Call to enable windowless (off-screen) rendering support. Do not
    /// enable this value if the application does not use windowless rendering as
    /// it may reduce rendering performance on some systems.
    pub windowless_rendering_enabled: bool,
    /// Call to disable configuration of browser process features using
    /// standard CEF and Chromium command-line arguments. Configuration can still
    /// be specified using CEF data structures or via the
    /// [App::on_before_command_line_processing] function.
    pub command_line_args_disabled: bool,
    /// The location where data for the global browser cache will be stored on
    /// disk. If non-empty this must be either equal to or a child directory of
    /// [Settings::set_root_cache_path]. If empty then browsers will be created in
    /// "incognito mode" where in-memory caches are used for storage and no data is
    /// persisted to disk. HTML5 databases such as localStorage will only persist
    /// across sessions if a cache path is specified. Can be overridden for
    /// individual [RequestContext] instances via the
    /// [RequestContextSettings::cache_path] value.
    pub cache_path: Option<PathBuf>,
    /// The root directory that all [Settings::set_cache_path] and
    /// [RequestContextSettings::cache_path] values must have in common. If this
    /// value is empty and [Settings::set_cache_path] is non-empty then this value will
    /// default to the [Settings::set_cache_path] value. Failure to set this value
    /// correctly may in the sandbox blocking read/he
    /// cache_path directory.
    pub root_cache_path: Option<PathBuf>,
    /// The location where user data such as spell checking dictionary files will
    /// be stored on disk. If empty then the default platform-specific user data
    /// directory will be used ("~/.cef_user_data" directory on Linux,
    /// "~/Library/Application Support/CEF/User Data" directory on Mac OS X,
    /// "Local Settings\Application Data\CEF\User Data" directory under the user
    /// profile directory on Windows).
    pub user_data_path: Option<PathBuf>,
    /// To persist session cookies (cookies without an expiry date or validity
    /// interval) by default when using the global cookie manager call this function.
    /// Session cookies are generally intended to be transient and most
    /// Web browsers do not persist them. A [Settings::set_cache_path] value must also be
    /// specified to enable this feature. Also configurable using the
    /// "persist-session-cookies" command-line switch. Can be overridden for
    /// individual [RequestContext] instances via the
    /// [RequestContextSettings::persist_session_cookies] value.
    pub persist_session_cookies: bool,
    /// To persist user preferences as a JSON file in the cache path directory call
    /// this function. A [Settings::set_cache_path] value must also be specified
    /// to enable this feature. Also configurable using the
    /// "persist-user-preferences" command-line switch. Can be overridden for
    /// individual CefRequestContext instances via the
    /// [RequestContextSettings::persist_user_preferences] value.
    pub persist_user_preferences: bool,
    /// Value that will be returned as the User-Agent HTTP header. If empty the
    /// default User-Agent string will be used. Also configurable using the
    /// "user-agent" command-line switch.
    pub user_agent: Option<String>,
    /// Value that will be inserted as the product portion of the default
    /// User-Agent string. If empty the Chromium product version will be used. If
    /// [Settings::set_user_agent] is specified this value will be ignored. Also configurable
    /// using the "product-version" command-line switch.
    pub product_version: Option<String>,
    /// The locale string that will be passed to WebKit. If empty the default
    /// locale of "en-US" will be used. This value is ignored on Linux where locale
    /// is determined using environment variable parsing with the precedence order:
    /// LANGUAGE, LC_ALL, LC_MESSAGES and LANG. Also configurable using the "lang"
    /// command-line switch.
    pub locale: Option<String>,
    /// The directory and file name to use for the debug log. If empty a default
    /// log file name and location will be used. On Windows and Linux a "debug.log"
    /// file will be written in the main executable directory. On Mac OS X a
    /// "~/Library/Logs/<app name>_debug.log" file will be written where <app name>
    /// is the name of the main app executable. Also configurable using the
    /// "log-file" command-line switch.
    pub log_file: Option<PathBuf>,
    /// The log severity. Only messages of this severity level or higher will be
    /// logged. When set to DISABLE no messages will be written to the log file,
    /// but FATAL messages will still be output to stderr. Also configurable using
    /// the "log-severity" command-line switch with a value of "verbose", "info",
    /// "warning", "error", "fatal" or "disable".
    pub log_severity: LogSeverity,
    /// Custom flags that will be used when initializing the V8 JavaScript engine.
    /// The consequences of using custom flags may not be well tested. Also
    /// configurable using the "js-flags" command-line switch.
    pub javascript_flags: Option<String>,
    /// The fully qualified path for the resources directory. If this value is
    /// empty the cef.pak and/or devtools_resources.pak files must be located in
    /// the module directory on Windows/Linux or the app bundle Resources directory
    /// on Mac OS X. Also configurable using the "resources-dir-path" command-line
    /// switch.
    pub resources_dir_path: PathBuf,
    /// The fully qualified path for the locales directory. If this value is empty
    /// the locales directory must be located in the module directory. This value
    /// is ignored on Mac OS X where pack files are always loaded from the app
    /// bundle Resources directory. Also configurable using the "locales-dir-path"
    /// command-line switch.
    pub locales_dir_path: Option<PathBuf>,
    /// Call to disable loading of pack files for resources and locales.
    /// A resource bundle handler must be provided for the browser and render
    /// processes via [App::get_resource_bundle_handler] if loading of pack files
    /// is disabled. Also configurable using the "disable-pack-loading" command-
    /// line switch.
    pub pack_loading_disabled: bool,
    /// Set to a value between 1024 and 65535 to enable remote debugging on the
    /// specified port. For example, if 8080 is specified the remote debugging URL
    /// will be http://localhost:8080. CEF can be remotely debugged from any CEF or
    /// Chrome browser window. Also configurable using the "remote-debugging-port"
    /// command-line switch.
    pub remote_debugging_port: u16,
    /// The number of stack trace frames to capture for uncaught exceptions.
    /// Specify a positive value to enable the [RenderProcessHandlerCallbacks::on_uncaught_exception] callback. Specify 0 (default value) and
    /// [RenderProcessHandlerCallbacks::on_uncaught_exception] will not be called. Also configurable using the
    /// "uncaught-exception-stack-size" command-line switch.
    pub uncaught_exception_stack_size: u32,
    /// Call to ignore errors related to invalid SSL certificates.
    /// Enabling this setting can lead to potential security vulnerabilities like
    /// "man in the middle" attacks. Applications that load content from the
    /// internet should not enable this setting. Also configurable using the
    /// "ignore-certificate-errors" command-line switch. Can be overridden for
    /// individual [RequestContext] instances via the
    /// [RequestContextSettings::ignore_certificate_errors] value.
    pub ignore_certificate_errors: bool,
    /// Background color used for the browser before a document is loaded and when
    /// no document color is specified. The alpha component must be either fully
    /// opaque (0xFF) or fully transparent (0x00). If the alpha component is fully
    /// opaque then the RGB components will be used as the background color. If the
    /// alpha component is fully transparent for a windowed browser then the
    /// default value of opaque white be used. If the alpha component is fully
    /// transparent for a windowless (off-screen) browser then transparent painting
    /// will be enabled.
    pub background_color: Color,
    /// Comma delimited ordered list of language codes without any whitespace that
    /// will be used in the "Accept-Language" HTTP header. May be overridden on a
    /// per-browser basis using the [BrowserSettings::accept_language_list] value.
    /// If both values are empty then "en-US,en" will be used. Can be overridden
    /// for individual [RequestContext] instances via the
    /// [RequestContextSettings::accept_language_list] value.
    pub accept_language_list: Option<String>,
    /// GUID string used for identifying the application. This is passed to the
    /// system AV function for scanning downloaded files. By default, the GUID
    /// will be an empty string and the file will be treated as an untrusted
    /// file when the GUID is empty.
    pub application_client_id_for_file_scanning: Option<Uuid>,
}

impl Settings {
    pub fn new<T: Into<PathBuf>>(resources_dir_path: T) -> Settings {
        Settings {
            browser_subprocess_path: None,
            framework_dir_path: None,
            main_bundle_path: None,
            multi_threaded_message_loop: false,
            external_message_pump: false,
            windowless_rendering_enabled: false,
            command_line_args_disabled: false,
            cache_path: None,
            root_cache_path: None,
            user_data_path: None,
            persist_session_cookies: false,
            persist_user_preferences: false,
            user_agent: None,
            product_version: None,
            locale: None,
            log_file: None,
            log_severity: LogSeverity::Default,
            javascript_flags: None,
            resources_dir_path: resources_dir_path.into(),
            locales_dir_path: None,
            pack_loading_disabled: false,
            remote_debugging_port: 0,
            uncaught_exception_stack_size: 0,
            ignore_certificate_errors: false,
            background_color: Color::wrap(0),
            accept_language_list: None,
            application_client_id_for_file_scanning: None,
        }
    }
    pub(crate) fn to_cef(&self, use_sandbox: bool) -> Result<cef_settings_t, std::io::Error> {
        let string_to_cef = |s: Option<&String>| s.map(|s| &**s).map(CefString::new).unwrap_or_else(CefString::null).into_raw();
        let path_to_cef = |path: Option<&PathBuf>| -> Result<_, std::io::Error> {Ok(path.map(|p| p.canonicalize()).transpose()?.as_ref().map(|p| p.to_str().unwrap()).map(CefString::new).unwrap_or_else(CefString::null).into_raw())};
        Ok(cef_settings_t {
            size: std::mem::size_of::<cef_settings_t>(),
            no_sandbox: !use_sandbox as c_int,
            browser_subprocess_path: path_to_cef(self.browser_subprocess_path.as_ref())?,
            framework_dir_path: path_to_cef(self.framework_dir_path.as_ref())?,
            main_bundle_path: path_to_cef(self.main_bundle_path.as_ref())?,
            multi_threaded_message_loop: self.multi_threaded_message_loop as c_int,
            external_message_pump: self.external_message_pump as c_int,
            windowless_rendering_enabled: self.windowless_rendering_enabled as c_int,
            command_line_args_disabled: self.command_line_args_disabled as c_int,
            cache_path: path_to_cef(self.cache_path.as_ref())?,
            root_cache_path: path_to_cef(self.root_cache_path.as_ref())?,
            user_data_path: path_to_cef(self.user_data_path.as_ref())?,
            persist_session_cookies: self.persist_session_cookies as c_int,
            persist_user_preferences: self.persist_user_preferences as c_int,
            user_agent: string_to_cef(self.user_agent.as_ref()),
            product_version: string_to_cef(self.product_version.as_ref()),
            locale: string_to_cef(self.locale.as_ref()),
            log_file: path_to_cef(self.log_file.as_ref())?,
            log_severity: cef_log_severity_t::LOGSEVERITY_DEFAULT,
            javascript_flags: string_to_cef(self.javascript_flags.as_ref()),
            resources_dir_path: path_to_cef(Some(&self.resources_dir_path))?,
            locales_dir_path: path_to_cef(self.locales_dir_path.as_ref())?,
            pack_loading_disabled: self.pack_loading_disabled as c_int,
            remote_debugging_port: self.remote_debugging_port as c_int,
            uncaught_exception_stack_size: self.uncaught_exception_stack_size as c_int,
            ignore_certificate_errors: self.ignore_certificate_errors as c_int,
            background_color: self.background_color.0,
            accept_language_list: string_to_cef(self.accept_language_list.as_ref()),
            application_client_id_for_file_scanning: string_to_cef(self.application_client_id_for_file_scanning.map(|u| u.to_string()).as_ref()),
        })
    }
    pub fn browser_subprocess_path<T: Into<PathBuf>>(mut self, browser_subprocess_path: T) -> Self {
        self.browser_subprocess_path = Some(browser_subprocess_path.into());
        self
    }
    pub fn framework_dir_path<T: Into<PathBuf>>(mut self, framework_dir_path: T) -> Self {
        self.framework_dir_path = Some(framework_dir_path.into());
        self
    }
    pub fn main_bundle_path<T: Into<PathBuf>>(mut self, main_bundle_path: T) -> Self {
        self.main_bundle_path = Some(main_bundle_path.into());
        self
    }
    pub fn multi_threaded_message_loop(mut self, multi_threaded_message_loop: bool) -> Self {
        self.multi_threaded_message_loop = multi_threaded_message_loop;
        self
    }
    pub fn external_message_pump(mut self, external_message_pump: bool) -> Self {
        self.external_message_pump = external_message_pump;
        self
    }
    pub fn windowless_rendering_enabled(mut self, windowless_rendering_enabled: bool) -> Self {
        self.windowless_rendering_enabled = windowless_rendering_enabled;
        self
    }
    pub fn command_line_args_disabled(mut self, command_line_args_disabled: bool) -> Self {
        self.command_line_args_disabled = command_line_args_disabled;
        self
    }
    pub fn cache_path<T: Into<PathBuf>>(mut self, cache_path: T) -> Self {
        self.cache_path = Some(cache_path.into());
        self
    }
    pub fn root_cache_path<T: Into<PathBuf>>(mut self, root_cache_path: T) -> Self {
        self.root_cache_path = Some(root_cache_path.into());
        self
    }
    pub fn user_data_path<T: Into<PathBuf>>(mut self, user_data_path: T) -> Self {
        self.user_data_path = Some(user_data_path.into());
        self
    }
    pub fn persist_session_cookies(mut self, persist_session_cookies: bool) -> Self {
        self.persist_session_cookies = persist_session_cookies;
        self
    }
    pub fn persist_user_preferences(mut self, persist_user_preferences: bool) -> Self {
        self.persist_user_preferences = persist_user_preferences;
        self
    }
    pub fn user_agent<T: Into<String>>(mut self, user_agent: T) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }
    pub fn product_version<T: Into<String>>(mut self, product_version: T) -> Self {
        self.product_version = Some(product_version.into());
        self
    }
    pub fn locale<T: Into<String>>(mut self, locale: T) -> Self {
        self.locale = Some(locale.into());
        self
    }
    pub fn log_file<T: Into<PathBuf>>(mut self, log_file: T) -> Self {
        self.log_file = Some(log_file.into());
        self
    }
    pub fn log_severity(mut self, log_severity: LogSeverity) -> Self {
        self.log_severity = log_severity;
        self
    }
    pub fn javascript_flags<T: Into<String>>(mut self, javascript_flags: T) -> Self {
        self.javascript_flags = Some(javascript_flags.into());
        self
    }
    pub fn locales_dir_path<T: Into<PathBuf>>(mut self, locales_dir_path: T) -> Self {
        self.locales_dir_path = Some(locales_dir_path.into());
        self
    }
    pub fn pack_loading_disabled(mut self, pack_loading_disabled: bool) -> Self {
        self.pack_loading_disabled = pack_loading_disabled;
        self
    }
    pub fn remote_debugging_port(mut self, remote_debugging_port: u16) -> Self {
        self.remote_debugging_port = remote_debugging_port;
        self
    }
    pub fn uncaught_exception_stack_size(mut self, uncaught_exception_stack_size: u32) -> Self {
        self.uncaught_exception_stack_size = uncaught_exception_stack_size;
        self
    }
    pub fn ignore_certificate_errors(mut self, ignore_certificate_errors: bool) -> Self {
        self.ignore_certificate_errors = ignore_certificate_errors;
        self
    }
    pub fn background_color(mut self, background_color: Color) -> Self {
        self.background_color = background_color;
        self
    }
    pub fn accept_language_list<T: Into<String>>(mut self, accept_language_list: T) -> Self {
        self.accept_language_list = Some(accept_language_list.into());
        self
    }
    pub fn application_client_id_for_file_scanning<T: Into<Uuid>>(mut self, application_client_id_for_file_scanning: T) -> Self {
        self.application_client_id_for_file_scanning = Some(application_client_id_for_file_scanning.into());
        self
    }
}

pub(crate) unsafe fn drop_settings(settings: cef_settings_t) {
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
            dtor(cefstr.str);
        }
    }
}
