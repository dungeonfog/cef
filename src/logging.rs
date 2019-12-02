use cef_sys::{cef_log, cef_log_severity_t, cef_get_min_log_level};
use log::{Record, Level, Metadata};
use std::ffi::CString;

/// Integration of Rust's log crate with CEF. Example usage:
///
/// ```rust
///  static LOGGER: cef::Logger = cef::Logger;
///
///  log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info)).unwrap();
///  info!("Hello World!");
/// ```
///
/// Note that you have to call [cef::Context::initialize] before logging can be used.
/// Also, don't forget to configure CEF's log level, which is separate from the one managed by [log].
/// Only if a message's level passes both filters, it will actually be logged.
pub struct Logger;

impl Logger {
    fn cef_level(level: Level) -> i32 {
        // for some reason, these are different than cef_log_severity_t!
        match level {
            Level::Error => 2,
            Level::Warn  => 1,
            Level::Info  => 0,
            Level::Debug => 0,
            Level::Trace => 0,
        }
    }
    fn log_level(level: i32) -> Level {
        match level {
            1 => Level::Warn,
            2 => Level::Error,
            3 | 4 => Level::max(),
            _ => Level::Info,
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let level = unsafe { cef_get_min_log_level() };
        if level == cef_log_severity_t::LOGSEVERITY_DISABLE as _ {
            false
        } else {
            metadata.level() <= Self::log_level(level as _)
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if let Ok(text) = CString::new(format!("{}", record.args()).as_bytes()) {
                let file = CString::new(record.file().unwrap_or("<unknown>").as_bytes()).unwrap_or_else(|_| CString::new(b"<unknown>".to_vec()).unwrap());
                let line = record.line().unwrap_or(0);
                let level = Self::cef_level(record.metadata().level());
                unsafe {
                    cef_log(file.as_ptr(), line as _, level as _, text.as_ptr());
                }
            }
        }
    }

    fn flush(&self) {}
}