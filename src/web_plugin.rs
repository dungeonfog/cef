use cef_sys::{cef_web_plugin_info_t, cef_string_userfree_utf16_free};

use crate::{
    string::CefString,
};

/// Information about a specific web plugin.
pub struct WebPluginInfo {
    name: String,
    path: String,
    version: String,
    description: String,
}

impl WebPluginInfo {
    /// Returns the plugin name (i.e. Flash).
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// Returns the plugin file path (DLL/bundle/library).
    pub fn get_path(&self) -> &str {
        &self.path
    }
    /// Returns the version of the plugin (may be OS-specific).
    pub fn get_version(&self) -> &str {
        &self.version
    }
    /// Returns a description of the plugin from the version information.
    pub fn get_description(&self) -> &str {
        &self.description
    }
}

impl From<*mut cef_web_plugin_info_t> for WebPluginInfo {
    fn from(info: *mut cef_web_plugin_info_t) -> Self {
        let mut name = unsafe { (*info).get_name.unwrap()(info) };
        let mut path = unsafe { (*info).get_path.unwrap()(info) };
        let mut version = unsafe { (*info).get_version.unwrap()(info) };
        let mut description = unsafe { (*info).get_description.unwrap()(info) };

        let result = unsafe { Self {
            name: CefString::copy_raw_to_string(name).unwrap(),
            path: CefString::copy_raw_to_string(path).unwrap(),
            version: CefString::copy_raw_to_string(version).unwrap(),
            description: CefString::copy_raw_to_string(description).unwrap(),
        }};
        unsafe {
            cef_string_userfree_utf16_free(name);
            cef_string_userfree_utf16_free(path);
            cef_string_userfree_utf16_free(version);
            cef_string_userfree_utf16_free(description);
        }
        result
    }
}
