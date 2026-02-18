use chrono::{DateTime, FixedOffset};
use regex::Regex;
use std::sync::OnceLock;

/// Represents a log level extracted from message content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Unknown,
}

impl LogLevel {
    /// Detect log level from message content by looking for keywords
    fn from_message(message: &str) -> Self {
        let lower = message.to_lowercase();
        if lower.contains("error") || lower.contains("fatal") || lower.contains("panic") {
            LogLevel::Error
        } else if lower.contains("warn") || lower.contains("warning") {
            LogLevel::Warn
        } else if lower.contains("debug") || lower.contains("trace") {
            LogLevel::Debug
        } else if lower.contains("info") {
            LogLevel::Info
        } else {
            LogLevel::Unknown
        }
    }
}

/// Represents a parsed Heroku log entry
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub source: String,
    pub dyno: String,
    pub message: String,
    pub level: LogLevel,
    pub raw: String,
}

impl LogEntry {
    /// Get a formatted display string for the log entry
    pub fn format_display(&self) -> String {
        format!(
            "{} {} [{}] {}",
            self.timestamp.format("%H:%M:%S%.3f"),
            self.source,
            self.dyno,
            self.message
        )
    }

    /// Get just the time portion formatted
    pub fn format_time(&self) -> String {
        self.timestamp.format("%H:%M:%S%.3f").to_string()
    }
}

/// Get the regex pattern for parsing Heroku logs
fn log_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        // Pattern: timestamp source[dyno]: message
        // Example: 2010-09-16T15:13:46.677020+00:00 app[web.1]: Starting process
        Regex::new(
            r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+[+-]\d{2}:\d{2})\s+(\w+)\[([^\]]+)\]:\s*(.*)$"
        )
        .expect("Failed to compile log regex")
    })
}

/// Parse a single Heroku log line
///
/// Returns `Some(LogEntry)` if the line matches the expected format,
/// or `None` if the line cannot be parsed.
///
/// # Example
/// ```
/// use logs_parser::parser::parse_log_line;
///
/// let line = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Starting process";
/// let entry = parse_log_line(line).expect("Failed to parse");
/// assert_eq!(entry.source, "app");
/// assert_eq!(entry.dyno, "web.1");
/// ```
pub fn parse_log_line(line: &str) -> Option<LogEntry> {
    let regex = log_regex();
    let captures = regex.captures(line)?;

    // Extract timestamp
    let timestamp_str = captures.get(1)?.as_str();
    let timestamp = DateTime::parse_from_rfc3339(timestamp_str).ok()?;

    // Extract other fields
    let source = captures.get(2)?.as_str().to_string();
    let dyno = captures.get(3)?.as_str().to_string();
    let message = captures.get(4)?.as_str().to_string();

    // Detect log level from message content
    let level = LogLevel::from_message(&message);

    Some(LogEntry {
        timestamp,
        source,
        dyno,
        message,
        level,
        raw: line.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_basic_log() {
        let line = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Starting process";
        let entry = parse_log_line(line).expect("Failed to parse");

        assert_eq!(entry.source, "app");
        assert_eq!(entry.dyno, "web.1");
        assert_eq!(entry.message, "Starting process");
        assert_eq!(entry.level, LogLevel::Unknown);
    }

    #[test]
    fn test_parse_with_error_level() {
        let line = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Error: Connection failed";
        let entry = parse_log_line(line).expect("Failed to parse");

        assert_eq!(entry.level, LogLevel::Error);
        assert!(entry.message.contains("Error"));
    }

    #[test]
    fn test_parse_with_warning_level() {
        let line = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Warning: Low memory";
        let entry = parse_log_line(line).expect("Failed to parse");

        assert_eq!(entry.level, LogLevel::Warn);
    }

    #[test]
    fn test_parse_heroku_router() {
        let line = "2010-09-16T15:13:46.677020+00:00 heroku[router]: at=info method=GET path=/";
        let entry = parse_log_line(line).expect("Failed to parse");

        assert_eq!(entry.source, "heroku");
        assert_eq!(entry.dyno, "router");
        assert_eq!(entry.level, LogLevel::Info);
    }

    #[test]
    fn test_parse_worker_dyno() {
        let line = "2010-09-16T15:13:46.677020+00:00 app[worker.3]: Processing job 12345";
        let entry = parse_log_line(line).expect("Failed to parse");

        assert_eq!(entry.source, "app");
        assert_eq!(entry.dyno, "worker.3");
        assert_eq!(entry.message, "Processing job 12345");
    }

    #[test]
    fn test_parse_invalid_line() {
        let line = "This is not a valid Heroku log line";
        let result = parse_log_line(line);

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_with_debug_level() {
        let line = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Debug: Checking configuration";
        let entry = parse_log_line(line).expect("Failed to parse");

        assert_eq!(entry.level, LogLevel::Debug);
    }

    #[test]
    fn test_format_display() {
        let line = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Test message";
        let entry = parse_log_line(line).expect("Failed to parse");

        let display = entry.format_display();
        assert!(display.contains("15:13:46"));
        assert!(display.contains("app"));
        assert!(display.contains("[web.1]"));
        assert!(display.contains("Test message"));
    }

    #[test]
    fn test_timestamp_parsing() {
        let line = "2024-02-17T10:30:45.123456+00:00 app[web.1]: Test";
        let entry = parse_log_line(line).expect("Failed to parse");

        assert_eq!(entry.timestamp.hour(), 10);
        assert_eq!(entry.timestamp.minute(), 30);
        assert_eq!(entry.timestamp.second(), 45);
    }
}
