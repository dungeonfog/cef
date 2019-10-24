use crate::string::CefString;
use cef_sys::{cef_window_info_t};
use std::ptr;
use winapi::shared::{
    windef::{HWND, HMENU},
    minwindef::DWORD,
};

/// Structure representing window information.
pub struct WindowInfo {
    pub window_name: String,
    pub style: DWORD,
    pub ex_style: DWORD,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub parent_window: HWND,
    pub window: HWND,
    pub menu: HMENU,
    pub windowless_rendering_enabled: bool,
    pub shared_texture_enabled: bool,
    pub external_begin_frame_enabled: bool,
}

impl WindowInfo {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn into_raw(&self) -> cef_window_info_t {
        self.into()
    }
}

impl<'a> From<&'a cef_window_info_t> for WindowInfo {
    fn from(info: &'a cef_window_info_t) -> WindowInfo {
        WindowInfo {
            window_name: unsafe{ CefString::from_ptr_unchecked(&info.window_name).into() },
            style: info.style,
            ex_style: info.ex_style,
            x: info.x,
            y: info.y,
            width: info.width,
            height: info.height,
            parent_window: info.parent_window as _,
            window: info.window as _,
            menu: info.menu as _,
            windowless_rendering_enabled: info.windowless_rendering_enabled != 0,
            shared_texture_enabled: info.shared_texture_enabled != 0,
            external_begin_frame_enabled: info.external_begin_frame_enabled != 0,
        }
    }
}

impl<'a> From<&'a WindowInfo> for cef_window_info_t {
    fn from(info: &'a WindowInfo) -> cef_window_info_t {
        cef_window_info_t {
            ex_style: info.ex_style,
            window_name: CefString::new(&info.window_name).into_raw(),
            style: info.style,
            x: info.x,
            y: info.y,
            width: info.width,
            height: info.height,
            parent_window: info.parent_window as _,
            menu: info.menu as _,
            window: info.window as _,
            windowless_rendering_enabled: info.windowless_rendering_enabled as _,
            shared_texture_enabled: info.shared_texture_enabled as _,
            external_begin_frame_enabled: info.external_begin_frame_enabled as _,
        }
    }
}

impl Default for WindowInfo {
    fn default() -> Self {
        WindowInfo {
            window_name: String::new(),
            style: 0,
            ex_style: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            parent_window: ptr::null_mut(),
            window: ptr::null_mut(),
            menu: ptr::null_mut(),
            windowless_rendering_enabled: false,
            shared_texture_enabled: false,
            external_begin_frame_enabled: false,
        }
    }
}
