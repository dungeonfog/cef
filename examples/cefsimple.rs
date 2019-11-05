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
use std::sync::Arc;

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

pub struct LifeSpanHandlerImpl {
    context: Arc<cef::Context>,
}

impl LifeSpanHandlerCallbacks for LifeSpanHandlerImpl {
    fn on_before_close(&self, _browser: Browser) {
        self.context.quit_message_loop();
    }
}

fn main() {
    let app = App::new(AppCallbacksImpl {});
    #[cfg(windows)]
    cef::enable_highdpi_support();
    let args = MainArgs::new();
    #[cfg(target_os = "windows")]
    let result = cef::execute_process(&args, Some(app.clone()), None);
    #[cfg(not(target_os = "windows"))]
    let result = cef::execute_process(&args, Some(app.clone()));
    if result >= 0 {
        std::process::exit(result);
    }
    let mut settings = Settings::new();
    //settings.set_log_severity(LogSeverity::Disable);
    settings.disable_sandbox();
    let resources_folder = std::path::Path::new("./Resources").canonicalize().unwrap();
    settings.set_resources_dir_path(&resources_folder);
    let locales_folder = std::path::Path::new("./Resources/locales").canonicalize().unwrap();
    settings.set_locales_dir_path(&locales_folder);

    if let Some(context) = {
        #[cfg(target_os = "windows")]
        {
            cef::Context::initialize(&args, &settings, Some(app), None)
        }
        #[cfg(not(target_os = "windows"))]
        {
            cef::Context::initialize(&args, &settings, Some(app))
        }
    } {
        let context = Arc::new(context);
        let mut window_info = WindowInfo::new();
        #[cfg(windows)] {
            window_info.style = WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;
        }
        window_info.window_name = "cefsimple Rust example".into();
        window_info.width = 500;
        window_info.height = 500;
        let browser_settings = BrowserSettings::new();

        let client = Client::new(ClientCallbacksImpl {
            life_span_handler: LifeSpanHandler::new(LifeSpanHandlerImpl {
                context: context.clone(),
            })
        });

        BrowserHost::create_browser(
            &window_info,
            client,
            "https://www.youtube.com",
            &browser_settings,
            None,
            None,
        );

        context.run_message_loop();
    }
}
