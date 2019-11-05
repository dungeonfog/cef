use cef_sys::{
    cef_event_flags_t, cef_key_event_t, cef_key_event_type_t, cef_mouse_button_type_t,
    cef_mouse_event_t, cef_pointer_type_t, cef_touch_event_t, cef_touch_event_type_t,
};
use num_enum::UnsafeFromPrimitive;
use bitflags::bitflags;
use std::mem;

/// Key event types.
#[repr(u32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum KeyEventType {
    /// Notification that a key transitioned from "up" to "down".
    RawKeyDown = cef_key_event_type_t::KEYEVENT_RAWKEYDOWN,
    /// Notification that a key was pressed. This does not necessarily correspond
    /// to a character depending on the key and language. Use [KeyEventType::Char] for
    /// character input.
    KeyDown = cef_key_event_type_t::KEYEVENT_KEYDOWN,
    /// Notification that a key was released.
    KeyUp = cef_key_event_type_t::KEYEVENT_KEYUP,
    /// Notification that a character was typed. Use this for text input. Key
    /// down events may generate 0, 1, or more than one character event depending
    /// on the key, locale, and operating system.
    Char = cef_key_event_type_t::KEYEVENT_CHAR,
}

bitflags!{
    #[derive(Default)]
    pub struct EventFlags: u32 {
        const CAPS_LOCK_ON = cef_event_flags_t::EVENTFLAG_CAPS_LOCK_ON.0 as _;
        const SHIFT_DOWN = cef_event_flags_t::EVENTFLAG_SHIFT_DOWN.0 as _;
        const CONTROL_DOWN = cef_event_flags_t::EVENTFLAG_CONTROL_DOWN.0 as _;
        const ALT_DOWN = cef_event_flags_t::EVENTFLAG_ALT_DOWN.0 as _;
        const LEFT_MOUSE_BUTTON = cef_event_flags_t::EVENTFLAG_LEFT_MOUSE_BUTTON.0 as _;
        const MIDDLE_MOUSE_BUTTON = cef_event_flags_t::EVENTFLAG_MIDDLE_MOUSE_BUTTON.0 as _;
        const RIGHT_MOUSE_BUTTON = cef_event_flags_t::EVENTFLAG_RIGHT_MOUSE_BUTTON.0 as _;
        const COMMAND_DOWN = cef_event_flags_t::EVENTFLAG_COMMAND_DOWN.0 as _;
        const NUM_LOCK_ON = cef_event_flags_t::EVENTFLAG_NUM_LOCK_ON.0 as _;
        const IS_KEY_PAD = cef_event_flags_t::EVENTFLAG_IS_KEY_PAD.0 as _;
        const IS_LEFT = cef_event_flags_t::EVENTFLAG_IS_LEFT.0 as _;
        const IS_RIGHT = cef_event_flags_t::EVENTFLAG_IS_RIGHT.0 as _;
    }
}

impl EventFlags {
    pub unsafe fn from_unchecked(i: u32) -> EventFlags {
        EventFlags::from_bits_unchecked(i)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub event_type: KeyEventType,
    pub modifiers: EventFlags,
    pub windows_key_code: i32,
    pub native_key_code: i32,
    pub is_system_key: i32,
    pub character: u16,
    pub unmodified_character: u16,
    pub focus_on_editable_field: i32,
}

impl KeyEvent {
    pub fn as_cef(&self) -> &cef_key_event_t {
        unsafe { &*(self as *const Self as *const cef_key_event_t) }
    }
}

impl Into<cef_key_event_t> for KeyEvent {
    fn into(self) -> cef_key_event_t {
        unsafe {
            mem::transmute::<
                KeyEvent,
                cef_key_event_t,
            >(self)
        }
    }
}

/// Mouse button types.
#[repr(u32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum MouseButtonType {
    Left = cef_mouse_button_type_t::MBT_LEFT,
    Middle = cef_mouse_button_type_t::MBT_MIDDLE,
    Right = cef_mouse_button_type_t::MBT_RIGHT,
}

/// Structure representing mouse event information.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    pub x: i32,
    pub y: i32,
    pub modifiers: EventFlags,
}

impl MouseEvent {
    pub fn as_cef(&self) -> &cef_mouse_event_t {
        unsafe { &*(self as *const Self as *const cef_mouse_event_t) }
    }
}

impl Into<cef_mouse_event_t> for MouseEvent {
    fn into(self) -> cef_mouse_event_t {
        unsafe {
            mem::transmute::<
                MouseEvent,
                cef_mouse_event_t,
            >(self)
        }
    }
}

bitflags!{
    #[derive(Default)]
    pub struct TouchEventType: i32 {
        const RELEASED = cef_touch_event_type_t::CEF_TET_RELEASED as _;
        const PRESSED = cef_touch_event_type_t::CEF_TET_PRESSED as _;
        const MOVED = cef_touch_event_type_t::CEF_TET_MOVED as _;
        const CANCELLED = cef_touch_event_type_t::CEF_TET_CANCELLED as _;
    }
}

impl TouchEventType {
    pub unsafe fn from_unchecked(i: i32) -> TouchEventType {
        TouchEventType::from_bits_unchecked(i)
    }
}

bitflags!{
    /// The device type that caused the event.
    #[derive(Default)]
    pub struct PointerType: i32 {
        const TOUCH = cef_pointer_type_t::CEF_POINTER_TYPE_TOUCH as _;
        const MOUSE = cef_pointer_type_t::CEF_POINTER_TYPE_MOUSE as _;
        const PEN = cef_pointer_type_t::CEF_POINTER_TYPE_PEN as _;
        const ERASER = cef_pointer_type_t::CEF_POINTER_TYPE_ERASER as _;
        const UNKNOWN = cef_pointer_type_t::CEF_POINTER_TYPE_UNKNOWN as _;
    }
}

impl PointerType {
    pub unsafe fn from_unchecked(i: i32) -> PointerType {
        PointerType::from_bits_unchecked(i)
    }
}

/// Structure representing touch event information.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct TouchEvent {
    pub touch_id: i32,
    pub x: f32,
    pub y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub rotation_angle: f32,
    pub pressure: f32,
    pub event_type: TouchEventType,
    pub modifiers: EventFlags,
    pub pointer_type: PointerType,
}

impl TouchEvent {
    pub fn as_cef(&self) -> &cef_touch_event_t {
        unsafe { &*(self as *const Self as *const cef_touch_event_t) }
    }
}

impl Into<cef_touch_event_t> for TouchEvent {
    fn into(self) -> cef_touch_event_t {
        unsafe {
            mem::transmute::<
                TouchEvent,
                cef_touch_event_t,
            >(self)
        }
    }
}
