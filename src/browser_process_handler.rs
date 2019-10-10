use cef_sys::{
    cef_base_ref_counted_t, cef_browser_process_handler_t, cef_command_line_t, cef_list_value_t,
};
use std::ptr::null_mut;

use crate::{
    command_line::CommandLine,
    refcounted::{RefCounted, RefCounter},
    values::{ListValue, StoredValue},
    // print_handler::PrintHandler,
};

/// Trait used to implement browser process callbacks. The functions of this
/// trait will be called on the browser process main thread unless otherwise
/// indicated.
pub trait BrowserProcessHandler: Sync + Send {
    /// Called on the browser process UI thread immediately after the CEF context
    /// has been initialized.
    fn on_context_initialized(&self) {}
    /// Called before a child process is launched. Will be called on the browser
    /// process UI thread when launching a render process and on the browser
    /// process IO thread when launching a GPU or plugin process. Provides an
    /// opportunity to modify the child process command line.
    fn on_before_child_process_launch(&self, _command_line: &mut CommandLine) {}
    /// Called on the browser process IO thread after the main thread has been
    /// created for a new render process. Provides an opportunity to specify extra
    /// information that will be passed to
    /// [RenderProcessHandler::on_render_thread_created()] in the render
    /// process.
    fn on_render_process_thread_created(&self, _extra_info: &mut Vec<StoredValue>) {}
    /// Return the handler for printing on Linux. If a print handler is not
    /// provided then printing will not be supported on the Linux platform.
    #[cfg(target_os = "linux")]
    fn get_print_handler(&self) -> Option<Box<dyn PrintHandler>> {
        None
    }
    /// Called from any thread when work has been scheduled for the browser process
    /// main (UI) thread. This callback is used in combination with [CefSettings].
    /// external_message_pump and [cef_do_message_loop_work()] in cases where the CEF
    /// message loop must be integrated into an existing application message loop
    /// (see additional comments and warnings on CefDoMessageLoopWork). This
    /// callback should schedule a [cef_do_message_loop_work()] call to happen on the
    /// main (UI) thread. `delay_ms` is the requested delay in milliseconds. If
    /// `delay_ms` is <= 0 then the call should happen reasonably soon. If
    /// `delay_ms` is > 0 then the call should be scheduled to happen after the
    /// specified delay and any currently pending scheduled call should be
    /// cancelled.
    fn on_schedule_message_pump_work(&self, delay_ms: i64) {}
}

pub(crate) struct BrowserProcessHandlerWrapper {
    delegate: Box<dyn BrowserProcessHandler>,
    #[cfg(target_os = "linux")]
    print_handler: *mut RefCounted<cef_print_handler_t>,
}

unsafe impl Send for BrowserProcessHandlerWrapper {}
unsafe impl Sync for BrowserProcessHandlerWrapper {}

impl BrowserProcessHandlerWrapper {
    pub(crate) fn new(
        delegate: Box<dyn BrowserProcessHandler>,
    ) -> *mut RefCounted<cef_browser_process_handler_t> {
        RefCounted::new(
            cef_browser_process_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_context_initialized: Some(Self::context_initialized),
                on_before_child_process_launch: Some(Self::before_child_process_launch),
                on_render_process_thread_created: Some(Self::render_process_thread_created),
                #[cfg(target_os = "linux")]
                get_print_handler: Some(Self::get_print_handler),
                #[cfg(not(target_os = "linux"))]
                get_print_handler: None,
                on_schedule_message_pump_work: Some(Self::schedule_message_pump_work),
            },
            Self {
                delegate,
                #[cfg(target_os = "linux")]
                print_handler: null_mut(),
            },
        )
    }

    extern "C" fn context_initialized(self_: *mut cef_browser_process_handler_t) {
        let this = unsafe { RefCounted::<cef_browser_process_handler_t>::make_temp(self_) };
        this.delegate.on_context_initialized();
    }
    extern "C" fn before_child_process_launch(
        self_: *mut cef_browser_process_handler_t,
        command_line: *mut cef_command_line_t,
    ) {
        let this = unsafe { RefCounted::<cef_browser_process_handler_t>::make_temp(self_) };
        this.delegate
            .on_before_child_process_launch(&mut CommandLine::from(command_line));
    }
    extern "C" fn render_process_thread_created(
        self_: *mut cef_browser_process_handler_t,
        extra_info: *mut cef_list_value_t,
    ) {
        let this = unsafe { RefCounted::<cef_browser_process_handler_t>::make_temp(self_) };
        let mut ei = ListValue::from(extra_info).into();
        this.delegate.on_render_process_thread_created(&mut ei);
        // TODO: copy stuff back from ei to extra_info
    }
    #[cfg(target_os = "linux")]
    extern "C" fn get_print_handler(
        self_: *mut cef_browser_process_handler_t,
    ) -> *mut cef_print_handler_t {
        let mut this = unsafe { RefCounted::<cef_browser_process_handler_t>::make_temp(self_) };
        if let Some(handler) = this.delegate.get_print_handler() {
            let wrapper = PrintHandlerWrapper::new(handler);
            this.print_handler = wrapper;
            wrapper as *mut cef_print_handler_t
        } else {
            if !this.print_handler.is_null() {
                RefCounted::<cef_print_handler_t>::release(
                    (*this).print_handler as *mut cef_base_ref_counted_t,
                );
                this.print_handler = null_mut();
            }
            null_mut()
        }
    }
    extern "C" fn schedule_message_pump_work(
        self_: *mut cef_browser_process_handler_t,
        delay_ms: i64,
    ) {
        let this = unsafe { RefCounted::<cef_browser_process_handler_t>::make_temp(self_) };
        this.delegate.on_schedule_message_pump_work(delay_ms);
    }
}

#[cfg(target_os = "linux")]
impl Drop for BrowserProcessHandlerWrapper {
    fn drop(&mut self) {
        if !self.print_handler.is_null() {
            RefCounted::<cef_print_handler_t>::release(
                self.print_handler as *mut cef_base_ref_counted_t,
            );
        }
    }
}
