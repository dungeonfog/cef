use crate::{
    color::Color,
    events::EventFlags,
    string::{CefString},
    refcounted::{RefCountedPtr, Wrapper},
    values::Point,
};
use cef_sys::{
    _cef_menu_model_t,
    _cef_menu_model_delegate_t,
    cef_menu_model_create,
    cef_event_flags_t,
    cef_point_t,
    cef_string_t,
};

use std::{
    convert::TryInto,
    os::raw::c_int,
};
use super::{CommandId, GroupId, MenuItemType, Accelerator, MenuColorType};

fn c<A, B>(a: A) -> B
    where A: TryInto<B>,
          A::Error: std::fmt::Debug,
{
    a.try_into().unwrap()
}

ref_counted_ptr!{
    /// Supports creation and modification of menus. See cef_menu_id_t for the
    /// command ids that have default implementations. All user-defined command ids
    /// should be between MENU_ID_USER_FIRST and MENU_ID_USER_LAST. The functions of
    /// this structure can only be accessed on the browser process the UI thread.
    pub struct MenuModel(*mut _cef_menu_model_t);
}

ref_counted_ptr!{
    /// Instantiate this structure to handle menu model events.
    pub struct MenuModelDelegate(*mut _cef_menu_model_delegate_t);
}

impl MenuModel {
    pub fn new(delegate: MenuModelDelegate) -> MenuModel {
        unsafe {
            MenuModel::from_ptr_unchecked(cef_menu_model_create(delegate.into_raw()))
        }
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

impl MenuModelDelegate {
    pub fn new<C: MenuModelDelegateCallbacks>(callbacks: C) -> MenuModelDelegate {
        unsafe{ MenuModelDelegate::from_ptr_unchecked(MenuModelDelegateWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }
}

/// Implement this trait to handle menu model events.
pub trait MenuModelDelegateCallbacks: 'static + Send + Sync {
    /// Perform the action associated with the specified |command_id| and optional
    /// |event_flags|.
    fn execute_command(
        &self,
        menu_model: MenuModel,
        command_id: CommandId,
        event_flags: EventFlags,
    );
    /// Called when the user moves the mouse outside the menu and over the owning
    /// window.
    fn mouse_outside_menu(
        &self,
        _menu_model: MenuModel,
        _point: Point,
    ) {
    }
    /// Called on unhandled open submenu keyboard commands. |is_rtl| will be `true`
    /// if the menu is displaying a right-to-left language.
    fn unhandled_open_submenu(
        &self,
        _menu_model: MenuModel,
        _is_rtl: bool,
    ) {
    }
    /// Called on unhandled close submenu keyboard commands. |is_rtl| will be `true`
    /// if the menu is displaying a right-to-left language.
    fn unhandled_close_submenu(
        &self,
        _menu_model: MenuModel,
        _is_rtl: bool,
    ) {
    }
    /// The menu is about to show.
    fn menu_will_show(
        &self,
        _menu_model: MenuModel,
    ) {
    }
    /// The menu has closed.
    fn menu_closed(
        &self,
        _menu_model: MenuModel,
    ) {
    }
    /// Optionally modify a menu item label. Return `true` if `label` was
    /// modified.
    fn format_label(
        &self,
        _menu_model: MenuModel,
        _label: &mut String,
    ) -> bool {
        false
    }
}

struct MenuModelDelegateWrapper(Box<dyn MenuModelDelegateCallbacks>);
impl Wrapper for MenuModelDelegateWrapper {
    type Cef = _cef_menu_model_delegate_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            _cef_menu_model_delegate_t {
                base: unsafe { std::mem::zeroed() },
                execute_command: Some(Self::execute_command),
                mouse_outside_menu: Some(Self::mouse_outside_menu),
                unhandled_open_submenu: Some(Self::unhandled_open_submenu),
                unhandled_close_submenu: Some(Self::unhandled_close_submenu),
                menu_will_show: Some(Self::menu_will_show),
                menu_closed: Some(Self::menu_closed),
                format_label: Some(Self::format_label),
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
}


cef_callback_impl!{
    impl for MenuModelDelegateWrapper: _cef_menu_model_delegate_t {
        fn execute_command(
            &self,
            menu_model: MenuModel: *mut _cef_menu_model_t,
            command_id: CommandId: c_int,
            event_flags: EventFlags: cef_event_flags_t,
        ) {
            self.0.execute_command(menu_model, command_id, event_flags);
        }
        fn mouse_outside_menu(
            &self,
            menu_model: MenuModel: *mut _cef_menu_model_t,
            point: &Point: *const cef_point_t
        ) {
            self.0.mouse_outside_menu(menu_model, *point);
        }
        fn unhandled_open_submenu(
            &self,
            menu_model: MenuModel: *mut _cef_menu_model_t,
            is_rtl: c_int: c_int,
        ) {
            self.0.unhandled_open_submenu(menu_model, is_rtl != 0);
        }
        fn unhandled_close_submenu(
            &self,
            menu_model: MenuModel: *mut _cef_menu_model_t,
            is_rtl: c_int: c_int,
        ) {
            self.0.unhandled_close_submenu(menu_model, is_rtl != 0);
        }
        fn menu_will_show(
            &self,
            menu_model: MenuModel: *mut _cef_menu_model_t,
        ) {
            self.0.menu_will_show(menu_model);
        }
        fn menu_closed(
            &self,
            menu_model: MenuModel: *mut _cef_menu_model_t,
        ) {
            self.0.menu_closed(menu_model);
        }
        fn format_label(
            &self,
            menu_model: MenuModel: *mut _cef_menu_model_t,
            label: &mut CefString: *mut cef_string_t,
        ) -> c_int {
            let mut string = String::from(&*label);
            if self.0.format_label(menu_model, &mut string) {
                *label = CefString::new(&string);
                1
            } else {
                0
            }
        }
    }
}
