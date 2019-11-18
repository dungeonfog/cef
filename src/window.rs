use crate::string::CefString;
use cef_sys::{cef_window_info_t, cef_window_handle_t};
use std::ptr;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct RawWindow(RawWindowHandle);

impl RawWindow {
    pub unsafe fn from_window<W: HasRawWindowHandle>(window: &W) -> RawWindow {
        RawWindow(window.raw_window_handle())
    }

    pub unsafe fn from_cef_handle(window: cef_window_handle_t) -> Option<RawWindow> {
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::windows::WindowsHandle;
            if window != ptr::null_mut() {
                Some(RawWindow(RawWindowHandle::Windows(WindowsHandle { hwnd: window as *mut _ as _, ..WindowsHandle::empty() })))
            } else {
                None
            }
        }
        #[cfg(target_os = "linux")]
        {
            use raw_window_handle::unix::XlibHandle;
            if window == 0 {
                None
            } else {
                Some(RawWindow(RawWindowHandle::Xlib(XlibHandle { window, ..XlibHandle::empty() })))
            }
        }
    }
}

unsafe impl HasRawWindowHandle for RawWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}

/// Structure representing window information.
pub struct WindowInfo {
    pub window_name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub parent_window: Option<RawWindow>,
    pub window: Option<RawWindow>,
    pub windowless_rendering_enabled: bool,
    pub shared_texture_enabled: bool,
    pub external_begin_frame_enabled: bool,
    pub platform_specific: PlatformSpecificWindowInfo,
}

#[cfg(target_os = "windows")]
use windows::PlatformSpecificWindowInfo;
#[cfg(target_os = "windows")]
pub mod windows {
    use std::ptr;
    use winapi::shared::{
        windef::HMENU,
        minwindef::DWORD,
    };
    pub struct PlatformSpecificWindowInfo {
        pub style: DWORD,
        pub ex_style: DWORD,
        pub menu: HMENU,
    }

    impl Default for PlatformSpecificWindowInfo {
        fn default() -> Self {
            PlatformSpecificWindowInfo {
                style: 0,
                ex_style: 0,
                menu: ptr::null_mut(),
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
use not_windows::PlatformSpecificWindowInfo;
#[cfg(not(target_os = "windows"))]
pub mod not_windows {
    #[derive(Default)]
    pub struct PlatformSpecificWindowInfo;
}

impl WindowInfo {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn into_raw(&self) -> cef_window_info_t {
        self.into()
    }

    pub unsafe fn from_raw(info: &cef_window_info_t) -> WindowInfo {
        #[cfg(target_os = "windows")]
        {
            WindowInfo {
                window_name: CefString::from_ptr_unchecked(&info.window_name).into(),
                x: info.x,
                y: info.y,
                width: info.width,
                height: info.height,
                parent_window: RawWindow::from_cef_handle(info.parent_window),
                window: RawWindow::from_cef_handle(info.window),
                windowless_rendering_enabled: info.windowless_rendering_enabled != 0,
                shared_texture_enabled: info.shared_texture_enabled != 0,
                external_begin_frame_enabled: info.external_begin_frame_enabled != 0,
                platform_specific: PlatformSpecificWindowInfo {
                    menu: info.menu as _,
                    style: info.style,
                    ex_style: info.ex_style,
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            use raw_window_handle::unix::XlibHandle;
            WindowInfo {
                window_name: CefString::from_ptr_unchecked(&info.window_name).into(),
                x: info.x as _,
                y: info.y as _,
                width: info.width as _,
                height: info.height as _,
                parent_window: RawWindow::from_cef_handle(info.parent_window),
                window: RawWindow::from_cef_handle(info.window),
                windowless_rendering_enabled: info.windowless_rendering_enabled != 0,
                shared_texture_enabled: info.shared_texture_enabled != 0,
                external_begin_frame_enabled: info.external_begin_frame_enabled != 0,
                platform_specific: PlatformSpecificWindowInfo
            }
        }
    }
}

#[cfg(target_os = "windows")]
impl<'a> From<&'a WindowInfo> for cef_window_info_t {
    fn from(info: &'a WindowInfo) -> cef_window_info_t {
        let unwrap_hwnd = |r: &RawWindow| match r.0 {
            RawWindowHandle::Windows(windows_handle) => windows_handle.hwnd,
            _ => panic!(),
        };
        cef_window_info_t {
            ex_style: info.platform_specific.ex_style,
            window_name: CefString::new(&info.window_name).into_raw(),
            style: info.platform_specific.style,
            x: info.x as _,
            y: info.y as _,
            width: info.width as _,
            height: info.height as _,
            parent_window: info.parent_window.as_ref().map(unwrap_hwnd).map(|h| h as _).unwrap_or(ptr::null_mut()),
            window: info.window.as_ref().map(unwrap_hwnd).map(|h| h as _).unwrap_or(ptr::null_mut()),
            menu: info.platform_specific.menu as _,
            windowless_rendering_enabled: info.windowless_rendering_enabled as _,
            shared_texture_enabled: info.shared_texture_enabled as _,
            external_begin_frame_enabled: info.external_begin_frame_enabled as _,
        }
    }
}

#[cfg(target_os = "linux")]
impl<'a> From<&'a WindowInfo> for cef_window_info_t {
    fn from(info: &'a WindowInfo) -> cef_window_info_t {
        let unwrap_window = |r: &RawWindow| match r {
            RawWindow(RawWindowHandle::Xlib(xlib_handle)) => xlib_handle.window,
            _ => panic!(),
        };
        cef_window_info_t {
            window_name: CefString::new(&info.window_name).into_raw(),
            x: info.x as _,
            y: info.y as _,
            width: info.width as _,
            height: info.height as _,
            parent_window: info.parent_window.as_ref().map(unwrap_window).unwrap_or(0),
            window: info.window.as_ref().map(unwrap_window).unwrap_or(0),
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
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            parent_window: None,
            window: None,
            windowless_rendering_enabled: false,
            shared_texture_enabled: false,
            external_begin_frame_enabled: false,
            platform_specific: PlatformSpecificWindowInfo::default(),
        }
    }
}
