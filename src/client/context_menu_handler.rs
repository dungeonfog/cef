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
            pub unsafe fn from_raw_unchecked(raw: i32) -> $Id {
                $Id(raw)
            }
        }
    };
}

id!(pub struct CommandId);
id!(pub struct GroupId);

/// Supported color types for menu items.
#[repr(u32)]
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
#[repr(u32)]
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
    /// Supported context menu type flags.
    pub struct ContextMenuTypeFlags: u32 {
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
    pub struct ContextMenuMediaStateFlags: u32 {
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
    pub struct ContextMenuEditStateFlags: u32 {
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
    pub struct ContextMenuHandler(*mut cef_context_menu_handler_t);
}

impl ContextMenuHandler {
    pub fn new<C: ContextMenuHandlerCallbacks>(callbacks: C) -> ContextMenuHandler {
        unsafe{ ContextMenuHandler::from_ptr_unchecked(ContextMenuHandlerWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }
}

ref_counted_ptr!{
    pub struct ContextMenuParams(*mut _cef_context_menu_params_t);
}

impl ContextMenuParams {
    pub fn new<C: ContextMenuParamsCallbacks>(callbacks: C) -> ContextMenuParams {
        unsafe{ ContextMenuParams::from_ptr_unchecked(ContextMenuParamsWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }
}

ref_counted_ptr!{
    pub struct MenuModel(*mut _cef_menu_model_t);
}

impl MenuModel {
    pub fn new<C: MenuModelCallbacks>(callbacks: C) -> MenuModel {
        unsafe{ MenuModel::from_ptr_unchecked(MenuModelWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }
}

ref_counted_ptr!{
    pub struct RunContextMenu(*mut _cef_run_context_menu_callback_t);
}

impl RunContextMenu {
    pub fn new<C: RunContextMenuCallbacks>(callbacks: C) -> RunContextMenu {
        unsafe{ RunContextMenu::from_ptr_unchecked(RunContextMenuWrapper(Box::new(callbacks)).wrap().into_raw()) }
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

pub trait ContextMenuHandlerCallbacks: 'static + Send + Sync {
    fn on_before_context_menu(
        &self,
        _browser: Browser,
        _frame: Frame,
        _params: ContextMenuParams,
        _model: MenuModel
    ) {
    }
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
    fn on_context_menu_dismissed(
        &self,
        _browser: Browser,
        _frame: Frame
    ) {
    }
}
pub trait ContextMenuParamsCallbacks: 'static + Send + Sync {
    fn get_xcoord(&self) -> i32;
    fn get_ycoord(&self) -> i32;
    fn get_type_flags(&self) -> ContextMenuTypeFlags;
    fn get_link_url(&self) -> String;
    fn get_unfiltered_link_url(&self) -> String;
    fn get_source_url(&self) -> String;
    fn has_image_contents(&self) -> bool;
    fn get_title_text(&self) -> String;
    fn get_page_url(&self) -> String;
    fn get_frame_url(&self) -> String;
    fn get_frame_charset(&self) -> String;
    fn get_media_type(&self) -> ContextMenuTypeFlags;
    fn get_media_state_flags(&self) -> ContextMenuMediaStateFlags;
    fn get_selection_text(&self) -> String;
    fn get_misspelled_word(&self) -> String;
    fn get_dictionary_suggestions(
        &self,
        suggestions: &mut Vec<String>,
    ) -> bool;
    fn is_editable(&self) -> bool;
    fn is_spell_check_enabled(&self) -> bool;
    fn get_edit_state_flags(&self) -> ContextMenuEditStateFlags;
    fn is_custom_menu(&self) -> bool;
    fn is_pepper_menu(&self) -> bool;
}
pub trait MenuModelCallbacks: 'static + Send + Sync {
    fn is_sub_menu(&self) -> bool;
    fn clear(&self) -> bool;
    fn get_count(&self) -> usize;
    fn add_separator(&self) -> bool;
    fn add_item(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> bool;
    fn add_check_item(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> bool;
    fn add_radio_item(
        &self,
        command_id: CommandId,
        label: &str,
        group_id: GroupId,
    ) -> bool;
    fn add_sub_menu(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> MenuModel;
    fn insert_separator_at(
        &self,
        index: usize,
    ) -> bool;
    fn insert_item_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
    ) -> bool;
    fn insert_check_item_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
    ) -> bool;
    fn insert_radio_item_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
        group_id: GroupId,
    ) -> bool;
    fn insert_sub_menu_at(
        &self,
        index: usize,
        command_id: CommandId,
        label: &str,
    ) -> *mut _cef_menu_model_t;
    fn remove(
        &self,
        command_id: CommandId,
    ) -> bool;
    fn remove_at(
        &self,
        index: usize,
    ) -> bool;
    fn get_index_of(
        &self,
        command_id: CommandId,
    ) -> Option<usize>;
    fn get_command_id_at(
        &self,
        index: usize,
    ) -> Option<usize>;
    fn set_command_id_at(
        &self,
        index: usize,
        command_id: CommandId,
    ) -> bool;
    fn get_label(
        &self,
        command_id: CommandId,
    ) -> String;
    fn get_label_at(
        &self,
        index: usize,
    ) -> String;
    fn set_label(
        &self,
        command_id: CommandId,
        label: &str,
    ) -> bool;
    fn set_label_at(
        &self,
        index: usize,
        label: &str,
    ) -> bool;
    fn get_type(
        &self,
        command_id: CommandId,
    ) -> MenuItemType;
    fn get_type_at(
        &self,
        index: usize,
    ) -> MenuItemType;
    fn get_group_id(
        &self,
        command_id: CommandId,
    ) -> Option<u32>;
    fn get_group_id_at(
        &self,
        index: usize,
    ) -> Option<u32>;
    fn set_group_id(
        &self,
        command_id: CommandId,
        group_id: GroupId,
    ) -> bool;
    fn set_group_id_at(
        &self,
        index: usize,
        group_id: GroupId,
    ) -> bool;
    fn get_sub_menu(
        &self,
        command_id: CommandId,
    ) -> MenuModel;
    fn get_sub_menu_at(
        &self,
        index: usize,
    ) -> MenuModel;
    fn is_visible(
        &self,
        command_id: CommandId,
    ) -> bool;
    fn is_visible_at(
        &self,
        index: usize,
    ) -> bool;
    fn set_visible(
        &self,
        command_id: CommandId,
        visible: bool,
    ) -> bool;
    fn set_visible_at(
        &self,
        index: usize,
        visible: bool,
    ) -> bool;
    fn is_enabled(
        &self,
        command_id: CommandId,
    ) -> bool;
    fn is_enabled_at(
        &self,
        index: usize,
    ) -> bool;
    fn set_enabled(
        &self,
        command_id: CommandId,
        enabled: bool,
    ) -> bool;
    fn set_enabled_at(
        &self,
        index: usize,
        enabled: bool,
    ) -> bool;
    fn is_checked(
        &self,
        command_id: CommandId,
    ) -> bool;
    fn is_checked_at(
        &self,
        index: usize,
    ) -> bool;
    fn set_checked(
        &self,
        command_id: CommandId,
        checked: bool,
    ) -> bool;
    fn set_checked_at(
        &self,
        index: usize,
        checked: bool,
    ) -> bool;
    fn has_accelerator(
        &self,
        command_id: CommandId,
    ) -> bool;
    fn has_accelerator_at(
        &self,
        index: usize,
    ) -> bool;
    fn set_accelerator(
        &self,
        command_id: CommandId,
        key_code: bool,
        shift_pressed: bool,
        ctrl_pressed: bool,
        alt_pressed: bool,
    ) -> bool;
    fn set_accelerator_at(
        &self,
        index: usize,
        key_code: bool,
        shift_pressed: bool,
        ctrl_pressed: bool,
        alt_pressed: bool,
    ) -> bool;
    fn remove_accelerator(
        &self,
        command_id: CommandId,
    ) -> bool;
    fn remove_accelerator_at(
        &self,
        index: usize,
    ) -> bool;
    fn get_accelerator(
        &self,
        command_id: CommandId,
        key_code: &mut bool,
        shift_pressed: &mut bool,
        ctrl_pressed: &mut bool,
        alt_pressed: &mut bool,
    ) -> bool;
    fn get_accelerator_at(
        &self,
        index: usize,
        key_code: &mut bool,
        shift_pressed: &mut bool,
        ctrl_pressed: &mut bool,
        alt_pressed: &mut bool,
    ) -> bool;
    fn set_color(
        &self,
        command_id: CommandId,
        color_type: MenuColorType,
        color: Color,
    ) -> bool;
    fn set_color_at(
        &self,
        index: usize,
        color_type: MenuColorType,
        color: Color,
    ) -> bool;
    fn get_color(
        &self,
        command_id: CommandId,
        color_type: MenuColorType,
        color: &mut Color,
    ) -> bool;
    fn get_color_at(
        &self,
        index: usize,
        color_type: MenuColorType,
        color: &mut Color,
    ) -> bool;
    fn set_font_list(
        &self,
        command_id: CommandId,
        font_list: &str,
    ) -> bool;
    fn set_font_list_at(
        &self,
        index: usize,
        font_list: &str,
    ) -> bool;
}
pub trait RunContextMenuCallbacks: 'static + Send + Sync {
    fn cont(
        &self,
        command_id: CommandId,
        event_flags: EventFlags,
    );
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
            unimplemented!()
        }
        fn run_context_menu(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t,
            params: ContextMenuParams: *mut _cef_context_menu_params_t,
            model: MenuModel: *mut _cef_menu_model_t,
            callback: RunContextMenu: *mut _cef_run_context_menu_callback_t
        ) -> c_int {
            unimplemented!()
        }
        fn on_context_menu_command(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t,
            params: ContextMenuParams: *mut _cef_context_menu_params_t,
            command_id: c_int: c_int,
            event_flags: EventFlags: cef_event_flags_t
        ) -> c_int {
            unimplemented!()
        }
        fn on_context_menu_dismissed(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t
        ) {
            unimplemented!()
        }
    }
}
cef_callback_impl!{
    impl for ContextMenuParamsWrapper: _cef_context_menu_params_t {
        fn get_xcoord(&self) -> c_int {
            unimplemented!()
        }
        fn get_ycoord(&self) -> c_int {
            unimplemented!()
        }
        fn get_type_flags(&self) -> cef_context_menu_type_flags_t::Type {
            unimplemented!()
        }
        fn get_link_url(&self) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn get_unfiltered_link_url(&self) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn get_source_url(&self) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn has_image_contents(&self) -> c_int {
            unimplemented!()
        }
        fn get_title_text(&self) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn get_page_url(&self) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn get_frame_url(&self) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn get_frame_charset(&self) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn get_media_type(&self) -> cef_context_menu_media_type_t::Type {
            unimplemented!()
        }
        fn get_media_state_flags(&self) -> cef_context_menu_media_state_flags_t::Type {
            unimplemented!()
        }
        fn get_selection_text(&self) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn get_misspelled_word(&self) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn get_dictionary_suggestions(
            &self,
            suggestions: ManuallyDrop<CefStringList>: cef_string_list_t
        ) -> c_int {
            unimplemented!()
        }
        fn is_editable(&self) -> c_int {
            unimplemented!()
        }
        fn is_spell_check_enabled(&self) -> c_int {
            unimplemented!()
        }
        fn get_edit_state_flags(&self) -> cef_context_menu_edit_state_flags_t::Type {
            unimplemented!()
        }
        fn is_custom_menu(&self) -> c_int {
            unimplemented!()
        }
        fn is_pepper_menu(&self) -> c_int {
            unimplemented!()
        }
    }
}
cef_callback_impl!{
    impl for MenuModelWrapper: _cef_menu_model_t {
        fn is_sub_menu(&self) -> c_int {
            unimplemented!()
        }
        fn clear(&self) -> c_int {
            unimplemented!()
        }
        fn get_count(&self) -> c_int {
            unimplemented!()
        }
        fn add_separator(&self) -> c_int {
            unimplemented!()
        }
        fn add_item(
            &self,
            command_id: c_int: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn add_check_item(
            &self,
            command_id: c_int: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn add_radio_item(
            &self,
            command_id: c_int: c_int,
            label: &CefString: *const cef_string_t,
            group_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn add_sub_menu(
            &self,
            command_id: c_int: c_int,
            label: &CefString: *const cef_string_t
        ) -> *mut _cef_menu_model_t {
            unimplemented!()
        }
        fn insert_separator_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn insert_item_at(
            &self,
            index: c_int: c_int,
            command_id: c_int: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn insert_check_item_at(
            &self,
            index: c_int: c_int,
            command_id: c_int: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn insert_radio_item_at(
            &self,
            index: c_int: c_int,
            command_id: c_int: c_int,
            label: &CefString: *const cef_string_t,
            group_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn insert_sub_menu_at(
            &self,
            index: c_int: c_int,
            command_id: c_int: c_int,
            label: &CefString: *const cef_string_t
        ) -> *mut _cef_menu_model_t {
            unimplemented!()
        }
        fn remove(
            &self,
            command_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn remove_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_index_of(
            &self,
            command_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_command_id_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_command_id_at(
            &self,
            index: c_int: c_int,
            command_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_label(
            &self,
            command_id: c_int: c_int
        ) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn get_label_at(
            &self,
            index: c_int: c_int
        ) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn set_label(
            &self,
            command_id: c_int: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn set_label_at(
            &self,
            index: c_int: c_int,
            label: &CefString: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn get_type(
            &self,
            command_id: c_int: c_int
        ) -> cef_menu_item_type_t::Type {
            unimplemented!()
        }
        fn get_type_at(
            &self,
            index: c_int: c_int
        ) -> cef_menu_item_type_t::Type {
            unimplemented!()
        }
        fn get_group_id(
            &self,
            command_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_group_id_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_group_id(
            &self,
            command_id: c_int: c_int,
            group_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_group_id_at(
            &self,
            index: c_int: c_int,
            group_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_sub_menu(
            &self,
            command_id: c_int: c_int
        ) -> *mut _cef_menu_model_t {
            unimplemented!()
        }
        fn get_sub_menu_at(
            &self,
            index: c_int: c_int
        ) -> *mut _cef_menu_model_t {
            unimplemented!()
        }
        fn is_visible(
            &self,
            command_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn is_visible_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_visible(
            &self,
            command_id: c_int: c_int,
            visible: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_visible_at(
            &self,
            index: c_int: c_int,
            visible: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn is_enabled(
            &self,
            command_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn is_enabled_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_enabled(
            &self,
            command_id: c_int: c_int,
            enabled: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_enabled_at(
            &self,
            index: c_int: c_int,
            enabled: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn is_checked(
            &self,
            command_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn is_checked_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_checked(
            &self,
            command_id: c_int: c_int,
            checked: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_checked_at(
            &self,
            index: c_int: c_int,
            checked: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn has_accelerator(
            &self,
            command_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn has_accelerator_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_accelerator(
            &self,
            command_id: c_int: c_int,
            key_code: c_int: c_int,
            shift_pressed: c_int: c_int,
            ctrl_pressed: c_int: c_int,
            alt_pressed: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_accelerator_at(
            &self,
            index: c_int: c_int,
            key_code: c_int: c_int,
            shift_pressed: c_int: c_int,
            ctrl_pressed: c_int: c_int,
            alt_pressed: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn remove_accelerator(
            &self,
            command_id: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn remove_accelerator_at(
            &self,
            index: c_int: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_accelerator(
            &self,
            command_id: c_int: c_int,
            key_code: &mut c_int: *mut c_int,
            shift_pressed: &mut c_int: *mut c_int,
            ctrl_pressed: &mut c_int: *mut c_int,
            alt_pressed: &mut c_int: *mut c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_accelerator_at(
            &self,
            index: c_int: c_int,
            key_code: &mut c_int: *mut c_int,
            shift_pressed: &mut c_int: *mut c_int,
            ctrl_pressed: &mut c_int: *mut c_int,
            alt_pressed: &mut c_int: *mut c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_color(
            &self,
            command_id: c_int: c_int,
            color_type: MenuColorType: cef_menu_color_type_t::Type,
            color: Color: cef_color_t
        ) -> c_int {
            unimplemented!()
        }
        fn set_color_at(
            &self,
            index: c_int: c_int,
            color_type: MenuColorType: cef_menu_color_type_t::Type,
            color: Color: cef_color_t
        ) -> c_int {
            unimplemented!()
        }
        fn get_color(
            &self,
            command_id: c_int: c_int,
            color_type: MenuColorType: cef_menu_color_type_t::Type,
            color: &mut Color: *mut cef_color_t
        ) -> c_int {
            unimplemented!()
        }
        fn get_color_at(
            &self,
            index: c_int: c_int,
            color_type: MenuColorType: cef_menu_color_type_t::Type,
            color: &mut Color: *mut cef_color_t
        ) -> c_int {
            unimplemented!()
        }
        fn set_font_list(
            &self,
            command_id: c_int: c_int,
            font_list: &CefString: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn set_font_list_at(
            &self,
            index: c_int: c_int,
            font_list: &CefString: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
    }
}
cef_callback_impl!{
    impl for RunContextMenuWrapper: _cef_run_context_menu_callback_t {
        fn cont(
            &self,
            command_id: c_int: c_int,
            event_flags: EventFlags: cef_event_flags_t
        ) {
            unimplemented!()
        }
        fn cancel(&self) {
            unimplemented!()
        }
    }
}
