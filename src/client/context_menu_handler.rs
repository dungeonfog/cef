use crate::{
    browser::{Browser},
    events::EventFlags,
    frame::Frame,
    refcounted::{RefCountedPtr, Wrapper},
    string::CefStringList,
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
};
use std::{
    os::raw::c_int,
    mem::ManuallyDrop,
};

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

pub trait ContextMenuHandlerCallbacks: 'static + Send + Sync { }
pub trait ContextMenuParamsCallbacks: 'static + Send + Sync { }
pub trait MenuModelCallbacks: 'static + Send + Sync { }
pub trait RunContextMenuCallbacks: 'static + Send + Sync { }

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
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
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
            command_id: c_int,
            label: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn add_check_item(
            &self,
            command_id: c_int,
            label: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn add_radio_item(
            &self,
            command_id: c_int,
            label: *const cef_string_t,
            group_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn add_sub_menu(
            &self,
            command_id: c_int,
            label: *const cef_string_t
        ) -> *mut _cef_menu_model_t {
            unimplemented!()
        }
        fn insert_separator_at(
            &self,
            index: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn insert_item_at(
            &self,
            index: c_int,
            command_id: c_int,
            label: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn insert_check_item_at(
            &self,
            index: c_int,
            command_id: c_int,
            label: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn insert_radio_item_at(
            &self,
            index: c_int,
            command_id: c_int,
            label: *const cef_string_t,
            group_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn insert_sub_menu_at(
            &self,
            index: c_int,
            command_id: c_int,
            label: *const cef_string_t
        ) -> *mut _cef_menu_model_t {
            unimplemented!()
        }
        fn remove(
            &self,
            command_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn remove_at(
            &self,
            index: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_index_of(
            &self,
            command_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_command_id_at(
            &self,
            index: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_command_id_at(
            &self,
            index: c_int,
            command_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_label(
            &self,
            command_id: c_int
        ) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn get_label_at(
            &self,
            index: c_int
        ) -> cef_string_userfree_t {
            unimplemented!()
        }
        fn set_label(
            &self,
            command_id: c_int,
            label: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn set_label_at(
            &self,
            index: c_int,
            label: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn get_type(
            &self,
            command_id: c_int
        ) -> Type {
            unimplemented!()
        }
        fn get_type_at(
            &self,
            index: c_int
        ) -> Type {
            unimplemented!()
        }
        fn get_group_id(
            &self,
            command_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_group_id_at(
            &self,
            index: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_group_id(
            &self,
            command_id: c_int,
            group_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_group_id_at(
            &self,
            index: c_int,
            group_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_sub_menu(
            &self,
            command_id: c_int
        ) -> *mut _cef_menu_model_t {
            unimplemented!()
        }
        fn get_sub_menu_at(
            &self,
            index: c_int
        ) -> *mut _cef_menu_model_t {
            unimplemented!()
        }
        fn is_visible(
            &self,
            command_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn is_visible_at(
            &self,
            index: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_visible(
            &self,
            command_id: c_int,
            visible: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_visible_at(
            &self,
            index: c_int,
            visible: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn is_enabled(
            &self,
            command_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn is_enabled_at(
            &self,
            index: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_enabled(
            &self,
            command_id: c_int,
            enabled: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_enabled_at(
            &self,
            index: c_int,
            enabled: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn is_checked(
            &self,
            command_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn is_checked_at(
            &self,
            index: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_checked(
            &self,
            command_id: c_int,
            checked: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_checked_at(
            &self,
            index: c_int,
            checked: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn has_accelerator(
            &self,
            command_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn has_accelerator_at(
            &self,
            index: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_accelerator(
            &self,
            command_id: c_int,
            key_code: c_int,
            shift_pressed: c_int,
            ctrl_pressed: c_int,
            alt_pressed: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_accelerator_at(
            &self,
            index: c_int,
            key_code: c_int,
            shift_pressed: c_int,
            ctrl_pressed: c_int,
            alt_pressed: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn remove_accelerator(
            &self,
            command_id: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn remove_accelerator_at(
            &self,
            index: c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_accelerator(
            &self,
            command_id: c_int,
            key_code: *mut c_int,
            shift_pressed: *mut c_int,
            ctrl_pressed: *mut c_int,
            alt_pressed: *mut c_int
        ) -> c_int {
            unimplemented!()
        }
        fn get_accelerator_at(
            &self,
            index: c_int,
            key_code: *mut c_int,
            shift_pressed: *mut c_int,
            ctrl_pressed: *mut c_int,
            alt_pressed: *mut c_int
        ) -> c_int {
            unimplemented!()
        }
        fn set_color(
            &self,
            command_id: c_int,
            color_type: Type,
            color: cef_color_t
        ) -> c_int {
            unimplemented!()
        }
        fn set_color_at(
            &self,
            index: c_int,
            color_type: Type,
            color: cef_color_t
        ) -> c_int {
            unimplemented!()
        }
        fn get_color(
            &self,
            command_id: c_int,
            color_type: Type,
            color: *mut cef_color_t
        ) -> c_int {
            unimplemented!()
        }
        fn get_color_at(
            &self,
            index: c_int,
            color_type: Type,
            color: *mut cef_color_t
        ) -> c_int {
            unimplemented!()
        }
        fn set_font_list(
            &self,
            command_id: c_int,
            font_list: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
        fn set_font_list_at(
            &self,
            index: c_int,
            font_list: *const cef_string_t
        ) -> c_int {
            unimplemented!()
        }
    }
}
