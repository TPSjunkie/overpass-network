// ./overpass-rs/ovp-client/src/common/logging/logger.rs

use log::{Level, Record};

pub trait Logger {
    fn log(&self, level: &str, message: &str);
}

impl Logger for dyn log::Log {
    fn log(&self, level: &str, message: &str) {
        let level = match level {
            "error" => Level::Error,
            "warn" => Level::Warn,
            "info" => Level::Info,
            "debug" => Level::Debug,
            "trace" => Level::Trace,
            _ => Level::Info,
        };
        self.log(
            &Record::builder()
                .level(level)
                .target("ovp-client")
                .args(format_args!("{}", message))
                .build(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_log() {
        let logger = log::logger();
        Logger::log(logger, "info", "test");
    }
}
