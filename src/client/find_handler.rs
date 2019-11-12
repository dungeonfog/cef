use crate::values::Rect;
use cef_sys::cef_rect_t;
use crate::{
    browser::{Browser},
    refcounted::{RefCountedPtr, Wrapper},
};
use cef_sys::{
    cef_browser_t,
    cef_find_handler_t,
};
use std::os::raw::{c_int};
use parking_lot::Mutex;

ref_counted_ptr!{
    /// Instantiate this structure to handle events related to find results. The
    /// functions of this structure will be called on the UI thread.
    pub struct FindHandler(*mut cef_find_handler_t);
}

impl FindHandler {
    pub fn new<C: FindHandlerCallbacks>(callbacks: C) -> FindHandler {
        unsafe{ FindHandler::from_ptr_unchecked(FindHandlerWrapper(Mutex::new(Box::new(callbacks))).wrap().into_raw()) }
    }
}

/// Implement this trait to handle events related to find results. The
/// functions of this structure will be called on the UI thread.
pub trait FindHandlerCallbacks: 'static + Send {
    // Called to report find results returned by cef_browser_host_t::find().
    // `identifer` is the identifier passed to find(), `count` is the number of
    // matches currently identified, `selectionRect` is the location of where the
    // match was found (in window coordinates), `activeMatchOrdinal` is the
    // current position in the search results, and `finalUpdate` is `true` if
    // this is the last find notification.
    fn on_find_result(
        &mut self,
        browser: Browser,
        identifier: i32,
        count: usize,
        selection_rect: Rect,
        active_match_ordinal: usize,
        final_update: bool
    );
}

struct FindHandlerWrapper(Mutex<Box<dyn FindHandlerCallbacks>>);

impl Wrapper for FindHandlerWrapper {
    type Cef = cef_find_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_find_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_find_result: Some(Self::on_find_result),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for FindHandlerWrapper: cef_find_handler_t {
        fn on_find_result(
            &self,
            browser: Browser: *mut cef_browser_t,
            identifier: c_int: c_int,
            count: c_int: c_int,
            selection_rect: &Rect: *const cef_rect_t,
            active_match_ordinal: c_int: c_int,
            final_update: c_int: c_int
        ) {
            self.0.lock().on_find_result(
                browser,
                identifier,
                count as usize,
                *selection_rect,
                active_match_ordinal as usize,
                final_update != 0,
            )
        }
    }
}
