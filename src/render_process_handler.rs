use cef_sys::{
    cef_base_ref_counted_t, cef_browser_t, cef_dictionary_value_t, cef_domnode_t, cef_frame_t,
    cef_list_value_t, cef_load_handler_t, cef_process_id_t, cef_process_message_t,
    cef_render_process_handler_t, cef_v8context_t, cef_v8exception_t, cef_v8stack_trace_t,
};
use std::{collections::HashMap, convert::TryFrom, ptr::null_mut, sync::Arc};

use crate::{
    browser::Browser,
    browser_process_handler::BrowserProcessHandler,
    dom::DOMNode,
    frame::Frame,
    load_handler::{LoadHandler, LoadHandlerWrapper},
    process::{ProcessId, ProcessMessage},
    ptr_hash::Hashed,
    refcounted::{RefCounted, RefCounter},
    v8context::{V8Context, V8Exception, V8StackTrace},
    values::{DictionaryValue, ListValue},
    StoredValue,
};

/// Trait used to implement render process callbacks. The functions of this
/// trait will be called on the render process main thread ([ProcessId::Renderer])
/// unless otherwise indicated.
pub trait RenderProcessHandler: Send + Sync {
    /// Called after the render process main thread has been created. `extra_info`
    /// is originating from
    /// [BrowserProcessHandler::on_render_process_thread_created].
    fn on_render_thread_created(&self, extra_info: &Vec<StoredValue>) {}
    /// Called after WebKit has been initialized.
    fn on_web_kit_initialized(&self) {}
    /// Called after a browser has been created. When browsing cross-origin a new
    /// browser will be created before the old browser with the same identifier is
    /// destroyed. |extra_info| is originating from
    /// [BrowserHost::create_browser()],
    /// [BrowserHost::create_browser_sync()],
    /// [LifeSpanHandler::on_before_popup()] or [BrowserView::create()].
    fn on_browser_created(&self, browser: &Browser, extra_info: &HashMap<String, StoredValue>) {}
    /// Called before a browser is destroyed.
    fn on_browser_destroyed(&self, browser: &Browser) {}
    /// Return the handler for browser load status events.
    fn get_load_handler(&self) -> Option<Box<dyn LoadHandler + 'static>> {
        None
    }
    /// Called immediately after the V8 context for a frame has been created. To
    /// retrieve the JavaScript 'window' object use the
    /// [V8Context::get_global()] function. V8 handles can only be accessed
    /// from the thread on which they are created. A task runner for posting tasks
    /// on the associated thread can be retrieved via the
    /// [V8Context::get_task_runner()] function.
    fn on_context_created(&self, browser: &Browser, frame: &Frame, context: &V8Context) {}
    /// Called immediately before the V8 context for a frame is released.
    fn on_context_released(&self, browser: &Browser, frame: &Frame, context: &V8Context) {}
    /// Called for global uncaught exceptions in a frame. Execution of this
    /// callback is disabled by default. To enable set
    /// [CefSettings.uncaught_exception_stack_size] > 0.
    fn on_uncaught_exception(
        &self,
        browser: &Browser,
        frame: &Frame,
        context: &V8Context,
        exception: &V8Exception,
        stack_trace: &V8StackTrace,
    ) {
    }
    /// Called when a new node in the the browser gets focus. The `node` value may
    /// be None if no specific node has gained focus. The node object passed to
    /// this function represents a snapshot of the DOM at the time this function is
    /// executed.
    fn on_focused_node_changed(&self, browser: &Browser, frame: &Frame, node: Option<&DOMNode>) {}
    /// Called when a new message is received from a different process. Return true
    /// if the message was handled or false otherwise.
    fn on_process_message_received(
        &self,
        browser: &Browser,
        frame: &Frame,
        source_process: ProcessId,
        message: &ProcessMessage,
    ) -> bool {
        false
    }
}

pub(crate) struct RenderProcessHandlerWrapper {
    delegate: Box<dyn RenderProcessHandler>,
    load_handler: *mut RefCounted<cef_load_handler_t>,
}

unsafe impl Send for RenderProcessHandlerWrapper {}
unsafe impl Sync for RenderProcessHandlerWrapper {}

impl RenderProcessHandlerWrapper {
    pub(crate) fn new(
        delegate: Box<dyn RenderProcessHandler>,
    ) -> *mut RefCounted<cef_render_process_handler_t> {
        RefCounted::new(
            cef_render_process_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_render_thread_created: Some(Self::render_thread_created),
                on_web_kit_initialized: Some(Self::web_kit_initialized),
                on_browser_created: Some(Self::browser_created),
                on_browser_destroyed: Some(Self::browser_destroyed),
                get_load_handler: Some(Self::get_load_handler),
                on_context_created: Some(Self::context_created),
                on_context_released: Some(Self::context_released),
                on_uncaught_exception: Some(Self::uncaught_exception),
                on_focused_node_changed: Some(Self::focused_node_changed),
                on_process_message_received: Some(Self::process_message_received),
            },
            Self {
                delegate,
                load_handler: null_mut(),
            },
        )
    }

    extern "C" fn render_thread_created(
        self_: *mut cef_render_process_handler_t,
        extra_info: *mut cef_list_value_t,
    ) {
        let this = unsafe { RefCounted::<cef_render_process_handler_t>::make_temp(self_) };
        this.delegate
            .on_render_thread_created(&ListValue::from(extra_info).into());
    }

    extern "C" fn web_kit_initialized(self_: *mut cef_render_process_handler_t) {
        let this = unsafe { RefCounted::<cef_render_process_handler_t>::make_temp(self_) };
        (*this).delegate.on_web_kit_initialized();
    }

    extern "C" fn browser_created(
        self_: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        extra_info: *mut cef_dictionary_value_t,
    ) {
        let this = unsafe { RefCounted::<cef_render_process_handler_t>::make_temp(self_) };
        (*this).delegate.on_browser_created(
            &Browser::from(browser),
            &DictionaryValue::from(extra_info).into(),
        );
    }

    extern "C" fn browser_destroyed(
        self_: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
    ) {
        let this = unsafe { RefCounted::<cef_render_process_handler_t>::make_temp(self_) };
        (*this)
            .delegate
            .on_browser_destroyed(&Browser::from(browser));
    }

    extern "C" fn get_load_handler(
        self_: *mut cef_render_process_handler_t,
    ) -> *mut cef_load_handler_t {
        let mut this = unsafe { RefCounted::<cef_render_process_handler_t>::make_temp(self_) };
        if let Some(handler) = (*this).delegate.get_load_handler() {
            let wrapper = LoadHandlerWrapper::new(handler);
            (*this).load_handler = wrapper;
            wrapper as *mut cef_load_handler_t
        } else {
            if !(*this).load_handler.is_null() {
                RefCounted::<cef_render_process_handler_t>::release(
                    (*this).load_handler as *mut cef_base_ref_counted_t,
                );
                (*this).load_handler = null_mut();
            }
            std::ptr::null_mut()
        }
    }

    extern "C" fn context_created(
        self_: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        context: *mut cef_v8context_t,
    ) {
        let this = unsafe { RefCounted::<cef_render_process_handler_t>::make_temp(self_) };
        (*this).delegate.on_context_created(
            &Browser::from(browser),
            &Frame::from(frame),
            &V8Context::from(context),
        );
    }

    extern "C" fn context_released(
        self_: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        context: *mut cef_v8context_t,
    ) {
        let this = unsafe { RefCounted::<cef_render_process_handler_t>::make_temp(self_) };
        (*this).delegate.on_context_created(
            &Browser::from(browser),
            &Frame::from(frame),
            &V8Context::from(context),
        );
    }

    extern "C" fn uncaught_exception(
        self_: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        context: *mut cef_v8context_t,
        exception: *mut cef_v8exception_t,
        stack_trace: *mut cef_v8stack_trace_t,
    ) {
        let this = unsafe { RefCounted::<cef_render_process_handler_t>::make_temp(self_) };
        (*this).delegate.on_uncaught_exception(
            &Browser::from(browser),
            &Frame::from(frame),
            &V8Context::from(context),
            &V8Exception::from(exception),
            &V8StackTrace::from(stack_trace),
        );
    }

    extern "C" fn focused_node_changed(
        self_: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        node: *mut cef_domnode_t,
    ) {
        let this = unsafe { RefCounted::<cef_render_process_handler_t>::make_temp(self_) };
        match DOMNode::try_from(node) {
            Ok(domnode) => (*this).delegate.on_focused_node_changed(
                &Browser::from(browser),
                &Frame::from(frame),
                Some(&domnode),
            ),
            Err(_) => (*this).delegate.on_focused_node_changed(
                &Browser::from(browser),
                &Frame::from(frame),
                None,
            ),
        };
    }

    extern "C" fn process_message_received(
        self_: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        source_process: cef_process_id_t::Type,
        message: *mut cef_process_message_t,
    ) -> std::os::raw::c_int {
        let this = unsafe { RefCounted::<cef_render_process_handler_t>::make_temp(self_) };
        (*this).delegate.on_process_message_received(
            &Browser::from(browser),
            &Frame::from(frame),
            unsafe { ProcessId::from_unchecked(source_process as i32) },
            &ProcessMessage::from(message),
        ) as std::os::raw::c_int
    }
}
