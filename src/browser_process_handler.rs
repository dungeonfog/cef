use crate::{
    command_line::CommandLine,
    // print_handler::PrintHandler,
};

pub trait BrowserProcessHandler {
    fn on_context_initialized(&self) {}
    fn on_before_child_process_launch(&self, _command_line: &CommandLine) {}
    fn on_render_process_thread_created(&self, _extra_info: &[u8]) {} // cef_list_value_t?
    // fn get_print_handler(&self) -> Option<Box<dyn PrintHandler>> { None }
    fn on_schedule_message_pump_work(&self, delay_ms: i64) {}
}
