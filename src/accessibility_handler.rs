use crate::{
    refcounted::{RefCountedPtr, Wrapper},
    send_cell::SendCell,
    values::{StoredValue, Value},
};
use cef_sys::{
    cef_accessibility_handler_t,
    cef_value_t
};

ref_counted_ptr! {
    /// Implement this structure to receive accessibility notification when
    /// accessibility events have been registered. The functions of this structure
    /// will be called on the UI thread.
    pub struct AccessibilityHandler(*mut cef_accessibility_handler_t);
}

impl AccessibilityHandler {
    pub fn new<C: AccessibilityHandlerCallbacks>(callbacks: C) -> AccessibilityHandler {
        unsafe{ AccessibilityHandler::from_ptr_unchecked(AccessibilityHandlerWrapper::new(Box::new(callbacks)).wrap().into_raw()) }
    }
}

pub trait AccessibilityHandlerCallbacks: 'static + Send {
    /// Called after renderer process sends accessibility tree changes to the
    /// browser process.
    fn on_accessibility_tree_change(&mut self, value: StoredValue) {
    }
    /// Called after renderer process sends accessibility location changes to the
    /// browser process.
    fn on_accessibility_location_change(&mut self, value: StoredValue) {
    }
}

struct AccessibilityHandlerWrapper {
    delegate: SendCell<Box<dyn AccessibilityHandlerCallbacks>>,
}

impl AccessibilityHandlerWrapper {
    fn new(delegate: Box<dyn AccessibilityHandlerCallbacks>) -> AccessibilityHandlerWrapper {
        AccessibilityHandlerWrapper {
            delegate: SendCell::new(delegate)
        }
    }
}

impl Wrapper for AccessibilityHandlerWrapper {
    type Cef = cef_accessibility_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_accessibility_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_accessibility_tree_change: Some(AccessibilityHandlerWrapper::on_accessibility_tree_change),
                on_accessibility_location_change: Some(AccessibilityHandlerWrapper::on_accessibility_location_change),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for AccessibilityHandlerWrapper: cef_accessibility_handler_t {
        fn on_accessibility_tree_change(
            &self,
            value: Value: *mut cef_value_t,
        ) {
            unsafe{ self.delegate.get() }.on_accessibility_tree_change(value.into())
        }
        fn on_accessibility_location_change(
            &self,
            value: Value: *mut cef_value_t,
        ) {
            unsafe{ self.delegate.get() }.on_accessibility_location_change(value.into())
        }
    }
}
