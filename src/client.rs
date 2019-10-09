use cef_sys::{cef_client_t, cef_base_ref_counted_t};

use crate::{
    refcounted::{RefCounter, RefCounted},
    browser::Browser,
    frame::Frame,
    process::ProcessId,
};

/// Implement this trait to provide handler implementations.
pub trait Client {
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
    // /// Return the handler for browser load status events.
    // fn get_load_handler(&self) -> Option<Box<dyn LoadHandler>> { None }
    // /// Return the handler for off-screen rendering events.
    // fn get_render_handler(&self) -> Option<Box<dyn RenderHandler>> { None }
    // /// Return the handler for browser request events.
    // fn get_request_handler(&self) -> Option<Box<dyn RequestHandler>> { None }
    // // Called when a new message is received from a different process. Return true
    // // if the message was handled or false otherwise.
    // fn on_process_message_received(&self, browser: &Browser, frame: &Frame, process_id: ProcessId, message: &ProcessMessage) -> bool { false }
}

pub(crate) struct ClientWrapper {
    delegate: Box<dyn Client>,
}

impl RefCounter for cef_client_t {
    type Wrapper = ClientWrapper;
    fn set_base(&mut self, base: cef_base_ref_counted_t) {
        self.base = base;
    }
}

impl ClientWrapper {
    pub(crate) fn wrap<C: Client + 'static>(delegate: C) -> *mut cef_client_t {
        let mut rc = RefCounted::new(cef_client_t {
            base: unsafe { std::mem::zeroed() },
            ..unsafe { std::mem::zeroed() }
        }, Self {
            delegate: Box::new(delegate),
        });
        unsafe { &mut *rc }.get_cef()
    }
}
