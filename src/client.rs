use cef_sys::{
    cef_browser_t, cef_client_t, cef_frame_t, cef_load_handler_t, cef_process_id_t,
    cef_process_message_t, cef_request_handler_t,
};
use downcast_rs::{impl_downcast, Downcast};
use std::{ptr::null_mut, sync::Arc};

use crate::{
    browser::Browser,
    frame::Frame,
    load_handler::{LoadHandler, LoadHandlerWrapper},
    process::{ProcessId, ProcessMessage},
    refcounted::{RefCountedPtr, Wrapper},
    request_handler::{RequestHandler, RequestHandlerWrapper},
};

/// Implement this trait to provide handler implementations.
pub trait Client: 'static + Send + Sync + Downcast {
    // /// Return the handler for audio rendering events.
    // fn get_audio_handler(&self) -> Option<Box<dyn AudioHandler>> { None }
    // /// Return the handler for context menus. If no handler is provided the default
    // /// implementation will be used.
    // fn get_context_menu_handler(&self) -> Option<Box<dyn ContextMenuHandler>> { None }
    // /// Return the handler for dialogs. If no handler is provided the default
    // /// implementation will be used.
    // fn get_dialog_handler(&self) -> Option<Box<dyn DialogHandler>> { None }
    // /// Return the handler for browser display state events.
    // fn get_display_handler(&self) -> Option<Box<dyn DisplayHandler>> { None }
    // /// Return the handler for download events. If no handler is returned downloads
    // /// will not be allowed.
    // fn get_download_handler(&self) -> Option<Box<dyn DownloadHandler>> { None }
    // /// Return the handler for drag events.
    // fn get_drag_handler(&self) -> Option<Box<dyn DragHandler>> { None }
    // /// Return the handler for find result events.
    // fn get_find_handler(&self) -> Option<Box<dyn FindHandler>> { None }
    // /// Return the handler for focus events.
    // fn get_focus_handler(&self) -> Option<Box<dyn FocusHandler>> { None }
    // /// Return the handler for JavaScript dialogs. If no handler is provided the
    // /// default implementation will be used.
    // fn get_jsdialog_handler(&self) -> Option<Box<dyn JsDialogHandler>> { None }
    // /// Return the handler for keyboard events.
    // fn get_keyboard_handler(&self) -> Option<Box<dyn KeyboardHandler>> { None }
    // /// Return the handler for browser life span events.
    // fn get_life_span_handler(&self) -> Option<Box<dyn LifeSpanHandler>> { None }
    /// Return the handler for browser load status events.
    fn get_load_handler(&self) -> Option<Arc<dyn LoadHandler>> {
        None
    }
    // /// Return the handler for off-screen rendering events.
    // fn get_render_handler(&self) -> Option<Box<dyn RenderHandler>> { None }
    /// Return the handler for browser request events.
    fn get_request_handler(&self) -> Option<Arc<dyn RequestHandler>> {
        None
    }
    /// Called when a new message is received from a different process. Return true
    /// if the message was handled or false otherwise.
    fn on_process_message_received(
        &self,
        browser: Browser,
        frame: Frame,
        process_id: ProcessId,
        message: ProcessMessage,
    ) -> bool {
        false
    }
}

impl_downcast!(Client);

#[repr(transparent)]
pub(crate) struct ClientWrapper(Arc<dyn Client>);

impl Wrapper for ClientWrapper {
    type Cef = cef_client_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_client_t {
                base: unsafe { std::mem::zeroed() },
                get_load_handler: Some(Self::get_load_handler),
                get_request_handler: Some(Self::get_request_handler),
                on_process_message_received: Some(Self::process_message_received),
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
}

impl ClientWrapper {
    pub(crate) fn new(delegate: Arc<dyn Client>) -> Self {
        Self(delegate)
    }
    pub(crate) fn get_client<C: Client>(&self) -> Option<&C> {
        self.0.downcast_ref()
    }
}

cef_callback_impl! {
    impl for ClientWrapper: cef_client_t {
        fn get_load_handler(&self) -> *mut cef_load_handler_t {
            self.0.get_load_handler().map(|lh| LoadHandlerWrapper::new(lh).wrap().into_raw()).unwrap_or_else(null_mut)
        }
        fn get_request_handler(&self) -> *mut cef_request_handler_t {
            self.0.get_request_handler().map(|rh| RequestHandlerWrapper::new(rh).wrap().into_raw()).unwrap_or_else(null_mut)
        }
        fn process_message_received(
            &self,
            browser       : Browser       : *mut cef_browser_t,
            frame         : Frame         : *mut cef_frame_t,
            source_process: ProcessId     : cef_process_id_t::Type,
            message       : ProcessMessage: *mut cef_process_message_t
        ) -> std::os::raw::c_int {
            self.0.on_process_message_received(browser, frame, source_process, message) as std::os::raw::c_int
        }
    }
}
