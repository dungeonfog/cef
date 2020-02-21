use cef_sys::{cef_browser_process_handler_t, cef_command_line_t, cef_list_value_t};

use crate::{
    command_line::CommandLine,
    refcounted::{RefCountedPtr, Wrapper},
    values::ListValue,
    // print_handler::PrintHandler,
};

ref_counted_ptr!{
    pub struct BrowserProcessHandler(*mut cef_browser_process_handler_t);
}

impl BrowserProcessHandler {
    pub fn new<C: BrowserProcessHandlerCallbacks>(callbacks: C) -> BrowserProcessHandler {
        unsafe{ BrowserProcessHandler::from_ptr_unchecked(BrowserProcessHandlerWrapper::new(Box::new(callbacks)).wrap().into_raw()) }
    }
}

/// Trait used to implement browser process callbacks.
pub trait BrowserProcessHandlerCallbacks: 'static + Sync + Send {
    /// Called on the browser process UI thread immediately after the CEF context
    /// has been initialized.
    fn on_context_initialized(&self) {}
    /// Called before a child process is launched. Will be called on the browser
    /// process UI thread when launching a render process and on the browser
    /// process IO thread when launching a GPU or plugin process. Provides an
    /// opportunity to modify the child process command line.
    fn on_before_child_process_launch(&self, _command_line: CommandLine) {}
    /// Called on the browser process IO thread after the main thread has been
    /// created for a new render process. Provides an opportunity to specify extra
    /// information that will be passed to
    /// [RenderProcessHandlerCallbacks::on_render_thread_created()] in the render
    /// process.
    fn on_render_process_thread_created(&self, _extra_info: ListValue) {}
    /// Return the handler for printing on Linux. If a print handler is not
    /// provided then printing will not be supported on the Linux platform.
    #[cfg(target_os = "linux")]
    fn get_print_handler(&self) -> Option<PrintHandler> {
        None
    }
    /// Called from any thread when work has been scheduled for the browser process
    /// main (UI) thread. This callback is used in combination with [CefSettings::external_message_pump]
    /// and [App::do_message_loop_work] in cases where the CEF
    /// message loop must be integrated into an existing application message loop
    /// (see additional comments and warnings on [App::do_message_loop_work]). This
    /// callback should schedule a [App::cef_do_message_loop_work] call to happen on the
    /// main (UI) thread. `delay_ms` is the requested delay in milliseconds. If
    /// `delay_ms` is <= 0 then the call should happen reasonably soon. If
    /// `delay_ms` is > 0 then the call should be scheduled to happen after the
    /// specified delay and any currently pending scheduled call should be
    /// cancelled.
    fn on_schedule_message_pump_work(&self, delay_ms: i64) {}
}

pub(crate) struct BrowserProcessHandlerWrapper {
    delegate: Box<dyn BrowserProcessHandlerCallbacks>,
}

impl Wrapper for BrowserProcessHandlerWrapper {
    type Cef = cef_browser_process_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
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
            self,
        )
    }
}

impl BrowserProcessHandlerWrapper {
    pub(crate) fn new(delegate: Box<dyn BrowserProcessHandlerCallbacks>) -> BrowserProcessHandlerWrapper {
        Self {
            delegate,
        }
    }
}

cef_callback_impl! {
    impl for BrowserProcessHandlerWrapper: cef_browser_process_handler_t {
        fn context_initialized(&self) {
            self.delegate.on_context_initialized();
        }
        fn before_child_process_launch(
            &self,
            command_line: CommandLine: *mut cef_command_line_t,
        ) {
            self.delegate.on_before_child_process_launch(command_line);
        }
        fn render_process_thread_created(
            &self,
            extra_info: ListValue: *mut cef_list_value_t,
        ) {
            self.delegate.on_render_process_thread_created(extra_info);
        }
        #[cfg(target_os = "linux")]
        fn get_print_handler(
            &self
        ) -> *mut cef_sys::cef_print_handler_t {
            self.delegate.get_print_handler().map(|h| h.into_raw()).unwrap_or(std::ptr::null_mut())
        }
        fn schedule_message_pump_work(
            &self,
            delay_ms: i64: i64,
        ) {
            self.delegate.on_schedule_message_pump_work(delay_ms);
        }
    }
}

#[cfg(target_os = "linux")]
pub use self::print_handler::PrintHandler;
#[cfg(target_os = "linux")]
mod print_handler {
    ref_counted_ptr!{
        pub struct PrintHandler(*mut cef_sys::cef_print_handler_t);
    }
}
