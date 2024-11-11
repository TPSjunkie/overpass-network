// src/logging/configuration.rs

use log::{LevelFilter, SetLoggerError};
use simplelog::{Config, SimpleLogger, WriteLogger};
use std::fs::File;

/// Configures the logging system.
pub struct LoggingConfiguration {
    pub level: LevelFilter,
    pub file: Option<String>,
}

impl LoggingConfiguration {
    /// Initializes the logger with the specified configuration.
    pub fn init(&self) -> Result<(), SetLoggerError> {
        if let Some(ref file_path) = self.file {
            let file = File::create(file_path).unwrap();
            WriteLogger::init(self.level, Config::default(), file)
        } else {
            SimpleLogger::init(self.level, Config::default())
        }
    }
}

impl Default for LoggingConfiguration {
    fn default() -> Self {
        LoggingConfiguration {
            level: LevelFilter::Info,
            file: None,
        }
    }
}
