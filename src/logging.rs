use cef_sys::{cef_log, cef_log_severity_t, cef_get_min_log_level};
use log::{Record, Level, Metadata};
use std::borrow::Cow;
use std::ffi::CString;

/// Integration of Rust's log crate with CEF. Example usage:
///
/// ```rust
///  let mut logger_builder = cef::logging::Logger::builder();
///  logger_builder.level(log::LevelFilter::Warn);
///  logger_builder.level_for_module_path("mycrate", log::LevelFilter::Debug);
///  let logger = logger_builder.build();
///
///  log::set_boxed_logger(Box::new(logger)).map(|()| log::set_max_level(log::LevelFilter::Info)).unwrap();
///  log::info!("Hello World!");
/// ```
///
/// Note that you have to call [cef::Context::initialize] before logging can be used.
/// Also, don't forget to configure CEF's log level, which is separate from the one managed by [log].
/// Only if a message's level passes both filters, it will actually be logged.
pub struct Logger {
    level: log::LevelFilter,
    module_path_levels: Vec<(Cow<'static, str>, log::LevelFilter)>,
}

impl Logger {
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::new()
    }

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
        match level as cef_log_severity_t::Type {
            cef_log_severity_t::LOGSEVERITY_DEBUG => Level::Debug,
            cef_log_severity_t::LOGSEVERITY_INFO => Level::Info,
            _ => Level::Error,
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let level = unsafe { cef_get_min_log_level() };
        if level == cef_log_severity_t::LOGSEVERITY_DISABLE as _ {
            false
        } else {
            metadata.level() >= Self::log_level(level as _)
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_module_path = record.module_path().or(record.module_path_static());
            let log_level = record.metadata().level();

            if let Some(log_module_path) = log_module_path {
                let module_level = self
                    .module_path_levels
                    .iter()
                    .find_map(|(module_path, level)| {
                        if log_module_path.starts_with(module_path.as_ref()) {
                            Some(level)
                        } else {
                            None
                        }
                    });
                let active_level = module_level.unwrap_or(&self.level);

                if log_level > *active_level {
                    return;
                }
            }

            if let Ok(text) = CString::new(format!("{}", record.args()).as_bytes()) {
                let file = CString::new(record.file().unwrap_or("<unknown>").as_bytes())
                    .unwrap_or_else(|_| CString::new(b"<unknown>".to_vec()).unwrap());
                let line = record.line().unwrap_or(0);
                let cef_log_level = Self::cef_level(log_level);
                unsafe {
                    cef_log(file.as_ptr(), line as _, cef_log_level as _, text.as_ptr());
                }
            }
        }
    }

    fn flush(&self) {}
}

pub struct LoggerBuilder {
    level: log::LevelFilter,
    module_path_levels: Vec<(Cow<'static, str>, log::LevelFilter)>,
}

impl LoggerBuilder {
    pub fn new() -> LoggerBuilder {
        LoggerBuilder {
            level: log::LevelFilter::Off,
            module_path_levels: Vec::new(),
        }
    }

    pub fn level(mut self, level: log::LevelFilter) -> Self {
        self.level = level;
        self
    }

    pub fn level_for_module_path<S: Into<Cow<'static, str>>>(
        mut self,
        module_path: S,
        level: log::LevelFilter,
    ) -> Self {
        self.module_path_levels.push((module_path.into(), level));
        self
    }

    pub fn build(self) -> Logger {
        Logger {
            level: self.level,
            module_path_levels: self.module_path_levels,
        }
    }
}
