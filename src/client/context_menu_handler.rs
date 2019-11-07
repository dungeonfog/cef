use crate::{
    browser::{Browser},
    color::Color,
    events::EventFlags,
    frame::Frame,
    refcounted::{RefCountedPtr, Wrapper},
    string::{CefString, CefStringList},
};
use cef_sys::{
    cef_browser_t,
    cef_frame_t,
    cef_context_menu_handler_t,
    _cef_context_menu_params_t,
    _cef_menu_model_t,
    _cef_run_context_menu_callback_t,
    cef_event_flags_t,
    cef_context_menu_type_flags_t,
    cef_string_userfree_t,
    cef_string_list_t,
    cef_context_menu_media_type_t,
    cef_context_menu_media_state_flags_t,
    cef_context_menu_edit_state_flags_t,
    cef_menu_color_type_t,
    cef_string_t,
    cef_color_t,
    cef_menu_item_type_t,
};
use num_enum::UnsafeFromPrimitive;
use std::{
    convert::TryInto,
    os::raw::c_int,
    mem::ManuallyDrop,
};
use bitflags::bitflags;

macro_rules! id {
    ($vis:vis struct $Id:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        $vis struct $Id(i32);
        impl $Id {
            pub const fn get(self) -> i32 {
                self.0
            }
            pub fn new(raw: i32) -> Option<$Id> {
                if raw < 0 {
                    None
                } else {
                    Some($Id(raw))
                }
            }
            pub fn unique() -> $Id {
                use std::sync::atomic::{AtomicI32, Ordering};
                static COUNTER: AtomicI32 = AtomicI32::new(1);
                let new = COUNTER.fetch_add(1, Ordering::SeqCst);
                assert!(new > 0);
                $Id(new)
            }
            pub unsafe fn from_unchecked(raw: i32) -> $Id {
                $Id(raw)
            }
        }
    };
}

id!(pub struct CommandId);
id!(pub struct GroupId);

/// Supported color types for menu items.
#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum MenuColorType {
    Text = cef_menu_color_type_t::CEF_MENU_COLOR_TEXT,
    TextHovered = cef_menu_color_type_t::CEF_MENU_COLOR_TEXT_HOVERED,
    TextAccelerator = cef_menu_color_type_t::CEF_MENU_COLOR_TEXT_ACCELERATOR,
    TextAcceleratorHovered = cef_menu_color_type_t::CEF_MENU_COLOR_TEXT_ACCELERATOR_HOVERED,
    Background = cef_menu_color_type_t::CEF_MENU_COLOR_BACKGROUND,
    BackgroundHovered = cef_menu_color_type_t::CEF_MENU_COLOR_BACKGROUND_HOVERED,
    Count = cef_menu_color_type_t::CEF_MENU_COLOR_COUNT,
}

/// Supported menu item types.
#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum MenuItemType {
    None = cef_menu_item_type_t::MENUITEMTYPE_NONE,
    Command = cef_menu_item_type_t::MENUITEMTYPE_COMMAND,
    Check = cef_menu_item_type_t::MENUITEMTYPE_CHECK,
    Radio = cef_menu_item_type_t::MENUITEMTYPE_RADIO,
    Separator = cef_menu_item_type_t::MENUITEMTYPE_SEPARATOR,
    Submenu = cef_menu_item_type_t::MENUITEMTYPE_SUBMENU,
}

bitflags!{
    pub struct Modifiers: u8 {
        const SHIFT = 1 << 0;
        const CTRL = 1 << 1;
        const ALT = 1 << 2;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Accelerator {
    pub key: i32,
    pub modifiers: Modifiers
}

impl Accelerator {
    fn from_raw(key: c_int, shift_pressed: c_int, ctrl_pressed: c_int, alt_pressed: c_int) -> Accelerator {
        let mut modifiers = Modifiers::empty();
        modifiers.set(Modifiers::SHIFT, shift_pressed != 0);
        modifiers.set(Modifiers::CTRL, ctrl_pressed != 0);
        modifiers.set(Modifiers::ALT, alt_pressed != 0);
        Accelerator {
            key,
            modifiers,
        }
    }

    fn set_raw(self, key: &mut c_int, shift_pressed: &mut c_int, ctrl_pressed: &mut c_int, alt_pressed: &mut c_int) {
        *key = self.key;
        *shift_pressed = self.modifiers.contains(Modifiers::SHIFT) as c_int;
        *ctrl_pressed = self.modifiers.contains(Modifiers::CTRL) as c_int;
        *alt_pressed = self.modifiers.contains(Modifiers::ALT) as c_int;
    }

    fn to_raw(self) -> (c_int, c_int, c_int, c_int) {
        (
            self.key,
            self.modifiers.contains(Modifiers::SHIFT) as c_int,
            self.modifiers.contains(Modifiers::CTRL) as c_int,
            self.modifiers.contains(Modifiers::ALT) as c_int,
        )
    }
}

bitflags!{
    /// Supported context menu type flags.
    pub struct ContextMenuTypeFlags: i32 {
        /// No node is selected.
        const NONE = cef_context_menu_type_flags_t::CM_TYPEFLAG_NONE;
        /// The top page is selected.
        const PAGE = cef_context_menu_type_flags_t::CM_TYPEFLAG_PAGE;
        /// A subframe page is selected.
        const FRAME = cef_context_menu_type_flags_t::CM_TYPEFLAG_FRAME;
        /// A link is selected.
        const LINK = cef_context_menu_type_flags_t::CM_TYPEFLAG_LINK;
        /// A media node is selected.
        const MEDIA = cef_context_menu_type_flags_t::CM_TYPEFLAG_MEDIA;
        /// There is a textual or mixed selection that is selected.
        const SELECTION = cef_context_menu_type_flags_t::CM_TYPEFLAG_SELECTION;
        /// An editable element is selected.
        const EDITABLE = cef_context_menu_type_flags_t::CM_TYPEFLAG_EDITABLE;
    }
}
bitflags!{
    /// Supported context menu media state bit flags.
    pub struct ContextMenuMediaStateFlags: i32 {
        const NONE = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_NONE;
        const ERROR = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_ERROR;
        const PAUSED = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_PAUSED;
        const MUTED = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_MUTED;
        const LOOP = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_LOOP;
        const CAN_SAVE = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_CAN_SAVE;
        const HAS_AUDIO = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_HAS_AUDIO;
        const HAS_VIDEO = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_HAS_VIDEO;
        const CONTROL_ROOT_ELEMENT = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_CONTROL_ROOT_ELEMENT;
        const CAN_PRINT = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_CAN_PRINT;
        const CAN_ROTATE = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_CAN_ROTATE;
    }
}
bitflags!{
    /// Supported context menu edit state bit flags.
    pub struct ContextMenuEditStateFlags: i32 {
        const NONE = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_NONE;
        const CAN_UNDO = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_UNDO;
        const CAN_REDO = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_REDO;
        const CAN_CUT = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_CUT;
        const CAN_COPY = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_COPY;
        const CAN_PASTE = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_PASTE;
        const CAN_DELETE = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_DELETE;
        const CAN_SELECT_ALL = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_SELECT_ALL;
        const CAN_TRANSLATE = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_TRANSLATE;
    }
}

ref_counted_ptr!{
    /// Implement this structure to handle context menu events. The functions of this
    /// structure will be called on the UI thread.
    pub struct ContextMenuHandler(*mut cef_context_menu_handler_t);
}

impl ContextMenuHandler {
    pub fn new<C: ContextMenuHandlerCallbacks>(callbacks: C) -> ContextMenuHandler {
        unsafe{ ContextMenuHandler::from_ptr_unchecked(ContextMenuHandlerWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }

    /// Called before a context menu is displayed. `params` provides information
    /// about the context menu state. `model` initially contains the default
    /// context menu. The `model` can be cleared to show no context menu or
    /// modified to show a custom menu. Do not keep references to `params` or
    /// `model` outside of this callback.
    pub fn on_before_context_menu(
        &self,
        browser: Browser,
        frame: Frame,
        params: ContextMenuParams,
        model: MenuModel
    ) {
        unsafe {
            self.0.on_before_context_menu.unwrap()(
                self.as_ptr(),
                browser.into_raw(),
                frame.into_raw(),
                params.into_raw(),
                model.into_raw(),
            );
        }
    }
    /// Called to allow custom display of the context menu. `params` provides
    /// information about the context menu state. `model` contains the context menu
    /// model resulting from OnBeforeContextMenu. For custom display return `true`
    /// and execute `callback` either synchronously or asynchronously with the
    /// selected command ID. For default display return `false`. Do not keep
    /// references to `params` or `model` outside of this callback.
    pub fn run_context_menu(
        &self,
        browser: Browser,
        frame: Frame,
        params: ContextMenuParams,
        model: MenuModel,
        callback: RunContextMenu
    ) -> bool {
        unsafe {
            self.0.run_context_menu.unwrap()(
                self.as_ptr(),
                browser.into_raw(),
                frame.into_raw(),
                params.into_raw(),
                model.into_raw(),
                callback.into_raw()
            ) != 0
        }
    }
    /// Called to execute a command selected from the context menu. Return `true`
    /// if the command was handled or `false` for the default implementation. See
    /// cef_menu_id_t for the command ids that have default implementations. All
    /// user-defined command ids should be between MENU_ID_USER_FIRST and
    /// MENU_ID_USER_LAST. `params` will have the same values as what was passed to
    /// on_before_context_menu(). Do not keep a reference to `params` outside of
    /// this callback.
    pub fn on_context_menu_command(
        &self,
        browser: Browser,
        frame: Frame,
        params: ContextMenuParams,
        command_id: CommandId,
        event_flags: EventFlags
    ) -> bool {
        unsafe {
            self.0.on_context_menu_command.unwrap()(
                self.as_ptr(),
                browser.into_raw(),
                frame.into_raw(),
                params.into_raw(),
                command_id.get(),
                cef_event_flags_t(event_flags.bits()),
            ) != 0
        }
    }
    /// Called when the context menu is dismissed irregardless of whether the menu
    /// was NULL or a command was selected.
    pub fn on_context_menu_dismissed(
        &self,
        browser: Browser,
        frame: Frame
    ) {
        unsafe {
            self.0.on_context_menu_dismissed.unwrap()(
                self.as_ptr(),
                browser.into_raw(),
                frame.into_raw(),
            );
        }
    }
}

ref_counted_ptr!{
    /// Provides information about the context menu state. The ethods of this
    /// structure can only be accessed on browser process the UI thread.
    pub struct ContextMenuParams(*mut _cef_context_menu_params_t);
}

impl ContextMenuParams {
    pub fn new<C: ContextMenuParamsCallbacks>(callbacks: C) -> ContextMenuParams {
        unsafe{ ContextMenuParams::from_ptr_unchecked(ContextMenuParamsWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }

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
            ContextMenuTypeFlags::from_bits_truncate(self.0.get_type_flags.unwrap()(self.as_ptr()))
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
            ContextMenuMediaStateFlags::from_bits_truncate(self.0.get_media_state_flags.unwrap()(self.as_ptr()))
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
            ContextMenuEditStateFlags::from_bits_truncate(self.0.get_edit_state_flags.unwrap()(self.as_ptr()))
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

ref_counted_ptr!{
    /// Supports creation and modification of menus. See cef_menu_id_t for the
    /// command ids that have default implementations. All user-defined command ids
    /// should be between MENU_ID_USER_FIRST and MENU_ID_USER_LAST. The functions of
    /// this structure can only be accessed on the browser process the UI thread.
    pub struct MenuModel(*mut _cef_menu_model_t);
}

impl MenuModel {
    pub fn new<C: MenuModelCallbacks>(callbacks: C) -> MenuModel {
        unsafe{ MenuModel::from_ptr_unchecked(MenuModelWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }

    /// Returns `true` if this menu is a submenu.
    pub fn is_sub_menu(&self) -> bool {
        unsafe {
            self.0.is_sub_menu.unwrap()(self.as_ptr()) != 0
        }
    }
    /// Clears the menu. Returns `true` on success.
    pub fn clear(&self) -> bool {
        unsafe {
            self.0.clear.unwrap()(self.as_ptr()) != 0
        }
    }
    /// Returns the number of items in this menu.
    pub fn get_count(&self) -> usize {
        unsafe {
            c(self.0.clear.unwrap()(self.as_ptr()))
        }
    }
    /// Add a separator to the menu. Returns `true` on success.
    pub fn add_separator(&self) -> bool {
        unsafe {
            self.0.add_separator.unwrap()(self.as_ptr()) != 0
        }
    }
    /// Add an item to the menu. Returns `true` on success.
    pub fn add_item(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> bool {
        unsafe {
            self.0.add_item.unwrap()(
                self.as_ptr(),
                command_id.get(),
                CefString::new(label).as_ptr()
            ) != 0
        }
    }
    /// Add a check item to the menu. Returns `true` on success.
    pub fn add_check_item(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> bool {
        unsafe {
            self.0.add_check_item.unwrap()(
                self.as_ptr(),
                command_id.get(),
                CefString::new(label).as_ptr()
            ) != 0
        }
    }
    /// Add a radio item to the menu. Only a single item with the specified
    /// `group_id` can be checked at a time. Returns `true` on success.
    pub fn add_radio_item(
        &self,
        command_id: CommandId,
        label: &str,
        group_id: GroupId,
    ) -> bool {
        unsafe {
            self.0.add_radio_item.unwrap()(
                self.as_ptr(),
                command_id.get(),
                CefString::new(label).as_ptr(),
                group_id.get()
            ) != 0
        }
    }
    /// Add a sub-menu to the menu. The new sub-menu is returned.
    pub fn add_sub_menu(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> MenuModel {
        unsafe {
            MenuModel::from_ptr_unchecked(self.0.add_sub_menu.unwrap()(
                self.as_ptr(),
                command_id.get(),
                CefString::new(label).as_ptr()
            ))
        }
    }
    /// Insert a separator in the menu at the specified `index`. Returns `true`
    /// on success.
    pub fn insert_separator_at(
        &self,
        index: usize,
    ) -> bool {
        unsafe {
            self.0.insert_separator_at.unwrap()(
                self.as_ptr(),
                c(index),
            ) != 0
        }
    }
    /// Insert an item in the menu at the specified `index`. Returns `true` on
    /// success.
    pub fn insert_item_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
    ) -> bool {
        unsafe {
            self.0.insert_item_at.unwrap()(
                self.as_ptr(),
                c(index),
                command_id.get(),
                CefString::new(label).as_ptr(),
            ) != 0
        }
    }
    /// Insert a check item in the menu at the specified `index`. Returns `true`
    /// on success.
    pub fn insert_check_item_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
    ) -> bool {
        unsafe {
            self.0.insert_check_item_at.unwrap()(
                self.as_ptr(),
                c(index),
                command_id.get(),
                CefString::new(label).as_ptr(),
            ) != 0
        }
    }
    /// Insert a radio item in the menu at the specified `index`. Only a single
    /// item with the specified `group_id` can be checked at a time. Returns `true`
    /// on success.
    pub fn insert_radio_item_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
        group_id: GroupId,
    ) -> bool {
        unsafe {
            self.0.insert_radio_item_at.unwrap()(
                self.as_ptr(),
                c(index),
                command_id.get(),
                CefString::new(label).as_ptr(),
                group_id.get(),
            ) != 0
        }
    }
    /// Insert a sub-menu in the menu at the specified `index`. The new sub-menu is
    /// returned.
    pub fn insert_sub_menu_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
    ) -> MenuModel {
        unsafe {
            MenuModel::from_ptr_unchecked(self.0.insert_sub_menu_at.unwrap()(
                self.as_ptr(),
                c(index),
                command_id.get(),
                CefString::new(label).as_ptr()
            ))
        }
    }
    /// Removes the item with the specified `command_id`. Returns `true` on
    /// success.
    pub fn remove(
        &self,
        command_id: CommandId,
    ) -> bool {
        unsafe {
            self.0.remove.unwrap()(
                self.as_ptr(),
                command_id.get(),
            ) != 0
        }
    }
    /// Removes the item at the specified `index`. Returns `true` on success.
    pub fn remove_at(
        &self,
        index: usize,
    ) -> bool {
        unsafe {
            self.0.remove_at.unwrap()(
                self.as_ptr(),
                c(index),
            ) != 0
        }
    }
    /// Returns the index associated with the specified `command_id` or -1 if not
    /// found due to the command id not existing in the menu.
    pub fn get_index_of(
        &self,
        command_id: CommandId,
    ) -> Option<usize> {
        unsafe {
            let index = self.0.get_index_of.unwrap()(
                self.as_ptr(),
                command_id.get(),
            );
            if index >= 0 {
                Some(c(index))
            } else {
                None
            }
        }
    }

    /// Returns the command id at the specified `index` or -1 if not found due to
    /// invalid range or the index being a separator.
    pub fn get_command_id_at(
        &self,
        index: usize,
    ) -> Option<usize> {
        unsafe {
            let index = self.0.get_command_id_at.unwrap()(
                self.as_ptr(),
                c(index),
            );
            if index >= 0 {
                Some(c(index))
            } else {
                None
            }
        }
    }
    /// Sets the command id at the specified `index`. Returns `true` on success.
    pub fn set_command_id_at(
        &self,
        index: usize,
        command_id: CommandId,
    ) -> bool {
        unsafe {
            self.0.set_command_id_at.unwrap()(
                self.as_ptr(),
                c(index),
                command_id.get(),
            ) != 0
        }
    }
    /// Returns the label for the specified `command_id` or NULL if not found.
    pub fn get_label(
        &self,
        command_id: CommandId,
    ) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_label.unwrap()(self.as_ptr(), command_id.get())).into()
        }
    }
    /// Returns the label at the specified `index` or NULL if not found due to
    /// invalid range or the index being a separator.
    pub fn get_label_at(
        &self,
        index: usize,
    ) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_label_at.unwrap()(self.as_ptr(), c(index))).into()
        }
    }
    /// Sets the label for the specified `command_id`. Returns `true` on success.
    pub fn set_label(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> bool {
        unsafe {
            self.0.set_label.unwrap()(
                self.as_ptr(),
                command_id.get(),
                CefString::new(label).as_ptr()
            ) != 0
        }
    }
    /// Set the label at the specified `index`. Returns `true` on success.
    pub fn set_label_at(
        &self,
        index: usize,
        label: &str,
    ) -> bool {
        unsafe {
            self.0.set_label_at.unwrap()(
                self.as_ptr(),
                c(index),
                CefString::new(label).as_ptr()
            ) != 0
        }
    }
    /// Returns the item type for the specified `command_id`.
    pub fn get_type(
        &self,
        command_id: CommandId,
    ) -> MenuItemType {
        unsafe {
            MenuItemType::from_unchecked(self.0.get_type.unwrap()(
                self.as_ptr(),
                command_id.get(),
            ))
        }
    }
    /// Returns the item type at the specified `index`.
    pub fn get_type_at(
        &self,
        index: usize,
    ) -> MenuItemType {
        unsafe {
            MenuItemType::from_unchecked(self.0.get_type.unwrap()(
                self.as_ptr(),
                c(index),
            ))
        }
    }
    /// Returns the group id for the specified `command_id` or -1 if invalid.
    pub fn get_group_id(
        &self,
        command_id: CommandId,
    ) -> Option<GroupId> {
        unsafe {
            GroupId::new(self.0.get_group_id.unwrap()(
                self.as_ptr(),
                command_id.get(),
            ))
        }
    }
    /// Returns the group id at the specified `index` or -1 if invalid.
    pub fn get_group_id_at(
        &self,
        index: usize,
    ) -> Option<GroupId> {
        unsafe {
            GroupId::new(self.0.get_group_id_at.unwrap()(
                self.as_ptr(),
                c(index),
            ))
        }
    }
    /// Sets the group id for the specified `command_id`. Returns `true` on
    /// success.
    pub fn set_group_id(
        &self,
        command_id: CommandId,
        group_id: GroupId,
    ) -> bool {
        unsafe {
            self.0.set_group_id.unwrap()(
                self.as_ptr(),
                command_id.get(),
                group_id.get(),
            ) != 0
        }
    }
    /// Sets the group id at the specified `index`. Returns `true` on success.
    pub fn set_group_id_at(
        &self,
        index: usize,
        group_id: GroupId,
    ) -> bool {
        unsafe {
            self.0.set_group_id_at.unwrap()(
                self.as_ptr(),
                c(index),
                group_id.get(),
            ) != 0
        }
    }
    /// Returns the submenu for the specified `command_id` or NULL if invalid.
    pub fn get_sub_menu(
        &self,
        command_id: CommandId,
    ) -> MenuModel {
        unsafe {
            MenuModel::from_ptr_unchecked(self.0.get_sub_menu.unwrap()(
                self.as_ptr(),
                command_id.get(),
            ))
        }
    }
    /// Returns the submenu at the specified `index` or NULL if invalid.
    pub fn get_sub_menu_at(
        &self,
        index: usize,
    ) -> MenuModel {
        unsafe {
            MenuModel::from_ptr_unchecked(self.0.get_sub_menu_at.unwrap()(
                self.as_ptr(),
                c(index),
            ))
        }
    }
    /// Returns `true` if the specified `command_id` is visible.
    pub fn is_visible(
        &self,
        command_id: CommandId,
    ) -> bool {
        unsafe {
            self.0.is_visible.unwrap()(
                self.as_ptr(),
                command_id.get(),
            ) != 0
        }
    }
    /// Returns `true` if the specified `index` is visible.
    pub fn is_visible_at(
        &self,
        index: usize,
    ) -> bool {
        unsafe {
            self.0.is_visible_at.unwrap()(
                self.as_ptr(),
                c(index),
            ) != 0
        }
    }
    /// Change the visibility of the specified `command_id`. Returns `true` on
    /// success.
    pub fn set_visible(
        &self,
        command_id: CommandId,
        visible: bool,
    ) -> bool {
        unsafe {
            self.0.set_visible.unwrap()(
                self.as_ptr(),
                command_id.get(),
                visible as c_int
            ) != 0
        }
    }
    /// Change the visibility at the specified `index`. Returns `true` on
    /// success.
    pub fn set_visible_at(
        &self,
        index: usize,
        visible: bool,
    ) -> bool {
        unsafe {
            self.0.set_visible_at.unwrap()(
                self.as_ptr(),
                c(index),
                visible as c_int
            ) != 0
        }
    }
    /// Returns `true` if the specified `command_id` is enabled.
    pub fn is_enabled(
        &self,
        command_id: CommandId,
    ) -> bool {
        unsafe {
            self.0.is_enabled.unwrap()(
                self.as_ptr(),
                command_id.get()
            ) != 0
        }
    }
    /// Returns `true` if the specified `index` is enabled.
    pub fn is_enabled_at(
        &self,
        index: usize,
    ) -> bool {
        unsafe {
            self.0.is_enabled_at.unwrap()(
                self.as_ptr(),
                c(index),
            ) != 0
        }
    }
    /// Change the enabled status of the specified `command_id`. Returns `true`
    /// on success.
    pub fn set_enabled(
        &self,
        command_id: CommandId,
        enabled: bool,
    ) -> bool {
        unsafe {
            self.0.set_enabled.unwrap()(
                self.as_ptr(),
                command_id.get(),
                enabled as c_int,
            ) != 0
        }
    }
    /// Change the enabled status at the specified `index`. Returns `true` on
    /// success.
    pub fn set_enabled_at(
        &self,
        index: usize,
        enabled: bool,
    ) -> bool {
        unsafe {
            self.0.set_enabled_at.unwrap()(
                self.as_ptr(),
                c(index),
                enabled as c_int,
            ) != 0
        }
    }
    /// Returns `true` if the specified `command_id` is checked. Only applies to
    /// check and radio items.
    pub fn is_checked(
        &self,
        command_id: CommandId,
    ) -> bool {
        unsafe {
            self.0.is_checked.unwrap()(
                self.as_ptr(),
                command_id.get(),
            ) != 0
        }
    }
    /// Returns `true` if the specified `index` is checked. Only applies to check
    /// and radio items.
    pub fn is_checked_at(
        &self,
        index: usize,
    ) -> bool {
        unsafe {
            self.0.is_checked_at.unwrap()(
                self.as_ptr(),
                c(index),
            ) != 0
        }
    }
    /// Check the specified `command_id`. Only applies to check and radio items.
    /// Returns `true` on success.
    pub fn set_checked(
        &self,
        command_id: CommandId,
        checked: bool,
    ) -> bool {
        unsafe {
            self.0.set_checked.unwrap()(
                self.as_ptr(),
                command_id.get(),
                checked as c_int,
            ) != 0
        }
    }
    /// Check the specified `index`. Only applies to check and radio items. Returns
    /// `true` on success.
    pub fn set_checked_at(
        &self,
        index: usize,
        checked: bool,
    ) -> bool {
        unsafe {
            self.0.set_checked_at.unwrap()(
                self.as_ptr(),
                c(index),
                checked as c_int,
            ) != 0
        }
    }
    /// Returns `true` if the specified `command_id` has a keyboard accelerator
    /// assigned.
    pub fn has_accelerator(
        &self,
        command_id: CommandId,
    ) -> bool {
        unsafe {
            self.0.has_accelerator.unwrap()(
                self.as_ptr(),
                command_id.get(),
            ) != 0
        }
    }
    /// Returns `true` if the specified `index` has a keyboard accelerator
    /// assigned.
    pub fn has_accelerator_at(
        &self,
        index: usize,
    ) -> bool {
        unsafe {
            self.0.has_accelerator_at.unwrap()(
                self.as_ptr(),
                c(index),
            ) != 0
        }
    }
    /// Set the keyboard accelerator for the specified `command_id`. `key_code` can
    /// be any virtual key or character value. Returns `true` on success.
    pub fn set_accelerator(
        &self,
        command_id: CommandId,
        accelerator: Accelerator,
    ) -> bool {
        unsafe {
            let (key, shift_pressed, ctrl_pressed, alt_pressed) = accelerator.to_raw();
            self.0.set_accelerator.unwrap()(
                self.as_ptr(),
                command_id.get(),
                key,
                shift_pressed,
                ctrl_pressed,
                alt_pressed,
            ) != 0
        }
    }
    /// Set the keyboard accelerator at the specified `index`. `key_code` can be
    /// any virtual key or character value. Returns `true` on success.
    pub fn set_accelerator_at(
        &self,
        index: i32,
        accelerator: Accelerator,
    ) -> bool {
        unsafe {
            let (key, shift_pressed, ctrl_pressed, alt_pressed) = accelerator.to_raw();
            self.0.set_accelerator_at.unwrap()(
                self.as_ptr(),
                c(index),
                key,
                shift_pressed,
                ctrl_pressed,
                alt_pressed,
            ) != 0
        }
    }
    /// Remove the keyboard accelerator for the specified `command_id`. Returns
    /// `true` on success.
    pub fn remove_accelerator(
        &self,
        command_id: CommandId,
    ) -> bool {
        unsafe {
            self.0.remove_accelerator.unwrap()(
                self.as_ptr(),
                command_id.get(),
            ) != 0
        }
    }
    /// Remove the keyboard accelerator at the specified `index`. Returns `true`
    /// on success.
    pub fn remove_accelerator_at(
        &self,
        index: usize,
    ) -> bool {
        unsafe {
            self.0.remove_accelerator_at.unwrap()(
                self.as_ptr(),
                c(index),
            ) != 0
        }
    }
    /// Retrieves the keyboard accelerator for the specified `command_id`. Returns
    /// `true` on success.
    pub fn get_accelerator(
        &self,
        command_id: CommandId,
    ) -> Option<Accelerator> {
        unsafe {
            let (mut key, mut shift_pressed, mut ctrl_pressed, mut alt_pressed) = (0, 0, 0, 0);
            let has_accelerator = self.0.get_accelerator.unwrap()(
                self.as_ptr(),
                command_id.get(),
                &mut key,
                &mut shift_pressed,
                &mut ctrl_pressed,
                &mut alt_pressed,
            ) != 0;
            if has_accelerator {
                Some(Accelerator::from_raw(key, shift_pressed, ctrl_pressed, alt_pressed))
            } else {
                None
            }
        }
    }
    /// Retrieves the keyboard accelerator for the specified `index`. Returns `true`
    /// on success.
    pub fn get_accelerator_at(
        &self,
        index: usize,
    ) -> Option<Accelerator> {
        unsafe {
            let (mut key, mut shift_pressed, mut ctrl_pressed, mut alt_pressed) = (0, 0, 0, 0);
            let has_accelerator = self.0.get_accelerator_at.unwrap()(
                self.as_ptr(),
                c(index),
                &mut key,
                &mut shift_pressed,
                &mut ctrl_pressed,
                &mut alt_pressed,
            ) != 0;
            if has_accelerator {
                Some(Accelerator::from_raw(key, shift_pressed, ctrl_pressed, alt_pressed))
            } else {
                None
            }
        }
    }
    /// Set the explicit color for `command_id` and `color_type` to `color`.
    /// Specify a `color` value of 0 to remove the explicit color. If no explicit
    /// color or default color is set for `color_type` then the system color will
    /// be used. Returns `true` on success.
    pub fn set_color(
        &self,
        command_id: CommandId,
        color_type: MenuColorType,
        color: Color,
    ) -> bool {
        unsafe {
            self.0.set_color.unwrap()(
                self.as_ptr(),
                command_id.get(),
                color_type as _,
                color.get(),
            ) != 0
        }
    }
    /// Set the explicit color for `command_id` and `index` to `color`. Specify a
    /// `color` value of 0 to remove the explicit color. Specify an `index` value
    /// of -1 to set the default color for items that do not have an explicit color
    /// set. If no explicit color or default color is set for `color_type` then the
    /// system color will be used. Returns `true` on success.
    pub fn set_color_at(
        &self,
        index: usize,
        color_type: MenuColorType,
        color: Color,
    ) -> bool {
        unsafe {
            self.0.set_color_at.unwrap()(
                self.as_ptr(),
                c(index),
                color_type as _,
                color.get(),
            ) != 0
        }
    }
    /// Returns in `color` the color that was explicitly set for `command_id` and
    /// `color_type`. If a color was not set then 0 will be returned in `color`.
    /// Returns `true` on success.
    pub fn get_color(
        &self,
        command_id: CommandId,
        color_type: MenuColorType,
    ) -> Option<Color> {
        unsafe {
            let mut color = Color::wrap(0);
            let has_color = self.0.get_color.unwrap()(
                self.as_ptr(),
                command_id.get(),
                color_type as _,
                &mut color.0,
            ) != 0;
            if has_color {
                Some(color)
            } else {
                None
            }
        }
    }
    /// Returns in `color` the color that was explicitly set for `command_id` and
    /// `color_type`. Specify an `index` value of -1 to return the default color in
    /// `color`. If a color was not set then 0 will be returned in `color`. Returns
    /// `true` on success.
    pub fn get_color_at(
        &self,
        index: usize,
        color_type: MenuColorType,
    ) -> Option<Color> {
        unsafe {
            let mut color = Color::wrap(0);
            let has_color = self.0.get_color_at.unwrap()(
                self.as_ptr(),
                c(index),
                color_type as _,
                &mut color.0,
            ) != 0;
            if has_color {
                Some(color)
            } else {
                None
            }
        }
    }
    /// Sets the font list for the specified `command_id`. If `font_list` is NULL
    /// the system font will be used. Returns `true` on success. The format is
    /// "<FONT_FAMILY_LIST>,[STYLES] <SIZE>", where: - FONT_FAMILY_LIST is a comma-
    /// separated list of font family names, - STYLES is an optional space-
    /// separated list of style names (case-sensitive
    ///   "Bold" and "Italic" are supported), and
    /// - SIZE is an integer font size in pixels with the suffix "px".
    ///
    /// Here are examples of valid font description strings: - "Arial, Helvetica,
    /// Bold Italic 14px" - "Arial, 14px"
    pub fn set_font_list(
        &self,
        command_id: CommandId,
        font_list: &str,
    ) -> bool {
        unsafe {
            self.0.set_font_list.unwrap()(
                self.as_ptr(),
                command_id.get(),
                CefString::new(font_list).as_ptr(),
            ) != 0
        }
    }
    /// Sets the font list for the specified `index`. Specify an `index` value of
    /// -1 to set the default font. If `font_list` is NULL the system font will be
    /// used. Returns `true` on success. The format is
    /// "<FONT_FAMILY_LIST>,[STYLES] <SIZE>", where: - FONT_FAMILY_LIST is a comma-
    /// separated list of font family names, - STYLES is an optional space-
    /// separated list of style names (case-sensitive
    ///   "Bold" and "Italic" are supported), and
    /// - SIZE is an integer font size in pixels with the suffix "px".
    ///
    /// Here are examples of valid font description strings: - "Arial, Helvetica,
    /// Bold Italic 14px" - "Arial, 14px"
    pub fn set_font_list_at(
        &self,
        index: usize,
        font_list: &str,
    ) -> bool {
        unsafe {
            self.0.set_font_list_at.unwrap()(
                self.as_ptr(),
                c(index),
                CefString::new(font_list).as_ptr(),
            ) != 0
        }
    }
}

ref_counted_ptr!{
    /// Callback structure used for continuation of custom context menu display.
    pub struct RunContextMenu(*mut _cef_run_context_menu_callback_t);
}

impl RunContextMenu {
    pub fn new<C: RunContextMenuCallbacks>(callbacks: C) -> RunContextMenu {
        unsafe{ RunContextMenu::from_ptr_unchecked(RunContextMenuWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }

    /// Complete context menu display by selecting the specified `command_id` and
    /// `event_flags`.
    pub fn cont(&self, command_id: CommandId, event_flags: EventFlags) {
        unsafe {
            self.0.cont.unwrap()(
                self.as_ptr(),
                command_id.get(),
                cef_event_flags_t(event_flags.bits())
            )
        }
    }
    /// Cancel context menu display.
    pub fn cancel(&self) {
        unsafe {
            self.0.cancel.unwrap()(
                self.as_ptr(),
            )
        }
    }
}

pub struct ContextMenuHandlerWrapper(Box<dyn ContextMenuHandlerCallbacks>);
pub struct ContextMenuParamsWrapper(Box<dyn ContextMenuParamsCallbacks>);
pub struct MenuModelWrapper(Box<dyn MenuModelCallbacks>);
pub struct RunContextMenuWrapper(Box<dyn RunContextMenuCallbacks>);
impl Wrapper for ContextMenuHandlerWrapper {
    type Cef = cef_context_menu_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_context_menu_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_before_context_menu: Some(Self::on_before_context_menu),
                run_context_menu: Some(Self::run_context_menu),
                on_context_menu_command: Some(Self::on_context_menu_command),
                on_context_menu_dismissed: Some(Self::on_context_menu_dismissed),
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
}
impl Wrapper for ContextMenuParamsWrapper {
    type Cef = _cef_context_menu_params_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            _cef_context_menu_params_t {
                base: unsafe { std::mem::zeroed() },
                get_xcoord: Some(Self::get_xcoord),
                get_ycoord: Some(Self::get_ycoord),
                get_type_flags: Some(Self::get_type_flags),
                get_link_url: Some(Self::get_link_url),
                get_unfiltered_link_url: Some(Self::get_unfiltered_link_url),
                get_source_url: Some(Self::get_source_url),
                has_image_contents: Some(Self::has_image_contents),
                get_title_text: Some(Self::get_title_text),
                get_page_url: Some(Self::get_page_url),
                get_frame_url: Some(Self::get_frame_url),
                get_frame_charset: Some(Self::get_frame_charset),
                get_media_type: Some(Self::get_media_type),
                get_media_state_flags: Some(Self::get_media_state_flags),
                get_selection_text: Some(Self::get_selection_text),
                get_misspelled_word: Some(Self::get_misspelled_word),
                get_dictionary_suggestions: Some(Self::get_dictionary_suggestions),
                is_editable: Some(Self::is_editable),
                is_spell_check_enabled: Some(Self::is_spell_check_enabled),
                get_edit_state_flags: Some(Self::get_edit_state_flags),
                is_custom_menu: Some(Self::is_custom_menu),
                is_pepper_menu: Some(Self::is_pepper_menu),
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
}
impl Wrapper for MenuModelWrapper {
    type Cef = _cef_menu_model_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            _cef_menu_model_t {
                base: unsafe { std::mem::zeroed() },
                is_sub_menu: Some(Self::is_sub_menu),
                clear: Some(Self::clear),
                get_count: Some(Self::get_count),
                add_separator: Some(Self::add_separator),
                add_item: Some(Self::add_item),
                add_check_item: Some(Self::add_check_item),
                add_radio_item: Some(Self::add_radio_item),
                add_sub_menu: Some(Self::add_sub_menu),
                insert_separator_at: Some(Self::insert_separator_at),
                insert_item_at: Some(Self::insert_item_at),
                insert_check_item_at: Some(Self::insert_check_item_at),
                insert_radio_item_at: Some(Self::insert_radio_item_at),
                insert_sub_menu_at: Some(Self::insert_sub_menu_at),
                remove: Some(Self::remove),
                remove_at: Some(Self::remove_at),
                get_index_of: Some(Self::get_index_of),
                get_command_id_at: Some(Self::get_command_id_at),
                set_command_id_at: Some(Self::set_command_id_at),
                get_label: Some(Self::get_label),
                get_label_at: Some(Self::get_label_at),
                set_label: Some(Self::set_label),
                set_label_at: Some(Self::set_label_at),
                get_type: Some(Self::get_type),
                get_type_at: Some(Self::get_type_at),
                get_group_id: Some(Self::get_group_id),
                get_group_id_at: Some(Self::get_group_id_at),
                set_group_id: Some(Self::set_group_id),
                set_group_id_at: Some(Self::set_group_id_at),
                get_sub_menu: Some(Self::get_sub_menu),
                get_sub_menu_at: Some(Self::get_sub_menu_at),
                is_visible: Some(Self::is_visible),
                is_visible_at: Some(Self::is_visible_at),
                set_visible: Some(Self::set_visible),
                set_visible_at: Some(Self::set_visible_at),
                is_enabled: Some(Self::is_enabled),
                is_enabled_at: Some(Self::is_enabled_at),
                set_enabled: Some(Self::set_enabled),
                set_enabled_at: Some(Self::set_enabled_at),
                is_checked: Some(Self::is_checked),
                is_checked_at: Some(Self::is_checked_at),
                set_checked: Some(Self::set_checked),
                set_checked_at: Some(Self::set_checked_at),
                has_accelerator: Some(Self::has_accelerator),
                has_accelerator_at: Some(Self::has_accelerator_at),
                set_accelerator: Some(Self::set_accelerator),
                set_accelerator_at: Some(Self::set_accelerator_at),
                remove_accelerator: Some(Self::remove_accelerator),
                remove_accelerator_at: Some(Self::remove_accelerator_at),
                get_accelerator: Some(Self::get_accelerator),
                get_accelerator_at: Some(Self::get_accelerator_at),
                set_color: Some(Self::set_color),
                set_color_at: Some(Self::set_color_at),
                get_color: Some(Self::get_color),
                get_color_at: Some(Self::get_color_at),
                set_font_list: Some(Self::set_font_list),
                set_font_list_at: Some(Self::set_font_list_at),
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
}
impl Wrapper for RunContextMenuWrapper {
    type Cef = _cef_run_context_menu_callback_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            _cef_run_context_menu_callback_t {
                base: unsafe { std::mem::zeroed() },
                cont: Some(Self::cont),
                cancel: Some(Self::cancel),
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
}

fn c<A, B>(a: A) -> B
    where A: TryInto<B>,
          A::Error: std::fmt::Debug,
{
    a.try_into().unwrap()
}

pub trait ContextMenuHandlerCallbacks: 'static + Send + Sync {
    /// Called before a context menu is displayed. `params` provides information
    /// about the context menu state. `model` initially contains the default
    /// context menu. The `model` can be cleared to show no context menu or
    /// modified to show a custom menu. Do not keep references to `params` or
    /// `model` outside of this callback.
    fn on_before_context_menu(
        &self,
        _browser: Browser,
        _frame: Frame,
        _params: ContextMenuParams,
        _model: MenuModel
    ) {
    }
    /// Called to execute a command selected from the context menu. Return `true`
    /// if the command was handled or `false` for the default implementation. See
    /// cef_menu_id_t for the command ids that have default implementations. All
    /// user-defined command ids should be between MENU_ID_USER_FIRST and
    /// MENU_ID_USER_LAST. `params` will have the same values as what was passed to
    /// on_before_context_menu(). Do not keep a reference to `params` outside of
    /// this callback.
    fn run_context_menu(
        &self,
        _browser: Browser,
        _frame: Frame,
        _params: ContextMenuParams,
        _model: MenuModel,
        _callback: RunContextMenu
    ) -> bool {
        false
    }
    /// Called to execute a command selected from the context menu. Return `true`
    /// if the command was handled or `false` for the default implementation. See
    /// cef_menu_id_t for the command ids that have default implementations. All
    /// user-defined command ids should be between MENU_ID_USER_FIRST and
    /// MENU_ID_USER_LAST. `params` will have the same values as what was passed to
    /// on_before_context_menu(). Do not keep a reference to `params` outside of
    /// this callback.
    fn on_context_menu_command(
        &self,
        _browser: Browser,
        _frame: Frame,
        _params: ContextMenuParams,
        _command_id: CommandId,
        _event_flags: EventFlags
    ) -> bool {
        false
    }
    /// Called when the context menu is dismissed irregardless of whether the menu
    /// was NULL or a command was selected.
    fn on_context_menu_dismissed(
        &self,
        _browser: Browser,
        _frame: Frame
    ) {
    }
}
pub trait ContextMenuParamsCallbacks: 'static + Send + Sync {
    /// Returns the X coordinate of the mouse where the context menu was invoked.
    /// Coords are relative to the associated RenderView's origin.
    fn get_xcoord(&self) -> i32;
    /// Returns the Y coordinate of the mouse where the context menu was invoked.
    /// Coords are relative to the associated RenderView's origin.
    fn get_ycoord(&self) -> i32;
    /// Returns flags representing the type of node that the context menu was
    /// invoked on.
    fn get_type_flags(&self) -> ContextMenuTypeFlags;
    /// Returns the URL of the link, if any, that encloses the node that the
    /// context menu was invoked on.
    fn get_link_url(&self) -> String;
    /// Returns the link URL, if any, to be used ONLY for "copy link address". We
    /// don't validate this field in the frontend process.
    fn get_unfiltered_link_url(&self) -> String;
    /// Returns the source URL, if any, for the element that the context menu was
    /// invoked on. Example of elements with source URLs are img, audio, and video.
    fn get_source_url(&self) -> String;
    /// Returns `true` if the context menu was invoked on an image which has non-
    /// NULL contents.
    fn has_image_contents(&self) -> bool;
    /// Returns the title text or the alt text if the context menu was invoked on
    /// an image.
    fn get_title_text(&self) -> String;
    /// Returns the URL of the top level page that the context menu was invoked on.
    fn get_page_url(&self) -> String;
    /// Returns the URL of the subframe that the context menu was invoked on.
    fn get_frame_url(&self) -> String;
    /// Returns the character encoding of the subframe that the context menu was
    /// invoked on.
    fn get_frame_charset(&self) -> String;
    /// Returns the type of context node that the context menu was invoked on.
    fn get_media_type(&self) -> ContextMenuTypeFlags;
    /// Returns flags representing the actions supported by the media element, if
    /// any, that the context menu was invoked on.
    fn get_media_state_flags(&self) -> ContextMenuMediaStateFlags;
    /// Returns the text of the selection, if any, that the context menu was
    /// invoked on.
    fn get_selection_text(&self) -> String;
    /// Returns the text of the misspelled word, if any, that the context menu was
    /// invoked on.
    fn get_misspelled_word(&self) -> String;
    /// Returns `true` if suggestions exist, `false` otherwise. Fills in
    /// `suggestions` from the spell check service for the misspelled word if there
    /// is one.
    fn get_dictionary_suggestions(&self) -> Option<Vec<String>>;
    /// Returns `true` if the context menu was invoked on an editable node.
    fn is_editable(&self) -> bool;
    /// Returns `true` if the context menu was invoked on an editable node where
    /// spell-check is enabled.
    fn is_spell_check_enabled(&self) -> bool;
    /// Returns flags representing the actions supported by the editable node, if
    /// any, that the context menu was invoked on.
    fn get_edit_state_flags(&self) -> ContextMenuEditStateFlags;
    /// Returns `true` if the context menu contains items specified by the
    /// renderer process (for example, plugin placeholder or pepper plugin menu
    /// items).
    fn is_custom_menu(&self) -> bool;
    /// Returns `true` if the context menu was invoked from a pepper plugin.
    fn is_pepper_menu(&self) -> bool;
}
pub trait MenuModelCallbacks: 'static + Send + Sync {
    /// Returns `true` if this menu is a submenu.
    fn is_sub_menu(&self) -> bool;
    /// Clears the menu. Returns `true` on success.
    fn clear(&self) -> bool;
    /// Returns the number of items in this menu.
    fn get_count(&self) -> usize;
    /// Add a separator to the menu. Returns `true` on success.
    fn add_separator(&self) -> bool;
    /// Add an item to the menu. Returns `true` on success.
    fn add_item(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> bool;
    /// Add a check item to the menu. Returns `true` on success.
    fn add_check_item(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> bool;
    /// Add a radio item to the menu. Only a single item with the specified
    /// `group_id` can be checked at a time. Returns `true` on success.
    fn add_radio_item(
        &self,
        command_id: CommandId,
        label: &str,
        group_id: GroupId,
    ) -> bool;
    /// Add a sub-menu to the menu. The new sub-menu is returned.
    fn add_sub_menu(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> MenuModel;
    /// Insert a separator in the menu at the specified `index`. Returns `true`
    /// on success.
    fn insert_separator_at(
        &self,
        index: usize,
    ) -> bool;
    /// Insert an item in the menu at the specified `index`. Returns `true` on
    /// success.
    fn insert_item_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
    ) -> bool;
    /// Insert a check item in the menu at the specified `index`. Returns `true`
    /// on success.
    fn insert_check_item_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
    ) -> bool;
    /// Insert a radio item in the menu at the specified `index`. Only a single
    /// item with the specified `group_id` can be checked at a time. Returns `true`
    /// on success.
    fn insert_radio_item_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
        group_id: GroupId,
    ) -> bool;
    /// Insert a sub-menu in the menu at the specified `index`. The new sub-menu is
    /// returned.
    fn insert_sub_menu_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
    ) -> MenuModel;
    /// Removes the item with the specified `command_id`. Returns `true` on
    /// success.
    fn remove(
        &self,
        command_id: CommandId,
    ) -> bool;
    /// Removes the item at the specified `index`. Returns `true` on success.
    fn remove_at(
        &self,
        index: usize,
    ) -> bool;
    /// Returns the index associated with the specified `command_id` or -1 if not
    /// found due to the command id not existing in the menu.
    fn get_index_of(
        &self,
        command_id: CommandId,
    ) -> Option<usize>;
    /// Returns the command id at the specified `index` or -1 if not found due to
    /// invalid range or the index being a separator.
    fn get_command_id_at(
        &self,
        index: usize,
    ) -> Option<usize>;
    /// Sets the command id at the specified `index`. Returns `true` on success.
    fn set_command_id_at(
        &self,
        index: usize,
        command_id: CommandId,
    ) -> bool;
    /// Returns the label for the specified `command_id` or NULL if not found.
    fn get_label(
        &self,
        command_id: CommandId,
    ) -> String;
    /// Returns the label at the specified `index` or NULL if not found due to
    /// invalid range or the index being a separator.
    fn get_label_at(
        &self,
        index: usize,
    ) -> String;
    /// Sets the label for the specified `command_id`. Returns `true` on success.
    fn set_label(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> bool;
    /// Set the label at the specified `index`. Returns `true` on success.
    fn set_label_at(
        &self,
        index: usize,
        label: &str,
    ) -> bool;
    /// Returns the item type for the specified `command_id`.
    fn get_type(
        &self,
        command_id: CommandId,
    ) -> MenuItemType;
    /// Returns the item type at the specified `index`.
    fn get_type_at(
        &self,
        index: usize,
    ) -> MenuItemType;
    /// Returns the group id for the specified `command_id` or -1 if invalid.
    fn get_group_id(
        &self,
        command_id: CommandId,
    ) -> Option<GroupId>;
    /// Returns the group id at the specified `index` or -1 if invalid.
    fn get_group_id_at(
        &self,
        index: usize,
    ) -> Option<GroupId>;
    /// Sets the group id for the specified `command_id`. Returns `true` on
    /// success.
    fn set_group_id(
        &self,
        command_id: CommandId,
        group_id: GroupId,
    ) -> bool;
    /// Sets the group id at the specified `index`. Returns `true` on success.
    fn set_group_id_at(
        &self,
        index: usize,
        group_id: GroupId,
    ) -> bool;
    /// Returns the submenu for the specified `command_id` or NULL if invalid.
    fn get_sub_menu(
        &self,
        command_id: CommandId,
    ) -> MenuModel;
    /// Returns the submenu at the specified `index` or NULL if invalid.
    fn get_sub_menu_at(
        &self,
        index: usize,
    ) -> MenuModel;
    /// Returns `true` if the specified `command_id` is visible.
    fn is_visible(
        &self,
        command_id: CommandId,
    ) -> bool;
    /// Returns `true` if the specified `index` is visible.
    fn is_visible_at(
        &self,
        index: usize,
    ) -> bool;
    /// Change the visibility of the specified `command_id`. Returns `true` on
    /// success.
    fn set_visible(
        &self,
        command_id: CommandId,
        visible: bool,
    ) -> bool;
    /// Change the visibility at the specified `index`. Returns `true` on
    /// success.
    fn set_visible_at(
        &self,
        index: usize,
        visible: bool,
    ) -> bool;
    /// Returns `true` if the specified `command_id` is enabled.
    fn is_enabled(
        &self,
        command_id: CommandId,
    ) -> bool;
    /// Returns `true` if the specified `index` is enabled.
    fn is_enabled_at(
        &self,
        index: usize,
    ) -> bool;
    /// Change the enabled status of the specified `command_id`. Returns `true`
    /// on success.
    fn set_enabled(
        &self,
        command_id: CommandId,
        enabled: bool,
    ) -> bool;
    /// Change the enabled status at the specified `index`. Returns `true` on
    /// success.
    fn set_enabled_at(
        &self,
        index: usize,
        enabled: bool,
    ) -> bool;
    /// Returns `true` if the specified `command_id` is checked. Only applies to
    /// check and radio items.
    fn is_checked(
        &self,
        command_id: CommandId,
    ) -> bool;
    /// Returns `true` if the specified `index` is checked. Only applies to check
    /// and radio items.
    fn is_checked_at(
        &self,
        index: usize,
    ) -> bool;
    /// Check the specified `command_id`. Only applies to check and radio items.
    /// Returns `true` on success.
    fn set_checked(
        &self,
        command_id: CommandId,
        checked: bool,
    ) -> bool;
    /// Check the specified `index`. Only applies to check and radio items. Returns
    /// `true` on success.
    fn set_checked_at(
        &self,
        index: usize,
        checked: bool,
    ) -> bool;
    /// Returns `true` if the specified `command_id` has a keyboard accelerator
    /// assigned.
    fn has_accelerator(
        &self,
        command_id: CommandId,
    ) -> bool;
    /// Returns `true` if the specified `index` has a keyboard accelerator
    /// assigned.
    fn has_accelerator_at(
        &self,
        index: usize,
    ) -> bool;
    /// Set the keyboard accelerator for the specified `command_id`. `key_code` can
    /// be any virtual key or character value. Returns `true` on success.
    fn set_accelerator(
        &self,
        command_id: CommandId,
        accelerator: Accelerator,
    ) -> bool;
    /// Set the keyboard accelerator at the specified `index`. `key_code` can be
    /// any virtual key or character value. Returns `true` on success.
    fn set_accelerator_at(
        &self,
        index: i32,
        accelerator: Accelerator,
    ) -> bool;
    /// Remove the keyboard accelerator for the specified `command_id`. Returns
    /// `true` on success.
    fn remove_accelerator(
        &self,
        command_id: CommandId,
    ) -> bool;
    /// Remove the keyboard accelerator at the specified `index`. Returns `true`
    /// on success.
    fn remove_accelerator_at(
        &self,
        index: usize,
    ) -> bool;
    /// Retrieves the keyboard accelerator for the specified `command_id`. Returns
    /// `true` on success.
    fn get_accelerator(
        &self,
        command_id: CommandId,
    ) -> Option<Accelerator>;
    /// Retrieves the keyboard accelerator for the specified `index`. Returns `true`
    /// on success.
    fn get_accelerator_at(
        &self,
        index: usize,
    ) -> Option<Accelerator>;
    /// Set the explicit color for `command_id` and `color_type` to `color`.
    /// Specify a `color` value of 0 to remove the explicit color. If no explicit
    /// color or default color is set for `color_type` then the system color will
    /// be used. Returns `true` on success.
    fn set_color(
        &self,
        command_id: CommandId,
        color_type: MenuColorType,
        color: Color,
    ) -> bool;
    /// Set the explicit color for `command_id` and `index` to `color`. Specify a
    /// `color` value of 0 to remove the explicit color. Specify an `index` value
    /// of -1 to set the default color for items that do not have an explicit color
    /// set. If no explicit color or default color is set for `color_type` then the
    /// system color will be used. Returns `true` on success.
    fn set_color_at(
        &self,
        index: usize,
        color_type: MenuColorType,
        color: Color,
    ) -> bool;
    /// Returns in `color` the color that was explicitly set for `command_id` and
    /// `color_type`. If a color was not set then 0 will be returned in `color`.
    /// Returns `true` on success.
    fn get_color(
        &self,
        command_id: CommandId,
        color_type: MenuColorType,
    ) -> Option<Color>;
    /// Returns in `color` the color that was explicitly set for `command_id` and
    /// `color_type`. Specify an `index` value of -1 to return the default color in
    /// `color`. If a color was not set then 0 will be returned in `color`. Returns
    /// `true` on success.
    fn get_color_at(
        &self,
        index: usize,
        color_type: MenuColorType,
    ) -> Option<Color>;
    /// Sets the font list for the specified `command_id`. If `font_list` is NULL
    /// the system font will be used. Returns `true` on success. The format is
    /// "<FONT_FAMILY_LIST>,[STYLES] <SIZE>", where: - FONT_FAMILY_LIST is a comma-
    /// separated list of font family names, - STYLES is an optional space-
    /// separated list of style names (case-sensitive
    ///   "Bold" and "Italic" are supported), and
    /// - SIZE is an integer font size in pixels with the suffix "px".
    ///
    /// Here are examples of valid font description strings: - "Arial, Helvetica,
    /// Bold Italic 14px" - "Arial, 14px"
    fn set_font_list(
        &self,
        command_id: CommandId,
        font_list: &str,
    ) -> bool;
    /// Sets the font list for the specified `index`. Specify an `index` value of
    /// -1 to set the default font. If `font_list` is NULL the system font will be
    /// used. Returns `true` on success. The format is
    /// "<FONT_FAMILY_LIST>,[STYLES] <SIZE>", where: - FONT_FAMILY_LIST is a comma-
    /// separated list of font family names, - STYLES is an optional space-
    /// separated list of style names (case-sensitive
    ///   "Bold" and "Italic" are supported), and
    /// - SIZE is an integer font size in pixels with the suffix "px".
    ///
    /// Here are examples of valid font description strings: - "Arial, Helvetica,
    /// Bold Italic 14px" - "Arial, 14px"
    fn set_font_list_at(
        &self,
        index: usize,
        font_list: &str,
    ) -> bool;
}
pub trait RunContextMenuCallbacks: 'static + Send + Sync {
    /// Complete context menu display by selecting the specified `command_id` and
    /// `event_flags`.
    fn cont(
        &self,
        command_id: CommandId,
        event_flags: EventFlags,
    );
    /// Cancel context menu display.
    fn cancel(&self);
}

cef_callback_impl!{
    impl for ContextMenuHandlerWrapper: cef_context_menu_handler_t {
        fn on_before_context_menu(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t,
            params: ContextMenuParams: *mut _cef_context_menu_params_t,
            model: MenuModel: *mut _cef_menu_model_t
        ) {
            self.0.on_before_context_menu(browser, frame, params, model);
        }
        fn run_context_menu(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t,
            params: ContextMenuParams: *mut _cef_context_menu_params_t,
            model: MenuModel: *mut _cef_menu_model_t,
            callback: RunContextMenu: *mut _cef_run_context_menu_callback_t
        ) -> c_int {
            self.0.run_context_menu(browser, frame, params, model, callback) as c_int
        }
        fn on_context_menu_command(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t,
            params: ContextMenuParams: *mut _cef_context_menu_params_t,
            command_id: CommandId: c_int,
            event_flags: EventFlags: cef_event_flags_t
        ) -> c_int {
            self.0.on_context_menu_command(browser, frame, params, command_id, event_flags) as c_int
        }
        fn on_context_menu_dismissed(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t
        ) {
            self.0.on_context_menu_dismissed(browser, frame);
        }
    }
}
cef_callback_impl!{
    impl for ContextMenuParamsWrapper: _cef_context_menu_params_t {
        fn get_xcoord(&self) -> c_int {
            self.0.get_xcoord()
        }
        fn get_ycoord(&self) -> c_int {
            self.0.get_xcoord()
        }
        fn get_type_flags(&self) -> cef_context_menu_type_flags_t::Type {
            self.0.get_type_flags().bits()
        }
        fn get_link_url(&self) -> cef_string_userfree_t {
            CefString::new(&self.0.get_link_url()).into()
        }
        fn get_unfiltered_link_url(&self) -> cef_string_userfree_t {
            CefString::new(&self.0.get_unfiltered_link_url()).into()
        }
        fn get_source_url(&self) -> cef_string_userfree_t {
            CefString::new(&self.0.get_source_url()).into()
        }
        fn has_image_contents(&self) -> c_int {
            self.0.has_image_contents() as c_int
        }
        fn get_title_text(&self) -> cef_string_userfree_t {
            CefString::new(&self.0.get_title_text()).into()
        }
        fn get_page_url(&self) -> cef_string_userfree_t {
            CefString::new(&self.0.get_page_url()).into()
        }
        fn get_frame_url(&self) -> cef_string_userfree_t {
            CefString::new(&self.0.get_frame_url()).into()
        }
        fn get_frame_charset(&self) -> cef_string_userfree_t {
            CefString::new(&self.0.get_frame_charset()).into()
        }
        fn get_media_type(&self) -> cef_context_menu_media_type_t::Type {
            self.0.get_media_type().bits()
        }
        fn get_media_state_flags(&self) -> cef_context_menu_media_state_flags_t::Type {
            self.0.get_media_state_flags().bits()
        }
        fn get_selection_text(&self) -> cef_string_userfree_t {
            CefString::new(&self.0.get_selection_text()).into()
        }
        fn get_misspelled_word(&self) -> cef_string_userfree_t {
            CefString::new(&self.0.get_misspelled_word()).into()
        }
        fn get_dictionary_suggestions(
            &self,
            suggestions: ManuallyDrop<CefStringList>: cef_string_list_t
        ) -> c_int {
            if let Some(rust_suggestions) = self.0.get_dictionary_suggestions() {
                let mut suggestions = suggestions;
                suggestions.extend(rust_suggestions.iter().map(|s| s.as_str()));
                1
            } else {
                0
            }
        }
        fn is_editable(&self) -> c_int {
            self.0.is_editable() as c_int
        }
        fn is_spell_check_enabled(&self) -> c_int {
            self.0.is_spell_check_enabled() as c_int
        }
        fn get_edit_state_flags(&self) -> cef_context_menu_edit_state_flags_t::Type {
            self.0.get_edit_state_flags().bits()
        }
        fn is_custom_menu(&self) -> c_int {
            self.0.is_custom_menu() as c_int
        }
        fn is_pepper_menu(&self) -> c_int {
            self.0.is_pepper_menu() as c_int
        }
    }
}
cef_callback_impl!{
    impl for MenuModelWrapper: _cef_menu_model_t {
        fn is_sub_menu(&self) -> c_int {
            self.0.is_sub_menu() as c_int
        }
        fn clear(&self) -> c_int {
            self.0.clear() as c_int
        }
        fn get_count(&self) -> c_int {
            c(self.0.get_count())
        }
        fn add_separator(&self) -> c_int {
            self.0.add_separator() as c_int
        }
        fn add_item(
            &self,
            command_id: CommandId: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            self.0.add_item(command_id, &String::from(label)) as c_int
        }
        fn add_check_item(
            &self,
            command_id: CommandId: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            self.0.add_check_item(command_id, &String::from(label)) as c_int
        }
        fn add_radio_item(
            &self,
            command_id: CommandId: c_int,
            label: &CefString: *const cef_string_t,
            group_id: GroupId: c_int
        ) -> c_int {
            self.0.add_radio_item(command_id, &String::from(label), group_id) as c_int
        }
        fn add_sub_menu(
            &self,
            command_id: CommandId: c_int,
            label: &CefString: *const cef_string_t
        ) -> *mut _cef_menu_model_t {
            self.0.add_sub_menu(command_id, &String::from(label)).into_raw()
        }
        fn insert_separator_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            self.0.insert_separator_at(c(index)) as c_int
        }
        fn insert_item_at(
            &self,
            index: c_int: c_int,
            command_id: CommandId: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            self.0.insert_item_at(c(index), command_id, &String::from(label)) as c_int
        }
        fn insert_check_item_at(
            &self,
            index: c_int: c_int,
            command_id: CommandId: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            self.0.insert_check_item_at(c(index), command_id, &String::from(label)) as c_int
        }
        fn insert_radio_item_at(
            &self,
            index: c_int: c_int,
            command_id: CommandId: c_int,
            label: &CefString: *const cef_string_t,
            group_id: GroupId: c_int
        ) -> c_int {
            self.0.insert_radio_item_at(c(index), command_id, &String::from(label), group_id) as c_int
        }
        fn insert_sub_menu_at(
            &self,
            index: c_int: c_int,
            command_id: CommandId: c_int,
            label: &CefString: *const cef_string_t
        ) -> *mut _cef_menu_model_t {
            self.0.insert_sub_menu_at(c(index), command_id, &String::from(label)).into_raw()
        }
        fn remove(
            &self,
            command_id: CommandId: c_int
        ) -> c_int {
            self.0.remove(command_id) as c_int
        }
        fn remove_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            self.0.remove_at(c(index)) as c_int
        }
        fn get_index_of(
            &self,
            command_id: CommandId: c_int
        ) -> c_int {
            self.0.get_index_of(command_id).map(c).unwrap_or(-1)
        }
        fn get_command_id_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            self.0.get_command_id_at(c(index)).map(c).unwrap_or(-1)
        }
        fn set_command_id_at(
            &self,
            index: c_int: c_int,
            command_id: CommandId: c_int
        ) -> c_int {
            self.0.set_command_id_at(c(index), command_id) as c_int
        }
        fn get_label(
            &self,
            command_id: CommandId: c_int
        ) -> cef_string_userfree_t {
            CefString::new(&self.0.get_label(command_id)).into()
        }
        fn get_label_at(
            &self,
            index: c_int: c_int
        ) -> cef_string_userfree_t {
            CefString::new(&self.0.get_label_at(c(index))).into()
        }
        fn set_label(
            &self,
            command_id: CommandId: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            self.0.set_label(command_id, &String::from(label)) as c_int
        }
        fn set_label_at(
            &self,
            index: c_int: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            self.0.set_label_at(c(index), &String::from(label)) as c_int
        }
        fn get_type(
            &self,
            command_id: CommandId: c_int
        ) -> cef_menu_item_type_t::Type {
            self.0.get_type(command_id) as _
        }
        fn get_type_at(
            &self,
            index: c_int: c_int
        ) -> cef_menu_item_type_t::Type {
            self.0.get_type_at(c(index)) as _
        }
        fn get_group_id(
            &self,
            command_id: CommandId: c_int
        ) -> c_int {
            self.0.get_group_id(command_id).map(|c| c.get()).unwrap_or(-1)
        }
        fn get_group_id_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            self.0.get_group_id_at(c(index)).map(|c| c.get()).unwrap_or(-1)
        }
        fn set_group_id(
            &self,
            command_id: CommandId: c_int,
            group_id: GroupId: c_int
        ) -> c_int {
            self.0.set_group_id(command_id, group_id) as c_int
        }
        fn set_group_id_at(
            &self,
            index: c_int: c_int,
            group_id: GroupId: c_int
        ) -> c_int {
            self.0.set_group_id_at(c(index), group_id) as c_int
        }
        fn get_sub_menu(
            &self,
            command_id: CommandId: c_int
        ) -> *mut _cef_menu_model_t {
            self.0.get_sub_menu(command_id).into_raw()
        }
        fn get_sub_menu_at(
            &self,
            index: c_int: c_int
        ) -> *mut _cef_menu_model_t {
            self.0.get_sub_menu_at(c(index)).into_raw()
        }
        fn is_visible(
            &self,
            command_id: CommandId: c_int
        ) -> c_int {
            self.0.is_visible(command_id) as c_int
        }
        fn is_visible_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            self.0.is_visible_at(c(index)) as c_int
        }
        fn set_visible(
            &self,
            command_id: CommandId: c_int,
            visible: c_int: c_int
        ) -> c_int {
            self.0.set_visible(command_id, visible != 0) as c_int
        }
        fn set_visible_at(
            &self,
            index: c_int: c_int,
            visible: c_int: c_int
        ) -> c_int {
            self.0.set_visible_at(c(index), visible != 0) as c_int
        }
        fn is_enabled(
            &self,
            command_id: CommandId: c_int
        ) -> c_int {
            self.0.is_enabled(command_id) as c_int
        }
        fn is_enabled_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            self.0.is_enabled_at(c(index)) as c_int
        }
        fn set_enabled(
            &self,
            command_id: CommandId: c_int,
            enabled: c_int: c_int
        ) -> c_int {
            self.0.set_enabled(command_id, enabled != 0) as c_int
        }
        fn set_enabled_at(
            &self,
            index: c_int: c_int,
            enabled: c_int: c_int
        ) -> c_int {
            self.0.set_enabled_at(c(index), enabled != 0) as c_int
        }
        fn is_checked(
            &self,
            command_id: CommandId: c_int
        ) -> c_int {
            self.0.is_checked(command_id) as c_int
        }
        fn is_checked_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            self.0.is_checked_at(c(index)) as c_int
        }
        fn set_checked(
            &self,
            command_id: CommandId: c_int,
            checked: c_int: c_int
        ) -> c_int {
            self.0.set_checked(command_id, checked != 0) as c_int
        }
        fn set_checked_at(
            &self,
            index: c_int: c_int,
            checked: c_int: c_int
        ) -> c_int {
            self.0.set_checked_at(c(index), checked != 0) as c_int
        }
        fn has_accelerator(
            &self,
            command_id: CommandId: c_int
        ) -> c_int {
            self.0.has_accelerator(command_id) as c_int
        }
        fn has_accelerator_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            self.0.has_accelerator_at(c(index)) as c_int
        }
        fn set_accelerator(
            &self,
            command_id: CommandId: c_int,
            key_code: c_int: c_int,
            shift_pressed: c_int: c_int,
            ctrl_pressed: c_int: c_int,
            alt_pressed: c_int: c_int
        ) -> c_int {
            self.0.set_accelerator(command_id, Accelerator::from_raw(key_code, shift_pressed, ctrl_pressed, alt_pressed)) as c_int
        }
        fn set_accelerator_at(
            &self,
            index: c_int: c_int,
            key_code: c_int: c_int,
            shift_pressed: c_int: c_int,
            ctrl_pressed: c_int: c_int,
            alt_pressed: c_int: c_int
        ) -> c_int {
            self.0.set_accelerator_at(c(index), Accelerator::from_raw(key_code, shift_pressed, ctrl_pressed, alt_pressed)) as c_int
        }
        fn remove_accelerator(
            &self,
            command_id: CommandId: c_int
        ) -> c_int {
            self.0.remove_accelerator(command_id) as c_int
        }
        fn remove_accelerator_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            self.0.remove_accelerator_at(c(index)) as c_int
        }
        fn get_accelerator(
            &self,
            command_id: CommandId: c_int,
            key_code: &mut c_int: *mut c_int,
            shift_pressed: &mut c_int: *mut c_int,
            ctrl_pressed: &mut c_int: *mut c_int,
            alt_pressed: &mut c_int: *mut c_int
        ) -> c_int {
            if let Some(accelerator) = self.0.get_accelerator(command_id) {
                accelerator.set_raw(key_code, shift_pressed, ctrl_pressed, alt_pressed);
                1
            } else {
                0
            }
        }
        fn get_accelerator_at(
            &self,
            index: c_int: c_int,
            key_code: &mut c_int: *mut c_int,
            shift_pressed: &mut c_int: *mut c_int,
            ctrl_pressed: &mut c_int: *mut c_int,
            alt_pressed: &mut c_int: *mut c_int
        ) -> c_int {
            if let Some(accelerator) = self.0.get_accelerator_at(c(index)) {
                accelerator.set_raw(key_code, shift_pressed, ctrl_pressed, alt_pressed);
                1
            } else {
                0
            }
        }
        fn set_color(
            &self,
            command_id: CommandId: c_int,
            color_type: MenuColorType: cef_menu_color_type_t::Type,
            color: Color: cef_color_t
        ) -> c_int {
            self.0.set_color(command_id, color_type, color) as c_int
        }
        fn set_color_at(
            &self,
            index: c_int: c_int,
            color_type: MenuColorType: cef_menu_color_type_t::Type,
            color: Color: cef_color_t
        ) -> c_int {
            self.0.set_color_at(c(index), color_type, color) as c_int
        }
        fn get_color(
            &self,
            command_id: CommandId: c_int,
            color_type: MenuColorType: cef_menu_color_type_t::Type,
            color: &mut Color: *mut cef_color_t
        ) -> c_int {
            if let Some(color_rust) = self.0.get_color(command_id, color_type) {
                *color = color_rust;
                1
            } else {
                0
            }
        }
        fn get_color_at(
            &self,
            index: c_int: c_int,
            color_type: MenuColorType: cef_menu_color_type_t::Type,
            color: &mut Color: *mut cef_color_t
        ) -> c_int {
            if let Some(color_rust) = self.0.get_color_at(c(index), color_type) {
                *color = color_rust;
                1
            } else {
                0
            }
        }
        fn set_font_list(
            &self,
            command_id: CommandId: c_int,
            font_list: &CefString: *const cef_string_t
        ) -> c_int {
            self.0.set_font_list(command_id, &String::from(font_list)) as c_int
        }
        fn set_font_list_at(
            &self,
            index: c_int: c_int,
            font_list: &CefString: *const cef_string_t
        ) -> c_int {
            self.0.set_font_list_at(c(index), &String::from(font_list)) as c_int
        }
    }
}
cef_callback_impl!{
    impl for RunContextMenuWrapper: _cef_run_context_menu_callback_t {
        fn cont(
            &self,
            command_id: CommandId: c_int,
            event_flags: EventFlags: cef_event_flags_t
        ) {
            self.0.cont(command_id, event_flags);
        }
        fn cancel(&self) {
            self.0.cancel();
        }
    }
}
