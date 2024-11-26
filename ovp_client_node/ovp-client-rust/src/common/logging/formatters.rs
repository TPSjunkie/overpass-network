// ./ovp-client/src/common/logging/formatters.rs

use chrono::Local;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct CustomFormatter {
    use_color: bool,
    use_utc: bool,
}

impl CustomFormatter {
    pub fn new(use_color: bool, use_utc: bool) -> Self {
        Self { use_color, use_utc }
    }
}

impl log::Log for CustomFormatter {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut output = Vec::new();
        let now = Local::now();
        let now_utc = now.with_timezone(&chrono::Utc);
        let now_local = now.with_timezone(&chrono::Local);

        if self.use_color {
            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            stdout.set_color(&self.color_spec(record.level())).unwrap();

            if self.use_utc {
                write!(
                    &mut output,
                    "{} {}",
                    now_utc.format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.args()
                )
                .unwrap();
            } else {
                write!(
                    &mut output,
                    "{} {}",
                    now_local.format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.args()
                )
                .unwrap();
            }

            stdout.reset().unwrap();
        } else {
            if self.use_utc {
                write!(
                    &mut output,
                    "{} {}",
                    now_utc.format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.args()
                )
                .unwrap();
            } else {
                write!(
                    &mut output,
                    "{} {}",
                    now_local.format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.args()
                )
                .unwrap();
            }
        }

        println!("{}", String::from_utf8(output).unwrap());
    }

    fn flush(&self) {}
}
impl CustomFormatter {
    fn color_spec(&self, level: log::Level) -> ColorSpec {
        let mut spec = ColorSpec::new();
        match level {
            log::Level::Error => spec.set_fg(Some(Color::Red)),
            log::Level::Warn => spec.set_fg(Some(Color::Yellow)),
            log::Level::Info => spec.set_fg(Some(Color::Green)),
            log::Level::Debug => spec.set_fg(Some(Color::Blue)),
            log::Level::Trace => spec.set_fg(Some(Color::Magenta)),
        };
        spec
    }
}
