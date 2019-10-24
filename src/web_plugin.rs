use cef_sys::{cef_string_userfree_utf16_free, cef_web_plugin_info_t};
use std::path::PathBuf;
use crate::string::CefString;

/// Information about a specific web plugin.
pub struct WebPluginInfo {
    /// Returns the plugin name (i.e. Flash).
    pub name: String,
    /// Returns the plugin file path (DLL/bundle/library).
    pub path: PathBuf,
    /// Returns the version of the plugin (may be OS-specific).
    pub version: String,
    /// Returns a description of the plugin from the version information.
    pub description: String,
}

impl WebPluginInfo {
    pub(crate) unsafe fn new(info: *mut cef_web_plugin_info_t) -> Self {
        let name = (*info).get_name.unwrap()(info);
        let path = (*info).get_path.unwrap()(info);
        let version = (*info).get_version.unwrap()(info);
        let description = (*info).get_description.unwrap()(info);

        let result = Self {
            name: CefString::from_ptr_unchecked(name).into(),
            path: PathBuf::from(String::from(CefString::from_ptr_unchecked(path))),
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
