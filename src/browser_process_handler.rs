use crate::{
    command_line::CommandLine,
    values::StoredValue,
    // print_handler::PrintHandler,
};

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
    fn get_print_handler(&self) -> Option<Box<impl PrintHandler>> { None }
    /// Called from any thread when work has been scheduled for the browser process
    /// main (UI) thread. This callback is used in combination with CefSettings.
    /// external_message_pump and cef_do_message_loop_work() in cases where the CEF
    /// message loop must be integrated into an existing application message loop
    /// (see additional comments and warnings on CefDoMessageLoopWork). This
    /// callback should schedule a cef_do_message_loop_work() call to happen on the
    /// main (UI) thread. |delay_ms| is the requested delay in milliseconds. If
    /// |delay_ms| is <= 0 then the call should happen reasonably soon. If
    /// |delay_ms| is > 0 then the call should be scheduled to happen after the
    /// specified delay and any currently pending scheduled call should be
    /// cancelled.
    fn on_schedule_message_pump_work(&self, delay_ms: i64) {}
}
