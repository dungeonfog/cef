use crate::{
    browser::{Browser},
    refcounted::{RefCountedPtr, Wrapper},
};
use cef_sys::{
    cef_browser_t,
    cef_focus_handler_t,
    cef_focus_source_t,
};
use std::os::raw::{c_int};
use parking_lot::Mutex;
use num_enum::UnsafeFromPrimitive;

ref_counted_ptr!{
    /// Instantiate this structure to handle events related to focus.
    ///
    /// The functions of this structure will be called on the UI thread.
    pub struct FocusHandler(*mut cef_focus_handler_t);
}

impl FocusHandler {
    pub fn new<C: FocusHandlerCallbacks>(callbacks: C) -> FocusHandler {
        unsafe{ FocusHandler::from_ptr_unchecked(FocusHandlerWrapper(Mutex::new(Box::new(callbacks))).wrap().into_raw()) }
    }
}

/// Focus sources.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum FocusSource {
    /// The source is explicit navigation via the API (LoadURL(), etc).
    Navigation = cef_focus_source_t::FOCUS_SOURCE_NAVIGATION as isize,
    /// The source is a system-generated focus event.
    System = cef_focus_source_t::FOCUS_SOURCE_SYSTEM as isize,
}

/// Implement this trait to handle events related to focus.
///
/// The functions of this trait will be called on the UI thread.
pub trait FocusHandlerCallbacks: 'static + Send {
    /// Called when the browser component is about to lose focus. For instance, if
    /// focus was on the last HTML element and the user pressed the TAB key. `next`
    /// will be `true` if the browser is giving focus to the next component and
    /// `false` if the browser is giving focus to the previous component.
    fn on_take_focus(
        &mut self,
        browser: Browser,
        next: bool,
    ) {
    }
    /// Called when the browser component is requesting focus. `source` indicates
    /// where the focus request is originating from. Return `false` to allow the
    /// focus to be set or `true` to cancel setting the focus.
    fn on_set_focus(
        &mut self,
        browser: Browser,
        source: FocusSource,
    ) -> bool {
        false
    }
    /// Called when the browser component has received focus.
    fn on_got_focus(
        &mut self,
        browser: Browser,
    ) {
    }
}

pub struct FocusHandlerWrapper(Mutex<Box<dyn FocusHandlerCallbacks>>);

impl Wrapper for FocusHandlerWrapper {
    type Cef = cef_focus_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_focus_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_take_focus: Some(Self::on_take_focus),
                on_set_focus: Some(Self::on_set_focus),
                on_got_focus: Some(Self::on_got_focus),
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for FocusHandlerWrapper: cef_focus_handler_t {
        fn on_take_focus(
            &self,
            browser: Browser: *mut cef_browser_t,
            next: c_int: c_int,
        ) {
            self.0.lock().on_take_focus(browser, next != 0);
        }
        fn on_set_focus(
            &self,
            browser: Browser: *mut cef_browser_t,
            source: FocusSource: cef_focus_source_t::Type
        ) -> c_int {
            self.0.lock().on_set_focus(browser, source) as c_int
        }
        fn on_got_focus(
            &self,
            browser: Browser: *mut cef_browser_t
        ) {
            self.0.lock().on_got_focus(browser)
        }
    }
}
