use crate::{
    events::EventFlags,
    refcounted::{RefCountedPtr, Wrapper},
};
use cef_sys::{
    _cef_run_context_menu_callback_t,
    cef_event_flags_t,
};
use std::{
    os::raw::c_int,
};
use super::CommandId;

ref_counted_ptr!{
    /// Callback structure used for continuation of custom context menu display.
    pub struct RunContextMenu(*mut _cef_run_context_menu_callback_t);
}

impl RunContextMenu {
    pub fn new<C: RunContextMenuCallbacks>(callbacks: C) -> RunContextMenu {
        unsafe{ RunContextMenu::from_ptr_unchecked(RunContextMenuWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }

    /// Complete context menu display by selecting the specified `command_id` and
    /// `event_flags`.
    pub fn cont(&self, command_id: CommandId, event_flags: EventFlags) {
        unsafe {
            self.0.cont.unwrap()(
                self.as_ptr(),
                command_id.get(),
                cef_event_flags_t(event_flags.bits() as _)
            )
        }
    }
    /// Cancel context menu display.
    pub fn cancel(&self) {
        unsafe {
            self.0.cancel.unwrap()(
                self.as_ptr(),
            )
        }
    }
}

pub struct RunContextMenuWrapper(Box<dyn RunContextMenuCallbacks>);
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

pub trait RunContextMenuCallbacks: 'static + Send + Sync {
    /// Complete context menu display by selecting the specified `command_id` and
    /// `event_flags`.
    fn cont(
        &self,
        command_id: CommandId,
        event_flags: EventFlags,
    );
    /// Cancel context menu display.
    fn cancel(&self);
}
cef_callback_impl!{
    impl for RunContextMenuWrapper: _cef_run_context_menu_callback_t {
        fn cont(
            &self,
            command_id: CommandId: c_int,
            event_flags: EventFlags: cef_event_flags_t
        ) {
            self.0.cont(command_id, event_flags);
        }
        fn cancel(&self) {
            self.0.cancel();
        }
    }
}
