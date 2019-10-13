use cef_sys::{cef_string_utf8_to_utf16, cef_window_info_t};

/// Structure representing window information.
pub struct WindowInfo(cef_window_info_t);

impl WindowInfo {
    pub fn new() -> Self {
        WindowInfo(unsafe { std::mem::zeroed() })
    }
    pub(crate) fn get(&self) -> *const cef_window_info_t {
        &self.0
    }

    pub fn set_style(&mut self, style: u32) {
        self.0.style = style;
    }
    pub fn set_ex_style(&mut self, style: u32) {
        self.0.ex_style = style;
    }
    /// The initial title of the window, to be set when the window is created.
    /// On Linux, some layout managers (e.g., Compiz) can look at the window title
    /// in order to decide where to place the window when it is
    /// created. When this attribute is not empty, the window title will
    /// be set before the window is mapped to the dispay. Otherwise the
    /// title will be initially empty.
    pub fn set_window_name(&mut self, name: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                name.as_ptr() as *const std::os::raw::c_char,
                name.len(),
                &mut self.0.window_name,
            );
        }
    }
    pub fn set_x(&mut self, x: i32) {
        self.0.x = x;
    }
    pub fn set_y(&mut self, y: i32) {
        self.0.y = y;
    }
    pub fn set_width(&mut self, width: i32) {
        self.0.width = width;
    }
    pub fn set_height(&mut self, height: i32) {
        self.0.height = height;
    }
    /// Set pointer for the parent window.
    pub fn set_parent_window(&mut self, parent: std::os::windows::raw::HANDLE) {
        self.0.parent_window = parent as cef_sys::HWND;
    }
    pub fn set_menu(&mut self, menu: std::os::windows::raw::HANDLE) {
        self.0.menu = menu as cef_sys::HMENU;
    }
    /// Call to create the browser using windowless (off-screen)
    /// rendering. No window will be created for the browser and all rendering will
    /// occur via the CefRenderHandler interface. The [WindowInfo::set_parent_window] parameter will be
    /// used to identify monitor info and to act as the parent window for dialogs,
    /// context menus, etc. If [WindowInfo::set_parent_window] is not called then the main screen
    /// monitor will be used and some functionality that requires a parent window
    /// may not function correctly. In order to create windowless browsers call the
    /// [Settings::enable_windowless_rendering] function.
    /// Transparent painting is enabled by default but can be disabled by calling
    /// [BrowserSettings::set_background_color] with an opaque value.
    pub fn enable_windowless_rendering(&mut self) {
        self.0.windowless_rendering_enabled = 1;
    }
    /// Call to enable shared textures for windowless rendering. Only
    /// valid if [WindowInfo::enable_windowless_rendering} above is also called. Currently
    /// only supported on Windows (D3D11).
    pub fn enable_shared_texture(&mut self) {
        self.0.shared_texture_enabled = 1;
    }
    /// Call to enable the ability to issue begin_frame requests from the
    /// client application by calling [BrowserHost::send_external_begin_frame].
    pub fn enable_external_begin_frame(&mut self) {
        self.0.external_begin_frame_enabled = 1;
    }
    /// Set the handle for the new browser window. Only used with windowed rendering.
    pub fn set_window(&mut self, window: std::os::windows::raw::HANDLE) {
        self.0.window = window as cef_sys::HWND;
    }
}

impl Drop for WindowInfo {
    fn drop(&mut self) {
        if let Some(dtor) = self.0.window_name.dtor {
            unsafe {
                dtor(self.0.window_name.str);
            }
        }
    }
}
