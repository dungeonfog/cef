#[cfg(windows)]
use winapi::um::winuser::{WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_OVERLAPPEDWINDOW, WS_VISIBLE};
use cef::{
    app::{App, AppCallbacks},
    browser::{Browser, BrowserSettings},
    browser_host::BrowserHost,
    client::{
        Client, ClientCallbacks,
        life_span_handler::{LifeSpanHandler, LifeSpanHandlerCallbacks}
    },
    main_args::MainArgs,
    settings::{Settings, LogSeverity},
    window::WindowInfo,
};

pub struct AppCallbacksImpl {}

impl AppCallbacks for AppCallbacksImpl {}

pub struct ClientCallbacksImpl {
    life_span_handler: LifeSpanHandler,
}

impl ClientCallbacks for ClientCallbacksImpl {
    fn get_life_span_handler(&self) -> Option<LifeSpanHandler> {
        Some(self.life_span_handler.clone())
    }
}

pub struct LifeSpanHandlerImpl {}

impl LifeSpanHandlerCallbacks for LifeSpanHandlerImpl {
    fn on_before_close(&self, _browser: Browser) {
        cef::quit_message_loop()
    }
}

fn main() {
    let app = App::new(AppCallbacksImpl {});
    #[cfg(windows)]
    cef::enable_highdpi_support();
    let args = MainArgs::new();
    let result = cef::execute_process(&args, Some(app.clone()), None);
    if result >= 0 {
        std::process::exit(result);
    }
    let mut settings = Settings::new();
    settings.set_log_severity(LogSeverity::Disable);
    settings.disable_sandbox();
    let resources_folder = std::path::Path::new("./Resources").canonicalize().unwrap();
    settings.set_resources_dir_path(&resources_folder);

    cef::initialize(&args, &settings, Some(app), None);

    let mut window_info = WindowInfo::new();
    #[cfg(windows)] {
        window_info.style = WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;
    }
    window_info.window_name = "cefsimple Rust example".into();
    window_info.width = 500;
    window_info.height = 500;
    let browser_settings = BrowserSettings::new();

    let client = Client::new(ClientCallbacksImpl {
        life_span_handler: LifeSpanHandler::new(LifeSpanHandlerImpl {})
    });

    BrowserHost::create_browser(
        &window_info,
        client,
        "https://www.youtube.com",
        &browser_settings,
        None,
        None,
    );

    cef::run_message_loop();

    cef::shutdown();
}
