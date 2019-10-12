use cef_sys::{cef_browser_settings_t, cef_browser_t, cef_state_t, cef_string_utf8_to_utf16};

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
    pub struct Browser(*mut cef_browser_t);
}

unsafe impl Send for Browser {}
unsafe impl Sync for Browser {}

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
    pub fn go_back(&mut self) {
        unsafe {
            (self.0.go_back.unwrap())(self.0.as_ptr());
        }
    }
    /// Returns true if the browser can navigate forwards.
    pub fn can_go_forward(&self) -> bool {
        unsafe { (self.0.can_go_forward.unwrap())(self.0.as_ptr()) != 0 }
    }
    /// Navigate forwards.
    pub fn go_forward(&mut self) {
        unsafe {
            (self.0.go_forward.unwrap())(self.0.as_ptr());
        }
    }
    /// Returns true if the browser is currently loading.
    pub fn is_loading(&self) -> bool {
        unsafe { (self.0.is_loading.unwrap())(self.0.as_ptr()) != 0 }
    }
    /// Reload the current page, optionally ignoring any cached data.
    pub fn reload(&mut self, ignore_cache: bool) {
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
    pub fn stop_load(&mut self) {
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
                CefString::new(name).as_ref(),
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
        let list = CefStringList::default();
        unsafe {
            (self.0.get_frame_names.unwrap())(self.0.as_ptr(), list.get());
        }
        unsafe { list.into_vec() }
    }
}

/// Represents the state of a setting.
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    /// Use the default state for the setting.
    Default = cef_state_t::STATE_DEFAULT,
    /// Enable or allow the setting.
    Enabled = cef_state_t::STATE_ENABLED,
    /// Disable or disallow the setting.
    Disabled = cef_state_t::STATE_DISABLED,
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

/// Browser initialization settings. Use [BrowserSettings::new] to get the recommended
/// default values. The consequences of using custom values may not be well
/// tested. Many of these and other settings can also configured using command-
/// line switches.
pub struct BrowserSettings(cef_browser_settings_t);

impl BrowserSettings {
    pub fn new() -> Self {
        Self(cef_browser_settings_t {
            size: std::mem::size_of::<cef_browser_settings_t>(),
            ..unsafe { std::mem::zeroed() }
        })
    }
    pub(crate) fn get(&self) -> *const cef_browser_settings_t {
        &self.0
    }

    /// The maximum rate in frames per second (fps) that [RenderHandler::on_paint]
    /// will be called for a windowless browser. The actual fps may be lower if
    /// the browser cannot generate frames at the requested rate. The minimum
    /// value is 1 and the maximum value is 60 (default 30). This value can also be
    /// changed dynamically via [BrowserHost::set_windowless_frame_rate].
    pub fn set_windowless_frame_rate(&mut self, frame_rate: i32) {
        self.0.windowless_frame_rate = frame_rate;
    }
    pub fn set_standard_font_family(&mut self, family: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                family.as_ptr() as *const std::os::raw::c_char,
                family.len(),
                &mut self.0.standard_font_family,
            );
        }
    }
    pub fn set_fixed_font_family(&mut self, family: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                family.as_ptr() as *const std::os::raw::c_char,
                family.len(),
                &mut self.0.fixed_font_family,
            );
        }
    }
    pub fn set_serif_font_family(&mut self, family: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                family.as_ptr() as *const std::os::raw::c_char,
                family.len(),
                &mut self.0.serif_font_family,
            );
        }
    }
    pub fn set_sans_serif_font_family(&mut self, family: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                family.as_ptr() as *const std::os::raw::c_char,
                family.len(),
                &mut self.0.sans_serif_font_family,
            );
        }
    }
    pub fn set_cursive_font_family(&mut self, family: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                family.as_ptr() as *const std::os::raw::c_char,
                family.len(),
                &mut self.0.cursive_font_family,
            );
        }
    }
    pub fn set_fantasy_font_family(&mut self, family: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                family.as_ptr() as *const std::os::raw::c_char,
                family.len(),
                &mut self.0.fantasy_font_family,
            );
        }
    }
    pub fn set_default_font_size(&mut self, size: i32) {
        self.0.default_font_size = size;
    }
    pub fn set_default_fixed_font_size(&mut self, size: i32) {
        self.0.default_fixed_font_size = size;
    }
    pub fn set_minimum_font_size(&mut self, size: i32) {
        self.0.minimum_font_size = size;
    }
    pub fn set_minimum_logical_font_size(&mut self, size: i32) {
        self.0.minimum_logical_font_size = size;
    }
    /// Set the default encoding for Web content. If empty "ISO-8859-1" will be used. Also
    /// configurable using the "default-encoding" command-line switch.
    pub fn set_default_encoding(&mut self, encoding: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                encoding.as_ptr() as *const std::os::raw::c_char,
                encoding.len(),
                &mut self.0.default_encoding,
            );
        }
    }
    /// Controls the loading of fonts from remote sources. Also configurable using
    /// the "disable-remote-fonts" command-line switch.
    pub fn set_remote_fonts(&mut self, state: State) {
        self.0.remote_fonts = state.into();
    }
    /// Controls whether JavaScript can be executed. Also configurable using the
    /// "disable-javascript" command-line switch.
    pub fn set_javascript(&mut self, state: State) {
        self.0.javascript = state.into();
    }
    /// Controls whether JavaScript can be used to close windows that were not
    /// opened via JavaScript. JavaScript can still be used to close windows that
    /// were opened via JavaScript or that have no back/forward history. Also
    /// configurable using the "disable-javascript-close-windows" command-line
    /// switch.
    pub fn set_javascript_close_windows(&mut self, state: State) {
        self.0.javascript_close_windows = state.into();
    }
    /// Controls whether JavaScript can access the clipboard. Also configurable
    /// using the "disable-javascript-access-clipboard" command-line switch.
    pub fn set_javascript_access_clipboard(&mut self, state: State) {
        self.0.javascript_access_clipboard = state.into();
    }
    /// Controls whether DOM pasting is supported in the editor via
    /// execCommand("paste"). [BrowserSettings::set_javascript_access_clipboard] must also
    /// be called. Also configurable using the "disable-javascript-dom-paste"
    /// command-line switch.
    pub fn set_javascript_dom_paste(&mut self, state: State) {
        self.0.javascript_dom_paste = state.into();
    }
    /// Controls whether any plugins will be loaded. Also configurable using the
    /// "disable-plugins" command-line switch.
    pub fn set_plugins(&mut self, state: State) {
        self.0.plugins = state.into();
    }
    /// Controls whether file URLs will have access to all URLs. Also configurable
    /// using the "allow-universal-access-from-files" command-line switch.
    pub fn set_universal_access_from_file_urls(&mut self, state: State) {
        self.0.universal_access_from_file_urls = state.into();
    }
    /// Controls whether file URLs will have access to other file URLs. Also
    /// configurable using the "allow-access-from-files" command-line switch.
    pub fn set_file_access_from_file_urls(&mut self, state: State) {
        self.0.file_access_from_file_urls = state.into();
    }
    /// Controls whether web security restrictions (same-origin policy) will be
    /// enforced. Disabling this setting is not recommend as it will allow risky
    /// security behavior such as cross-site scripting (XSS). Also configurable
    /// using the "disable-web-security" command-line switch.
    pub fn set_web_security(&mut self, state: State) {
        self.0.web_security = state.into();
    }
    /// Controls whether image URLs will be loaded from the network. A cached image
    /// will still be rendered if requested. Also configurable using the
    /// "disable-image-loading" command-line switch.
    pub fn set_image_loading(&mut self, state: State) {
        self.0.image_loading = state.into();
    }
    /// Controls whether standalone images will be shrunk to fit the page. Also
    /// configurable using the "image-shrink-standalone-to-fit" command-line
    /// switch.
    pub fn set_image_shrink_standalone_to_fit(&mut self, state: State) {
        self.0.image_shrink_standalone_to_fit = state.into();
    }
    /// Controls whether text areas can be resized. Also configurable using the
    /// "disable-text-area-resize" command-line switch.
    pub fn set_text_area_resize(&mut self, state: State) {
        self.0.text_area_resize = state.into();
    }
    /// Controls whether the tab key can advance focus to links. Also configurable
    /// using the "disable-tab-to-links" command-line switch.
    pub fn set_tab_to_links(&mut self, state: State) {
        self.0.tab_to_links = state.into();
    }
    /// Controls whether local storage can be used. Also configurable using the
    /// "disable-local-storage" command-line switch.
    pub fn set_local_storage(&mut self, state: State) {
        self.0.local_storage = state.into();
    }
    /// Controls whether databases can be used. Also configurable using the
    /// "disable-databases" command-line switch.
    pub fn set_databases(&mut self, state: State) {
        self.0.databases = state.into();
    }
    /// Controls whether the application cache can be used. Also configurable using
    /// the "disable-application-cache" command-line switch.
    pub fn set_application_cache(&mut self, state: State) {
        self.0.application_cache = state.into();
    }
    /// Controls whether WebGL can be used. Note that WebGL requires hardware
    /// support and may not work on all systems even when enabled. Also
    /// configurable using the "disable-webgl" command-line switch.
    pub fn set_webgl(&mut self, state: State) {
        self.0.webgl = state.into();
    }
    /// Background color used for the browser before a document is loaded and when
    /// no document color is specified. The alpha component must be either fully
    /// opaque (1.0) or fully transparent (0.0). If the alpha component is fully
    /// opaque then the RGB components will be used as the background color. If the
    /// alpha component is fully transparent for a windowed browser then the value passed to
    /// [Settings.set_background_color] will be used. If the alpha component is
    /// fully transparent for a windowless (off-screen) browser then transparent
    /// painting will be enabled.
    pub fn set_background_color(&mut self, color: Color) {
        self.0.background_color = color.get();
    }
    /// Pass a comma delimited ordered list of language codes without any whitespace that
    /// will be used in the "Accept-Language" HTTP header. May be set globally
    /// using the [BrowserSettings.set_accept_language_list] function. If both values are
    /// not called then "en-US,en" will be used.
    pub fn set_accept_language_list(&mut self, list: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                list.as_ptr() as *const std::os::raw::c_char,
                list.len(),
                &mut self.0.accept_language_list,
            );
        }
    }
}

impl Drop for BrowserSettings {
    fn drop(&mut self) {
        if let Some(dtor) = self.0.standard_font_family.dtor {
            unsafe {
                dtor(self.0.standard_font_family.str);
            }
        }
        if let Some(dtor) = self.0.fixed_font_family.dtor {
            unsafe {
                dtor(self.0.fixed_font_family.str);
            }
        }
        if let Some(dtor) = self.0.serif_font_family.dtor {
            unsafe {
                dtor(self.0.serif_font_family.str);
            }
        }
        if let Some(dtor) = self.0.sans_serif_font_family.dtor {
            unsafe {
                dtor(self.0.sans_serif_font_family.str);
            }
        }
        if let Some(dtor) = self.0.cursive_font_family.dtor {
            unsafe {
                dtor(self.0.cursive_font_family.str);
            }
        }
        if let Some(dtor) = self.0.fantasy_font_family.dtor {
            unsafe {
                dtor(self.0.fantasy_font_family.str);
            }
        }
        if let Some(dtor) = self.0.default_encoding.dtor {
            unsafe {
                dtor(self.0.default_encoding.str);
            }
        }
        if let Some(dtor) = self.0.accept_language_list.dtor {
            unsafe {
                dtor(self.0.accept_language_list.str);
            }
        }
    }
}
