use cef_sys::{cef_web_plugin_info_t};

/// Information about a specific web plugin.
pub struct WebPluginInfo(*mut cef_web_plugin_info_t);

impl WebPluginInfo {
    /// Returns the plugin name (i.e. Flash).
    pub fn get_name(&self) -> String {
        unimplemented!()
    }
    /// Returns the plugin file path (DLL/bundle/library).
    pub fn get_path(&self) -> String {
        unimplemented!()
    }
    /// Returns the version of the plugin (may be OS-specific).
    pub fn get_version(&self) -> String {
        unimplemented!()
    }
    /// Returns a description of the plugin from the version information.
    pub fn get_description(&self) -> String {
        unimplemented!()
    }
}
