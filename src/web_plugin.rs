use cef_sys::{cef_string_userfree_utf16_free, cef_web_plugin_info_t};

use crate::string::CefString;

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

impl WebPluginInfo {
    pub(crate) unsafe fn new(info: *mut cef_web_plugin_info_t) -> Self {
        let name = (*info).get_name.unwrap()(info);
        let path = (*info).get_path.unwrap()(info);
        let version = (*info).get_version.unwrap()(info);
        let description = (*info).get_description.unwrap()(info);

        let result = Self {
            name: CefString::from_ptr_unchecked(name).into(),
            path: CefString::from_ptr_unchecked(path).into(),
            version: CefString::from_ptr_unchecked(version).into(),
            description: CefString::from_ptr_unchecked(description).into(),
        };
        cef_string_userfree_utf16_free(name);
        cef_string_userfree_utf16_free(path);
        cef_string_userfree_utf16_free(version);
        cef_string_userfree_utf16_free(description);

        result
    }
}
