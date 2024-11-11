// src/logging/formatters.rs

use chrono::Local;
use log::Record;
use std::fmt::Write;

/// Formats log messages with a timestamp and level.
pub fn format_log(record: &Record) -> String {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let mut buffer = String::new();
    let _ = write!(
        buffer,
        "{} [{}] - {}",
        timestamp,
        record.level(),
        record.args()
    );
    buffer
}
