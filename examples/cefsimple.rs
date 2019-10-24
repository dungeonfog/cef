#[cfg(windows)]
use winapi::um::{
    libloaderapi::GetModuleHandleA,
    winuser::{WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_OVERLAPPEDWINDOW, WS_VISIBLE},
};
use cef::client::life_span_handler::{LifeSpanHandler, LifeSpanHandlerCallbacks};

pub struct AppCallbacks {}

impl cef::AppCallbacks for AppCallbacks {}

pub struct ClientCallbacks {
    life_span_handler: LifeSpanHandler,
}

impl cef::ClientCallbacks for ClientCallbacks {
    fn get_life_span_handler(&self) -> Option<LifeSpanHandler> {
        Some(self.life_span_handler.clone())
    }
}

pub struct SimpleLifeSpanHandler {}

impl LifeSpanHandlerCallbacks for SimpleLifeSpanHandler {
    fn on_before_close(&self, _browser: cef::Browser) {
        cef::App::quit_message_loop()
    }
}

fn main() {
    let app = cef::App::new(Box::new(AppCallbacks {}));
    #[cfg(windows)]
    cef::App::enable_highdpi_support();
    let args = cef::MainArgs::new(unsafe { GetModuleHandleA(std::ptr::null()) });
    let result = cef::App::execute_process(&args, Some(app.clone()), None);
    if result >= 0 {
        std::process::exit(result);
    }
    let mut settings = cef::Settings::new();
    settings.set_log_severity(cef::LogSeverity::Verbose);
    settings.disable_sandbox();
    let resources_folder = std::path::Path::new("./Resources").canonicalize().unwrap();
    settings.set_resources_dir_path(&resources_folder);

    cef::App::initialize(&args, &settings, Some(app), None);

    let mut window_info = cef::WindowInfo::new();
    #[cfg(windows)] {
        window_info.style = WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;
    }
    window_info.window_name = "cefsimple Rust example".into();
    window_info.width = 500;
    window_info.height = 500;
    let browser_settings = cef::BrowserSettings::new();

    let client = Box::new(ClientCallbacks {
        life_span_handler: LifeSpanHandler::new(SimpleLifeSpanHandler {})
    });

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
