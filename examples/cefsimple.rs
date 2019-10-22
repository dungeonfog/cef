use std::sync::Arc;
#[cfg(windows)]
use winapi::um::{
    libloaderapi::GetModuleHandleA,
    winuser::{WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_OVERLAPPEDWINDOW, WS_VISIBLE},
};

pub struct AppCallbacks();

impl cef::AppCallbacks for AppCallbacks {}

pub struct ClientCallbacks();

impl cef::ClientCallbacks for ClientCallbacks {}

fn main() {
    let app = cef::App::new(Arc::new(AppCallbacks()));
    #[cfg(windows)]
    cef::App::enable_highdpi_support();
    let args = cef::MainArgs::new(unsafe { GetModuleHandleA(std::ptr::null()) });
    let result = cef::App::execute_process(&args, Some(&app), None);
    if result >= 0 {
        std::process::exit(result);
    }
    let mut settings = cef::Settings::new();
    settings.set_log_severity(cef::LogSeverity::Verbose);
    settings.disable_sandbox();
    let resources_folder = std::env::var("CEFSIMPLE_ABSOLUTE_RESOURCES_PATH")
        .expect("CEFSIMPLE_ABSOLUTE_RESOURCES_PATH must be set to a valid CEF resources folder");
    settings.set_resources_dir_path(&resources_folder);

    cef::App::initialize(&args, &settings, Some(&app), None);

    let mut window_info = cef::WindowInfo::new();
    #[cfg(windows)]
    window_info.set_style(WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE);
    window_info.set_window_name("cefsimple Rust example");
    let browser_settings = cef::BrowserSettings::new();

    let client = Arc::new(ClientCallbacks());

    cef::BrowserHost::create_browser(
        &window_info,
        client,
        "https://www.youtube.com",
        &browser_settings,
        None,
        None,
    );

    cef::App::run_message_loop();

    cef::App::shutdown();
}
