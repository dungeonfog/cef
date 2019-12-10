use cef_sys::{cef_browser_settings_t, cef_browser_t, cef_state_t};

use crate::{
    browser_host::BrowserHost,
    color::Color,
    frame::Frame,
    string::{CefString, CefStringList},
};

ref_counted_ptr! {
    /// Structure used to represent a browser window. When used in the browser
    /// process the functions of this structure may be called on any thread unless
    /// otherwise indicated in the comments. When used in the render process the
    /// functions of this structure may only be called on the main thread.
    pub struct Browser(*mut cef_browser_t, true);
}

impl Browser {
    /// Returns the browser host object. This function can only be called in the
    /// browser process.
    pub fn get_host(&self) -> BrowserHost {
        unsafe { BrowserHost::from_ptr_unchecked((self.0.get_host.unwrap())(self.0.as_ptr())) }
    }
    /// Returns true if the browser can navigate backwards.
    pub fn can_go_back(&self) -> bool {
        unsafe { (self.0.can_go_back.unwrap())(self.0.as_ptr()) != 0 }
    }
    /// Navigate backwards.
    pub fn go_back(&self) {
        unsafe {
            (self.0.go_back.unwrap())(self.0.as_ptr());
        }
    }
    /// Returns true if the browser can navigate forwards.
    pub fn can_go_forward(&self) -> bool {
        unsafe { (self.0.can_go_forward.unwrap())(self.0.as_ptr()) != 0 }
    }
    /// Navigate forwards.
    pub fn go_forward(&self) {
        unsafe {
            (self.0.go_forward.unwrap())(self.0.as_ptr());
        }
    }
    /// Returns true if the browser is currently loading.
    pub fn is_loading(&self) -> bool {
        unsafe { (self.0.is_loading.unwrap())(self.0.as_ptr()) != 0 }
    }
    /// Reload the current page, optionally ignoring any cached data.
    pub fn reload(&self, ignore_cache: bool) {
        if ignore_cache {
            unsafe {
                (self.0.reload.unwrap())(self.0.as_ptr());
            }
        } else {
            unsafe {
                (self.0.reload_ignore_cache.unwrap())(self.0.as_ptr());
            }
        }
    }
    /// Stop loading the page.
    pub fn stop_load(&self) {
        unsafe {
            (self.0.stop_load.unwrap())(self.0.as_ptr());
        }
    }
    /// Returns the globally unique identifier for this browser. This value is also
    /// used as the tabId for extension APIs.
    pub fn get_identifier(&self) -> i32 {
        unsafe { (self.0.get_identifier.unwrap())(self.0.as_ptr()) }
    }
    /// Returns true if the window is a popup window.
    pub fn is_popup(&self) -> bool {
        unsafe { (self.0.is_popup.unwrap())(self.0.as_ptr()) != 0 }
    }
    /// Returns true if a document has been loaded in the browser.
    pub fn has_document(&self) -> bool {
        unsafe { (self.0.has_document.unwrap())(self.0.as_ptr()) != 0 }
    }
    /// Returns the main (top-level) frame for the browser window.
    pub fn get_main_frame(&self) -> Frame {
        unsafe { Frame::from_ptr_unchecked((self.0.get_main_frame.unwrap())(self.0.as_ptr())) }
    }
    /// Returns the focused frame for the browser window.
    pub fn get_focused_frame(&self) -> Option<Frame> {
        unsafe { Frame::from_ptr((self.0.get_focused_frame.unwrap())(self.0.as_ptr())) }
    }
    /// Returns the frame with the specified identifier, or None if not found.
    pub fn get_frame_byident(&self, identifier: i64) -> Option<Frame> {
        unsafe {
            Frame::from_ptr((self.0.get_frame_byident.unwrap())(
                self.0.as_ptr(),
                identifier,
            ))
        }
    }
    /// Returns the frame with the specified name, or None if not found.
    pub fn get_frame(&self, name: &str) -> Option<Frame> {
        unsafe {
            Frame::from_ptr((self.0.get_frame.unwrap())(
                self.0.as_ptr(),
                CefString::new(name).as_ptr(),
            ))
        }
    }
    /// Returns the number of frames that currently exist.
    pub fn get_frame_count(&self) -> usize {
        unsafe { (self.0.get_frame_count.unwrap())(self.0.as_ptr()) }
    }
    /// Returns the identifiers of all existing frames.
    pub fn get_frame_identifiers(&self) -> Vec<i64> {
        let mut count = self.get_frame_count();
        let mut result = vec![0; count];
        unsafe {
            (self.0.get_frame_identifiers.unwrap())(
                self.0.as_ptr(),
                &mut count,
                result.as_mut_ptr(),
            );
        }
        result
    }
    /// Returns the names of all existing frames.
    pub fn get_frame_names(&self) -> Vec<String> {
        let mut list = CefStringList::default();
        unsafe {
            (self.0.get_frame_names.unwrap())(self.0.as_ptr(), list.as_mut_ptr());
        }
        Vec::from(list)
    }
}

/// Represents the state of a setting.
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    /// Use the default state for the setting.
    Default = cef_state_t::STATE_DEFAULT as isize,
    /// Enable or allow the setting.
    Enabled = cef_state_t::STATE_ENABLED as isize,
    /// Disable or disallow the setting.
    Disabled = cef_state_t::STATE_DISABLED as isize,
}

impl Into<cef_state_t::Type> for State {
    fn into(self) -> cef_state_t::Type {
        match self {
            Self::Default => cef_state_t::STATE_DEFAULT,
            Self::Enabled => cef_state_t::STATE_ENABLED,
            Self::Disabled => cef_state_t::STATE_DISABLED,
        }
    }
}

impl State {
    pub unsafe fn from_unchecked(state: cef_state_t::Type) -> State {
        std::mem::transmute(state)
    }
}

impl Default for State {
    fn default() -> State {
        State::Default
    }
}

/// Browser initialization settings. Use [BrowserSettings::new] to get the recommended
/// default values. The consequences of using custom values may not be well
/// tested. Many of these and other settings can also configured using command-
/// line switches.
#[derive(Default)]
pub struct BrowserSettings {
    pub windowless_frame_rate: i32,
    pub standard_font_family: String,
    pub fixed_font_family: String,
    pub serif_font_family: String,
    pub sans_serif_font_family: String,
    pub cursive_font_family: String,
    pub fantasy_font_family: String,
    pub default_font_size: i32,
    pub default_fixed_font_size: i32,
    pub minimum_font_size: i32,
    pub minimum_logical_font_size: i32,
    pub default_encoding: String,
    pub remote_fonts: State,
    pub javascript: State,
    pub javascript_close_windows: State,
    pub javascript_access_clipboard: State,
    pub javascript_dom_paste: State,
    pub plugins: State,
    pub universal_access_from_file_urls: State,
    pub file_access_from_file_urls: State,
    pub web_security: State,
    pub image_loading: State,
    pub image_shrink_standalone_to_fit: State,
    pub text_area_resize: State,
    pub tab_to_links: State,
    pub local_storage: State,
    pub databases: State,
    pub application_cache: State,
    pub webgl: State,
    pub background_color: Color,
    pub accept_language_list: String,
}

impl BrowserSettings {
    pub fn new() -> BrowserSettings {
        BrowserSettings::default()
    }

    pub unsafe fn from_raw(raw: &cef_browser_settings_t) -> BrowserSettings {
        BrowserSettings {
            windowless_frame_rate: raw.windowless_frame_rate,
            standard_font_family: CefString::from_ptr_unchecked(&raw.standard_font_family).into(),
            fixed_font_family: CefString::from_ptr_unchecked(&raw.fixed_font_family).into(),
            serif_font_family: CefString::from_ptr_unchecked(&raw.serif_font_family).into(),
            sans_serif_font_family: CefString::from_ptr_unchecked(&raw.sans_serif_font_family).into(),
            cursive_font_family: CefString::from_ptr_unchecked(&raw.cursive_font_family).into(),
            fantasy_font_family: CefString::from_ptr_unchecked(&raw.fantasy_font_family).into(),
            default_font_size: raw.default_font_size,
            default_fixed_font_size: raw.default_fixed_font_size,
            minimum_font_size: raw.minimum_font_size,
            minimum_logical_font_size: raw.minimum_logical_font_size,
            default_encoding: CefString::from_ptr_unchecked(&raw.default_encoding).into(),
            remote_fonts: State::from_unchecked(raw.remote_fonts),
            javascript: State::from_unchecked(raw.javascript),
            javascript_close_windows: State::from_unchecked(raw.javascript_close_windows),
            javascript_access_clipboard: State::from_unchecked(raw.javascript_access_clipboard),
            javascript_dom_paste: State::from_unchecked(raw.javascript_dom_paste),
            plugins: State::from_unchecked(raw.plugins),
            universal_access_from_file_urls: State::from_unchecked(raw.universal_access_from_file_urls),
            file_access_from_file_urls: State::from_unchecked(raw.file_access_from_file_urls),
            web_security: State::from_unchecked(raw.web_security),
            image_loading: State::from_unchecked(raw.image_loading),
            image_shrink_standalone_to_fit: State::from_unchecked(raw.image_shrink_standalone_to_fit),
            text_area_resize: State::from_unchecked(raw.text_area_resize),
            tab_to_links: State::from_unchecked(raw.tab_to_links),
            local_storage: State::from_unchecked(raw.local_storage),
            databases: State::from_unchecked(raw.databases),
            application_cache: State::from_unchecked(raw.application_cache),
            webgl: State::from_unchecked(raw.webgl),
            background_color: Color::wrap(raw.background_color),
            accept_language_list: CefString::from_ptr_unchecked(&raw.accept_language_list).into(),
        }
    }

    pub fn into_raw(&self) -> cef_browser_settings_t {
        cef_browser_settings_t {
            size: std::mem::size_of::<cef_browser_settings_t>(),
            windowless_frame_rate: self.windowless_frame_rate,
            standard_font_family: CefString::new(&self.standard_font_family).into_raw(),
            fixed_font_family: CefString::new(&self.fixed_font_family).into_raw(),
            serif_font_family: CefString::new(&self.serif_font_family).into_raw(),
            sans_serif_font_family: CefString::new(&self.sans_serif_font_family).into_raw(),
            cursive_font_family: CefString::new(&self.cursive_font_family).into_raw(),
            fantasy_font_family: CefString::new(&self.fantasy_font_family).into_raw(),
            default_font_size: self.default_font_size,
            default_fixed_font_size: self.default_fixed_font_size,
            minimum_font_size: self.minimum_font_size,
            minimum_logical_font_size: self.minimum_logical_font_size,
            default_encoding: CefString::new(&self.default_encoding).into_raw(),
            remote_fonts: self.remote_fonts as _,
            javascript: self.javascript as _,
            javascript_close_windows: self.javascript_close_windows as _,
            javascript_access_clipboard: self.javascript_access_clipboard as _,
            javascript_dom_paste: self.javascript_dom_paste as _,
            plugins: self.plugins as _,
            universal_access_from_file_urls: self.universal_access_from_file_urls as _,
            file_access_from_file_urls: self.file_access_from_file_urls as _,
            web_security: self.web_security as _,
            image_loading: self.image_loading as _,
            image_shrink_standalone_to_fit: self.image_shrink_standalone_to_fit as _,
            text_area_resize: self.text_area_resize as _,
            tab_to_links: self.tab_to_links as _,
            local_storage: self.local_storage as _,
            databases: self.databases as _,
            application_cache: self.application_cache as _,
            webgl: self.webgl as _,
            background_color: self.background_color.get(),
            accept_language_list: CefString::new(&self.accept_language_list).into_raw(),
        }
    }
}
