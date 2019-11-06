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
    command_line::CommandLine,
};
use std::sync::Arc;
use x11_dl::xlib::{Xlib, Display, XErrorEvent};

pub struct AppCallbacksImpl {}

impl AppCallbacks for AppCallbacksImpl {
    fn on_before_command_line_processing (&self, process_type: Option<&str>, command_line: CommandLine) {
        if process_type == None {
            command_line.append_switch("disable-gpu");
            command_line.append_switch("disable-gpu-compositing");
        }
    }
}

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

extern "C" fn x_error_handler(_: *mut Display, error: *mut XErrorEvent) -> std::os::raw::c_int {
    unsafe {
        println!("X Error type {} resourceid {} serial {} error_code {} request_code {} minor_code {}",
            (*error).type_, (*error).resourceid, (*error).serial, (*error).error_code, (*error).request_code, (*error).minor_code);
    }
    0
}
extern "C" fn x_io_error_handler(_: *mut Display) -> std::os::raw::c_int {
    0
}

fn main() {
    #[cfg(windows)]
    cef::enable_highdpi_support();
    let args = MainArgs::new();
    #[cfg(target_os = "windows")]
    let result = cef::execute_process(&args, None, None);
    #[cfg(not(target_os = "windows"))]
    let result = cef::execute_process(&args, None);
    if result >= 0 {
        std::process::exit(result);
    }
    let xlib = Xlib::open().unwrap();

    unsafe {
        (xlib.XSetErrorHandler)(Some(x_error_handler));
        (xlib.XSetIOErrorHandler)(Some(x_io_error_handler));
    }

    let mut settings = Settings::new();
    settings.set_log_severity(LogSeverity::Info);
    settings.disable_sandbox();
    let resources_folder = std::path::Path::new("./Resources").canonicalize().unwrap();
    settings.set_resources_dir_path(&resources_folder);
    let locales_folder = std::path::Path::new("./Resources/locales").canonicalize().unwrap();
    settings.set_locales_dir_path(&locales_folder);

    let app = App::new(AppCallbacksImpl {});

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
