use cef_sys::{
    cef_event_flags_t, cef_key_event_t, cef_key_event_type_t, cef_mouse_button_type_t,
    cef_pointer_type_t, cef_touch_event_type_t,
};
use num_enum::UnsafeFromPrimitive;

/// Key event types.
#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum KeyEventType {
    /// Notification that a key transitioned from "up" to "down".
    RawKeyDown = cef_key_event_type_t::KEYEVENT_RAWKEYDOWN as i32,
    /// Notification that a key was pressed. This does not necessarily correspond
    /// to a character depending on the key and language. Use [KeyEventType::Char] for
    /// character input.
    KeyDown = cef_key_event_type_t::KEYEVENT_KEYDOWN as i32,
    /// Notification that a key was released.
    KeyUp = cef_key_event_type_t::KEYEVENT_KEYUP as i32,
    /// Notification that a character was typed. Use this for text input. Key
    /// down events may generate 0, 1, or more than one character event depending
    /// on the key, locale, and operating system.
    Char = cef_key_event_type_t::KEYEVENT_CHAR as i32,
}

/// Supported event bit flags.
#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum EventFlags {
    CapsLockOn = cef_event_flags_t::EVENTFLAG_CAPS_LOCK_ON.0,
    ShiftDown = cef_event_flags_t::EVENTFLAG_SHIFT_DOWN.0,
    ControlDown = cef_event_flags_t::EVENTFLAG_CONTROL_DOWN.0,
    AltDown = cef_event_flags_t::EVENTFLAG_ALT_DOWN.0,
    LeftMouseButton = cef_event_flags_t::EVENTFLAG_LEFT_MOUSE_BUTTON.0,
    MiddleMouseButton = cef_event_flags_t::EVENTFLAG_MIDDLE_MOUSE_BUTTON.0,
    RightMouseButton = cef_event_flags_t::EVENTFLAG_RIGHT_MOUSE_BUTTON.0,
    CommandDown = cef_event_flags_t::EVENTFLAG_COMMAND_DOWN.0,
    NumLockOn = cef_event_flags_t::EVENTFLAG_NUM_LOCK_ON.0,
    IsKeyPad = cef_event_flags_t::EVENTFLAG_IS_KEY_PAD.0,
    IsLeft = cef_event_flags_t::EVENTFLAG_IS_LEFT.0,
    IsRight = cef_event_flags_t::EVENTFLAG_IS_RIGHT.0,
}

pub struct KeyEvent(cef_key_event_t);

impl KeyEvent {
    pub fn new() -> Self {
        Self(unsafe { std::mem::zeroed() })
    }

    /// Set the type of keyboard event.
    pub fn set_event_type(&mut self, event_type: KeyEventType) {
        self.0.type_ = unsafe { std::mem::transmute(event_type) };
    }
    /// The type of keyboard event.
    pub fn event_type(&self) -> KeyEventType {
        unsafe { KeyEventType::from_unchecked(self.0.type_ as i32) }
    }

    /// Set bit flags describing any pressed modifier keys.
    pub fn set_modifiers(&mut self, modifiers: &[EventFlags]) {
        self.0.modifiers = modifiers
            .iter()
            .fold(0, |flags, flag| flags | (*flag as i32 as u32));
    }
    /// Bit flags describing any pressed modifier keys.
    pub fn modifiers(&self) -> Vec<EventFlags> {
        [
            EventFlags::CapsLockOn,
            EventFlags::ShiftDown,
            EventFlags::ControlDown,
            EventFlags::AltDown,
            EventFlags::LeftMouseButton,
            EventFlags::MiddleMouseButton,
            EventFlags::RightMouseButton,
            EventFlags::CommandDown,
            EventFlags::NumLockOn,
            EventFlags::IsKeyPad,
            EventFlags::IsLeft,
            EventFlags::IsRight,
        ]
        .iter()
        .filter(|flag| ((**flag) as u32 & self.0.modifiers) != 0)
        .cloned()
        .collect()
    }

    /// Set the Windows key code for the key event. This value is used by the DOM
    /// specification. Sometimes it comes directly from the event (i.e. on
    /// Windows) and sometimes it's determined using a mapping function. See
    /// WebCore/platform/chromium/KeyboardCodes.h for the list of values.
    pub fn set_windows_key_code(&mut self, code: i32) {
        self.0.windows_key_code = code;
    }
    /// The Windows key code for the key event. This value is used by the DOM
    /// specification. Sometimes it comes directly from the event (i.e. on
    /// Windows) and sometimes it's determined using a mapping function. See
    /// WebCore/platform/chromium/KeyboardCodes.h for the list of values.
    pub fn windows_key_code(&self) -> i32 {
        self.0.windows_key_code
    }

    /// Set the actual key code genenerated by the platform.
    pub fn set_native_key_code(&mut self, code: i32) {
        self.0.native_key_code = code;
    }
    /// The actual key code genenerated by the platform.
    pub fn native_key_code(&self) -> i32 {
        self.0.native_key_code
    }

    /// Set to indicate whether the event is considered a "system key" event (see
    /// http://msdn.microsoft.com/en-us/library/ms646286(VS.85).aspx for details).
    /// This value should always be false on non-Windows platforms.
    pub fn set_system_key(&mut self, flag: bool) {
        self.0.is_system_key = flag as i32;
    }
    /// Indicates whether the event is considered a "system key" event (see
    /// http://msdn.microsoft.com/en-us/library/ms646286(VS.85).aspx for details).
    /// This value will always be false on non-Windows platforms.
    pub fn is_system_key(&self) -> bool {
        self.0.is_system_key != 0
    }

    /// Set the character generated by the keystroke.
    pub fn set_character(&mut self, character: u16) {
        self.0.character = character;
    }
    /// The character generated by the keystroke.
    pub fn character(&self) -> u16 {
        self.0.character
    }

    /// Same as [KeyEvent::set_character] but unmodified by any concurrently-held modifiers
    /// (except shift). This is useful for working out shortcut keys.
    pub fn set_unmodified_character(&mut self, character: u16) {
        self.0.unmodified_character = character;
    }
    /// Same as |character| but unmodified by any concurrently-held modifiers
    /// (except shift). This is useful for working out shortcut keys.
    pub fn unmodified_character(&self) -> u16 {
        self.0.unmodified_character
    }
    /// Set to true if the focus is currently on an editable field on the page. This is
    /// useful for determining if standard key events should be intercepted.
    pub fn set_focus_on_editable_field(&mut self, flag: bool) {
        self.0.focus_on_editable_field = flag as i32;
    }
    /// True if the focus is currently on an editable field on the page. This is
    /// useful for determining if standard key events should be intercepted.
    pub fn focus_on_editable_field(&self) -> bool {
        self.0.focus_on_editable_field != 0
    }
}

impl Into<cef_key_event_t> for KeyEvent {
    fn into(self) -> cef_key_event_t {
        self.0
    }
}

/// Mouse button types.
#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum MouseButtonType {
    Left = cef_mouse_button_type_t::MBT_LEFT,
    Middle = cef_mouse_button_type_t::MBT_MIDDLE,
    Right = cef_mouse_button_type_t::MBT_RIGHT,
}

pub struct MouseEvent();

/// Touch points states types.
#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum TouchEventType {
    Released = cef_touch_event_type_t::CEF_TET_RELEASED,
    Pressed = cef_touch_event_type_t::CEF_TET_PRESSED,
    Moved = cef_touch_event_type_t::CEF_TET_MOVED,
    Cancelled = cef_touch_event_type_t::CEF_TET_CANCELLED,
}

/// The device type that caused the event.
#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum PointerType {
    Touch = cef_pointer_type_t::CEF_POINTER_TYPE_TOUCH,
    Mouse = cef_pointer_type_t::CEF_POINTER_TYPE_MOUSE,
    Pen = cef_pointer_type_t::CEF_POINTER_TYPE_PEN,
    Eraser = cef_pointer_type_t::CEF_POINTER_TYPE_ERASER,
    Unknown = cef_pointer_type_t::CEF_POINTER_TYPE_UNKNOWN,
}

/// Structure representing touch event information.
pub struct TouchEvent {
    /// Id of a touch point. Must be unique per touch, can be any number except -1.
    /// Note that a maximum of 16 concurrent touches will be tracked; touches
    /// beyond that will be ignored.
    id: i32,
    /// X coordinate relative to the left side of the view.
    x: f32,
    /// Y coordinate relative to the top side of the view.
    y: f32,
    /// X radius in pixels. Set to 0 if not applicable.
    radius_x: f32,
    /// Y radius in pixels. Set to 0 if not applicable.
    radius_y: f32,
    /// Rotation angle in radians. Set to 0 if not applicable.
    rotation_angle: f32,
    /// The normalized pressure of the pointer input in the range of [0,1].
    /// Set to 0 if not applicable.
    pressure: f32,
    /// The state of the touch point. Touches begin with one [TouchEventType::Pressed] event
    /// followed by zero or more [TouchEventType::Moved] events and finally one
    /// [TouchEventType::Released] or [TouchEventType::Cancelled] event. Events not respecting this
    /// order will be ignored.
    event_type: TouchEventType,
    /// Bit flags describing any pressed modifier keys. See
    /// [EventFlags] for values.
    modifiers: Vec<EventFlags>,
    /// The device type that caused the event.
    pointer_type: PointerType,
}
