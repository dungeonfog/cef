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
        cef::quit_message_loop().unwrap();
    }
}

fn main() {
    let app = App::new(AppCallbacksImpl {});
    let result = cef::execute_process(Some(app.clone()), None);
    if result >= 0 {
        std::process::exit(result);
    }

    // TODO(yanchith): aren't we missing browser_subprocess_path on macos?
    #[cfg(not(target_os = "macos"))]
    let settings = Settings::new("./Resources")
        .log_severity(LogSeverity::Verbose);
    #[cfg(target_os = "macos")]
    let settings = Settings::new("./Chromium Embedded Framework.framework/Resources")
        .log_severity(LogSeverity::Verbose)
        .framework_dir_path("./Chromium Embedded Framework.framework");

    let context = cef::Context::initialize(&settings, Some(app), None).unwrap();

    let mut window_info = WindowInfo::new();
    #[cfg(windows)] {
        window_info.platform_specific.style = WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;
    }
    window_info.window_name = "cefsimple Rust example".into();
    window_info.width = 500;
    window_info.height = 500;
    let browser_settings = BrowserSettings::new();

    let client = Client::new(ClientCallbacksImpl {
        life_span_handler: LifeSpanHandler::new(LifeSpanHandlerImpl {})
    });

    let _browser = BrowserHost::create_browser_sync(
        &window_info,
        client,
        "https://www.youtube.com",
        &browser_settings,
        None,
        None,
        &context,
    );

    context.run_message_loop();
}
