#[cfg(windows)]
use winapi::um::{libloaderapi::GetModuleHandleA, winuser::{
    WS_OVERLAPPEDWINDOW,
    WS_CLIPCHILDREN,
    WS_CLIPSIBLINGS,
    WS_VISIBLE,
    CW_USEDEFAULT,
}};

pub struct AppCallbacks();

impl cef::AppCallbacks for AppCallbacks {}

fn main() {
    let mut app = cef::App::new(AppCallbacks());

}
