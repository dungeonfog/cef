use crate::{
    string::{CefString, CefStringList},
};
use cef_sys::{
    _cef_context_menu_params_t,
};

use super::{ContextMenuTypeFlags, ContextMenuMediaStateFlags, ContextMenuEditStateFlags};

ref_counted_ptr!{
    /// Provides information about the context menu state. The ethods of this
    /// structure can only be accessed on browser process the UI thread.
    pub struct ContextMenuParams(*mut _cef_context_menu_params_t);
}

impl ContextMenuParams {
    /// Returns the X coordinate of the mouse where the context menu was invoked.
    /// Coords are relative to the associated RenderView's origin.
    pub fn get_xcoord(&self) -> i32 {
        unsafe {
            self.0.get_xcoord.unwrap()(self.as_ptr())
        }
    }
    /// Returns the Y coordinate of the mouse where the context menu was invoked.
    /// Coords are relative to the associated RenderView's origin.
    pub fn get_ycoord(&self) -> i32 {
        unsafe {
            self.0.get_ycoord.unwrap()(self.as_ptr())
        }
    }
    /// Returns flags representing the type of node that the context menu was
    /// invoked on.
    pub fn get_type_flags(&self) -> ContextMenuTypeFlags {
        unsafe {
            ContextMenuTypeFlags::from_bits_truncate(self.0.get_type_flags.unwrap()(self.as_ptr()).0)
        }
    }
    /// Returns the URL of the link, if any, that encloses the node that the
    /// context menu was invoked on.
    pub fn get_link_url(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_link_url.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the link URL, if any, to be used ONLY for "copy link address". We
    /// don't validate this field in the frontend process.
    pub fn get_unfiltered_link_url(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_unfiltered_link_url.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the source URL, if any, for the element that the context menu was
    /// invoked on. Example of elements with source URLs are img, audio, and video.
    pub fn get_source_url(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_source_url.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns `true` if the context menu was invoked on an image which has non-
    /// NULL contents.
    pub fn has_image_contents(&self) -> bool {
        unsafe {
            self.0.has_image_contents.unwrap()(self.as_ptr()) != 0
        }
    }
    /// Returns the title text or the alt text if the context menu was invoked on
    /// an image.
    pub fn get_title_text(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_title_text.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the URL of the top level page that the context menu was invoked on.
    pub fn get_page_url(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_page_url.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the URL of the subframe that the context menu was invoked on.
    pub fn get_frame_url(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_frame_url.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the character encoding of the subframe that the context menu was
    /// invoked on.
    pub fn get_frame_charset(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_frame_charset.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the type of context node that the context menu was invoked on.
    pub fn get_media_type(&self) -> ContextMenuTypeFlags {
        unsafe {
            ContextMenuTypeFlags::from_bits_truncate(self.0.get_media_type.unwrap()(self.as_ptr()))
        }
    }
    /// Returns flags representing the actions supported by the media element, if
    /// any, that the context menu was invoked on.
    pub fn get_media_state_flags(&self) -> ContextMenuMediaStateFlags {
        unsafe {
            ContextMenuMediaStateFlags::from_bits_truncate(self.0.get_media_state_flags.unwrap()(self.as_ptr()).0)
        }
    }
    /// Returns the text of the selection, if any, that the context menu was
    /// invoked on.
    pub fn get_selection_text(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_selection_text.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the text of the misspelled word, if any, that the context menu was
    /// invoked on.
    pub fn get_misspelled_word(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_misspelled_word.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns `true` if suggestions exist, `false` otherwise. Fills in
    /// `suggestions` from the spell check service for the misspelled word if there
    /// is one.
    pub fn get_dictionary_suggestions(&self) -> Option<Vec<String>> {
        unsafe {
            let mut string_list = CefStringList::new();
            let has_suggestions = self.0.get_dictionary_suggestions.unwrap()(self.as_ptr(), string_list.as_mut_ptr());
            if has_suggestions != 0 {
                Some(string_list.into_iter().map(|s| String::from(s)).collect())
            } else {
                None
            }
        }
    }
    /// Returns `true` if the context menu was invoked on an editable node.
    pub fn is_editable(&self) -> bool {
        unsafe {
            self.0.is_editable.unwrap()(self.as_ptr()) != 0
        }
    }
    /// Returns `true` if the context menu was invoked on an editable node where
    /// spell-check is enabled.
    pub fn is_spell_check_enabled(&self) -> bool {
        unsafe {
            self.0.is_spell_check_enabled.unwrap()(self.as_ptr()) != 0
        }
    }
    /// Returns flags representing the actions supported by the editable node, if
    /// any, that the context menu was invoked on.
    pub fn get_edit_state_flags(&self) -> ContextMenuEditStateFlags {
        unsafe {
            ContextMenuEditStateFlags::from_bits_truncate(self.0.get_edit_state_flags.unwrap()(self.as_ptr()).0)
        }
    }
    /// Returns `true` if the context menu contains items specified by the
    /// renderer process (for example, plugin placeholder or pepper plugin menu
    /// items).
    pub fn is_custom_menu(&self) -> bool {
        unsafe {
            self.0.is_custom_menu.unwrap()(self.as_ptr()) != 0
        }
    }
    /// Returns `true` if the context menu was invoked from a pepper plugin.
    pub fn is_pepper_menu(&self) -> bool {
        unsafe {
            self.0.is_pepper_menu.unwrap()(self.as_ptr()) != 0
        }
    }
}
