use cef_sys::{cef_render_process_handler_t};
use std::collections::HashMap;
use crate::{
    browser::Browser,
    load_handler::LoadHandler,
    StoredValue,
};

pub trait RenderProcessHandler: Send + Sync {
    /// Called after the render process main thread has been created. `extra_info`
    /// is originating from
    /// [BrowserProcessHandler::on_render_process_thread_created()].
    fn on_render_thread_created(&self, extra_info: &Vec<StoredValue>) {}
    /// Called after WebKit has been initialized.
    fn on_web_kit_initialized(&self) {}
    /// Called after a browser has been created. When browsing cross-origin a new
    /// browser will be created before the old browser with the same identifier is
    /// destroyed. |extra_info| is originating from
    /// [BrowserHost::create_browser()],
    /// [BrowserHost::create_browser_sync()],
    /// [LifeSpanHandler::on_before_popup()] or [BrowserView::create()].
    fn on_browser_created(&self, browser: &mut Browser, extra_info: &HashMap<String, StoredValue>) {}
    /// Called before a browser is destroyed.
    fn on_browser_destroyed(&self, browser: &mut Browser) {}
    /// Return the handler for browser load status events.
    fn get_load_handler(&self) -> Option<Box<dyn LoadHandler>> { None }
}
