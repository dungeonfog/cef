use cef_sys::{cef_client_t, cef_load_handler_t, cef_browser_t, cef_frame_t, cef_process_id_t, cef_process_message_t};
use std::ptr::null_mut;

use crate::{
    refcounted::{RefCounted},
    load_handler::{LoadHandler, LoadHandlerWrapper},
    process::{ProcessId, ProcessMessage},
    request_handler::RequestHandler,
};

/// Implement this trait to provide handler implementations.
pub trait Client: 'static + Sync + Send {
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
    fn get_load_handler(&self) -> Option<Box<dyn LoadHandler<Self>>> { None }
    // /// Return the handler for off-screen rendering events.
    // fn get_render_handler(&self) -> Option<Box<dyn RenderHandler>> { None }
    /// Return the handler for browser request events.
    fn get_request_handler(&self) -> Option<Box<dyn RequestHandler>> { None }
    /// Called when a new message is received from a different process. Return true
    /// if the message was handled or false otherwise.
    fn on_process_message_received(&self, browser: &Browser<Self>, frame: &Frame<Self>, process_id: ProcessId, message: &ProcessMessage) -> bool { false }
}

pub(crate) struct ClientWrapper<C> where C: Client {
    delegate: C,
    load_handler: Option<Box<dyn LoadHandler<Self>>>,
}

impl<C> ClientWrapper<C> where C: Client {
    pub(crate) fn wrap(delegate: C) -> *mut cef_client_t {
        let rc = RefCounted::new(
            cef_client_t {
                base: unsafe { std::mem::zeroed() },
                get_load_handler: Some(Self::get_load_handler),
                on_process_message_received: Some(Self::process_message_received),
                ..unsafe { std::mem::zeroed() }
            },
            Self {
                delegate,
                load_handler: None,
            },
        );
        unsafe { &mut *rc }.get_cef()
    }
}

cef_callback_impl! {
    impl ClientWrapper<C>: cef_client_t {
        fn get_load_handler(&mut self) -> *mut cef_load_handler_t {
            let load_handler = self.delegate.get_load_handler();
            if let Some(load_handler) = load_handler {
                self.load_handler.replace(load_handler);
            } else {
                self.load_handler.take();
            }

            load_handler.map(LoadHandlerWrapper::wrap).unwrap_or_else(null_mut)
        }
        fn process_message_received(&mut self,
            browser       : Browser<C>    : *mut cef_browser_t,
            frame         : Frame<C>      : *mut cef_frame_t,
            source_process: ProcessId     : cef_process_id_t,
            message       : ProcessMessage: *mut cef_process_message_t
        ) -> std::os::raw::c_int {
            self.delegate.on_process_message_received(&browser, &frame, source_process, &message) as std::os::raw::c_int
        }
    }
}
