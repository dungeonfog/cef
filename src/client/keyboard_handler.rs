use crate::{
    browser::{Browser},
    events::KeyEvent,
    refcounted::{RefCountedPtr, Wrapper},
};
use cef_sys::{
    cef_browser_t,
    cef_keyboard_handler_t,
    cef_key_event_t,
    cef_event_handle_t,
};
use std::os::raw::{c_int};
use parking_lot::Mutex;

ref_counted_ptr!{
    /// Instantiate this structure to handle events related to keyboard input.
    pub struct KeyboardHandler(*mut cef_keyboard_handler_t);
}

impl KeyboardHandler {
    pub fn new<C: KeyboardHandlerCallbacks>(callbacks: C) -> KeyboardHandler {
        unsafe{ KeyboardHandler::from_ptr_unchecked(KeyboardHandlerWrapper(Mutex::new(Box::new(callbacks))).wrap().into_raw()) }
    }
}

/// Implement this trait to handle events related to keyboard input.
pub trait KeyboardHandlerCallbacks: 'static + Send {
    /// Called before a keyboard event is sent to the renderer. |event| contains
    /// information about the keyboard event. |os_event| is the operating system
    /// event message, if any. Return true (1) if the event was handled or false
    /// (0) otherwise. If the event will be handled in on_key_event() as a keyboard
    /// shortcut set |is_keyboard_shortcut| to true (1) and return false (0).
    fn on_pre_key_event(
        &mut self,
        browser: Browser,
        event: KeyEvent,
        is_keyboard_shortcut: &mut bool
    ) -> bool {
        false
    }
    /// Called after the renderer and JavaScript in the page has had a chance to
    /// handle the event. |event| contains information about the keyboard event.
    /// |os_event| is the operating system event message, if any. Return true (1)
    /// if the keyboard event was handled or false (0) otherwise.
    fn on_key_event(
        &mut self,
        browser: Browser,
        event: KeyEvent,
    ) -> bool {
        false
    }
}

struct KeyboardHandlerWrapper(Mutex<Box<dyn KeyboardHandlerCallbacks>>);

impl Wrapper for KeyboardHandlerWrapper {
    type Cef = cef_keyboard_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_keyboard_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_pre_key_event: Some(Self::on_pre_key_event),
                on_key_event: Some(Self::on_key_event),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for KeyboardHandlerWrapper: cef_keyboard_handler_t {
        fn on_pre_key_event(
            &self,
            browser: Browser: *mut cef_browser_t,
            event: KeyEvent: *const cef_key_event_t,
            _os_event: cef_event_handle_t: cef_event_handle_t,
            is_keyboard_shortcut: &mut c_int: *mut c_int
        ) -> c_int {
            let mut keyboard_shortcut = *is_keyboard_shortcut != 0;
            let ret = self.0.lock().on_pre_key_event(browser, event, &mut keyboard_shortcut) as c_int;
            *is_keyboard_shortcut = keyboard_shortcut as c_int;
            ret
        }
        fn on_key_event(
            &self,
            browser: Browser: *mut cef_browser_t,
            event: KeyEvent: *const cef_key_event_t,
            _os_event: cef_event_handle_t: cef_event_handle_t,
        ) -> c_int {
            self.0.lock().on_key_event(browser, event) as c_int
        }
    }
}
