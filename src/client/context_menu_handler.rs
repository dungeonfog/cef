use crate::{
    browser::{Browser},
    events::EventFlags,
    frame::Frame,
    refcounted::{RefCountedPtr, Wrapper},
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
    cef_context_menu_media_state_flags_t,
    cef_context_menu_edit_state_flags_t,
    cef_menu_color_type_t,
    cef_menu_item_type_t,
};
use num_enum::UnsafeFromPrimitive;
use std::{
    os::raw::c_int,
};
use bitflags::bitflags;

mod context_menu_params;
mod menu_model;
mod run_context_menu;
pub use self::{
    context_menu_params::*,
    menu_model::*,
    run_context_menu::*,
};

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
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum MenuColorType {
    Text = cef_menu_color_type_t::CEF_MENU_COLOR_TEXT as isize,
    TextHovered = cef_menu_color_type_t::CEF_MENU_COLOR_TEXT_HOVERED as isize,
    TextAccelerator = cef_menu_color_type_t::CEF_MENU_COLOR_TEXT_ACCELERATOR as isize,
    TextAcceleratorHovered = cef_menu_color_type_t::CEF_MENU_COLOR_TEXT_ACCELERATOR_HOVERED as isize,
    Background = cef_menu_color_type_t::CEF_MENU_COLOR_BACKGROUND as isize,
    BackgroundHovered = cef_menu_color_type_t::CEF_MENU_COLOR_BACKGROUND_HOVERED as isize,
    Count = cef_menu_color_type_t::CEF_MENU_COLOR_COUNT as isize,
}

/// Supported menu item types.
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum MenuItemType {
    None = cef_menu_item_type_t::MENUITEMTYPE_NONE as isize,
    Command = cef_menu_item_type_t::MENUITEMTYPE_COMMAND as isize,
    Check = cef_menu_item_type_t::MENUITEMTYPE_CHECK as isize,
    Radio = cef_menu_item_type_t::MENUITEMTYPE_RADIO as isize,
    Separator = cef_menu_item_type_t::MENUITEMTYPE_SEPARATOR as isize,
    Submenu = cef_menu_item_type_t::MENUITEMTYPE_SUBMENU as isize,
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
        const NONE = cef_context_menu_type_flags_t::CM_TYPEFLAG_NONE.0;
        /// The top page is selected.
        const PAGE = cef_context_menu_type_flags_t::CM_TYPEFLAG_PAGE.0;
        /// A subframe page is selected.
        const FRAME = cef_context_menu_type_flags_t::CM_TYPEFLAG_FRAME.0;
        /// A link is selected.
        const LINK = cef_context_menu_type_flags_t::CM_TYPEFLAG_LINK.0;
        /// A media node is selected.
        const MEDIA = cef_context_menu_type_flags_t::CM_TYPEFLAG_MEDIA.0;
        /// There is a textual or mixed selection that is selected.
        const SELECTION = cef_context_menu_type_flags_t::CM_TYPEFLAG_SELECTION.0;
        /// An editable element is selected.
        const EDITABLE = cef_context_menu_type_flags_t::CM_TYPEFLAG_EDITABLE.0;
    }
}
bitflags!{
    /// Supported context menu media state bit flags.
    pub struct ContextMenuMediaStateFlags: i32 {
        const NONE = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_NONE.0;
        const ERROR = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_ERROR.0;
        const PAUSED = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_PAUSED.0;
        const MUTED = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_MUTED.0;
        const LOOP = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_LOOP.0;
        const CAN_SAVE = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_CAN_SAVE.0;
        const HAS_AUDIO = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_HAS_AUDIO.0;
        const HAS_VIDEO = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_HAS_VIDEO.0;
        const CONTROL_ROOT_ELEMENT = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_CONTROL_ROOT_ELEMENT.0;
        const CAN_PRINT = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_CAN_PRINT.0;
        const CAN_ROTATE = cef_context_menu_media_state_flags_t::CM_MEDIAFLAG_CAN_ROTATE.0;
    }
}
bitflags!{
    /// Supported context menu edit state bit flags.
    pub struct ContextMenuEditStateFlags: i32 {
        const NONE = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_NONE.0;
        const CAN_UNDO = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_UNDO.0;
        const CAN_REDO = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_REDO.0;
        const CAN_CUT = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_CUT.0;
        const CAN_COPY = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_COPY.0;
        const CAN_PASTE = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_PASTE.0;
        const CAN_DELETE = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_DELETE.0;
        const CAN_SELECT_ALL = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_SELECT_ALL.0;
        const CAN_TRANSLATE = cef_context_menu_edit_state_flags_t::CM_EDITFLAG_CAN_TRANSLATE.0;
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
                cef_event_flags_t(event_flags.bits() as _),
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

pub struct ContextMenuHandlerWrapper(Box<dyn ContextMenuHandlerCallbacks>);
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
