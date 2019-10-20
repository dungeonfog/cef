use cef_sys::{cef_extension_t, cef_string_userfree_utf16_free};
use std::collections::HashMap;

use crate::{
    request_context::RequestContext,
    string::CefString,
    values::{DictionaryValue, StoredValue},
};

ref_counted_ptr! {
    /// Object representing an extension. Methods may be called on any thread unless
    /// otherwise indicated.
    pub struct Extension(*mut cef_extension_t);
}

impl Extension {
    /// Returns the unique extension identifier. This is calculated based on the
    /// extension public key, if available, or on the extension path. See
    /// https://developer.chrome.com/extensions/manifest/key for details.
    pub fn get_identifier(&self) -> String {
        self.0
            .get_identifier
            .and_then(|get_identifier| unsafe { get_identifier(self.as_ptr()).as_mut() })
            .map(|cef_string| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(cef_string));
                cef_string_userfree_utf16_free(cef_string);
                s
            })
            .unwrap_or_default()
    }

    /// Returns the absolute path to the extension directory on disk. This value
    /// will be prefixed with PK_DIR_RESOURCES if a relative path was passed to
    /// [RequestContext::load_extension].
    pub fn get_path(&self) -> String {
        self.0
            .get_path
            .and_then(|get_path| unsafe { get_path(self.as_ptr()).as_mut() })
            .map(|cef_string| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(cef_string));
                cef_string_userfree_utf16_free(cef_string);
                s
            })
            .unwrap_or_default()
    }
    // Returns the extension manifest contents as a dictionary object.
    // See https://developer.chrome.com/extensions/manifest for details.
    pub fn get_manifest(&self) -> HashMap<String, StoredValue> {
        self.0
            .get_manifest
            .and_then(|get_manifest| {
                unsafe { DictionaryValue::from_ptr(get_manifest(self.0.as_ptr())) }
                    .map(DictionaryValue::into)
            })
            .unwrap_or_default()
    }
    /// Returns the handler for this extension. Will return None for internal
    /// extensions or if no handler was passed to [RequestContext::load_extension].
    pub fn get_handler(&self) -> Option<Box<dyn ExtensionHandler>> {
        unimplemented!()
    }
    /// Returns the request context that loaded this extension. Will return None
    /// for internal extensions or if the extension has been unloaded. See the
    /// [RequestContext::load_extension] documentation for more information
    /// about loader contexts. Must be called on the browser process UI thread.
    pub fn get_loader_context(&self) -> Option<RequestContext> {
        self.0
            .get_loader_context
            .and_then(|get_loader_context| unsafe {
                RequestContext::from_ptr(get_loader_context(self.0.as_ptr()))
            })
    }
    /// Returns true if this extension is currently loaded. Must be called on
    /// the browser process UI thread.
    pub fn is_loaded(&self) -> bool {
        self.0
            .is_loaded
            .map(|is_loaded| unsafe { is_loaded(self.0.as_ptr()) != 0 })
            .unwrap_or_default()
    }
    /// Unload this extension if it is not an internal extension and is currently
    /// loaded. Will result in a call to
    /// [ExtensionHandler::on_extension_unloaded] on success.
    pub fn unload(&mut self) {
        if let Some(unload) = self.0.unload {
            unsafe {
                unload(self.0.as_ptr());
            }
        }
    }
}

/// Implement this trait to handle events related to browser extensions. The
/// functions of this trait will be called on the UI thread. See
/// [RequestContext::load_extension] for information about extension loading.
pub trait ExtensionHandler {
    // TODO
}
