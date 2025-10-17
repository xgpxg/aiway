mod quickwit_appender;
pub use quickwit_appender::QuickwitAppender;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LogEntry {
    time: String,
    service: String,
    level: String,
    message: String,
}
/// 支持的日志格式：2025-01-01 00:00:00.000 INFO message
impl From<Vec<u8>> for LogEntry {
    fn from(value: Vec<u8>) -> Self {
        let value = String::from_utf8(value).unwrap();
        let time = &value[0..23];
        let level = &value[24..29].trim();
        let message = &value[30..].trim();
        Self {
            time: time.to_string(),
            service: "gateway".to_string(),
            level: level.to_string(),
            message: message.to_string(),
        }
    }
}
