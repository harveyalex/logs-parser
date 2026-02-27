use crate::parser::{LogEntry, LogLevel};
use regex::Regex;

/// Represents different types of filters that can be applied to log entries
#[derive(Debug, Clone)]
pub enum Filter {
    /// Case-insensitive text search in the message
    TextSearch(String),
    /// Regex pattern matching in the message
    Regex(Regex),
    /// Filter by dyno name (e.g., "web.1", "worker.3")
    Dyno(String),
    /// Filter by source (e.g., "app", "heroku", "heroku-router")
    Source(String),
    /// Filter by log level
    LogLevel(LogLevel),
}

// Implement PartialEq manually since Regex doesn't implement PartialEq
impl PartialEq for Filter {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Filter::TextSearch(a), Filter::TextSearch(b)) => a == b,
            (Filter::Regex(a), Filter::Regex(b)) => a.as_str() == b.as_str(),
            (Filter::Dyno(a), Filter::Dyno(b)) => a == b,
            (Filter::Source(a), Filter::Source(b)) => a == b,
            (Filter::LogLevel(a), Filter::LogLevel(b)) => a == b,
            _ => false,
        }
    }
}

impl Filter {
    /// Check if a log entry matches this filter
    pub fn matches(&self, entry: &LogEntry) -> bool {
        match self {
            Filter::TextSearch(text) => {
                let lower_text = text.to_lowercase();
                let lower_message = entry.message.to_lowercase();
                lower_message.contains(&lower_text)
            }
            Filter::Regex(regex) => regex.is_match(&entry.message),
            Filter::Dyno(dyno) => entry.dyno.eq_ignore_ascii_case(dyno),
            Filter::Source(source) => entry.source.eq_ignore_ascii_case(source),
            Filter::LogLevel(level) => entry.level == *level,
        }
    }

    /// Get a display string for this filter
    pub fn display(&self) -> String {
        match self {
            Filter::TextSearch(text) => format!("Text: \"{}\"", text),
            Filter::Regex(regex) => format!("Regex: /{}/", regex.as_str()),
            Filter::Dyno(dyno) => format!("Dyno: {}", dyno),
            Filter::Source(source) => format!("Source: {}", source),
            Filter::LogLevel(level) => format!("Level: {:?}", level),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_log_line;

    fn create_test_entry(message: &str) -> LogEntry {
        let line = format!(
            "2010-09-16T15:13:46.677020+00:00 app[web.1]: {}",
            message
        );
        parse_log_line(&line).expect("Failed to parse test entry")
    }

    #[test]
    fn test_text_search_filter() {
        let filter = Filter::TextSearch("error".to_string());
        let entry1 = create_test_entry("This is an error message");
        let entry2 = create_test_entry("This is a normal message");

        assert!(filter.matches(&entry1));
        assert!(!filter.matches(&entry2));
    }

    #[test]
    fn test_text_search_case_insensitive() {
        let filter = Filter::TextSearch("ERROR".to_string());
        let entry = create_test_entry("This is an error message");

        assert!(filter.matches(&entry));
    }

    #[test]
    fn test_regex_filter() {
        let regex = Regex::new(r"error \d+").unwrap();
        let filter = Filter::Regex(regex);
        let entry1 = create_test_entry("error 404 not found");
        let entry2 = create_test_entry("error message");

        assert!(filter.matches(&entry1));
        assert!(!filter.matches(&entry2));
    }

    #[test]
    fn test_dyno_filter() {
        let filter = Filter::Dyno("web.1".to_string());
        let entry1 = parse_log_line(
            "2010-09-16T15:13:46.677020+00:00 app[web.1]: Test message",
        )
        .unwrap();
        let entry2 = parse_log_line(
            "2010-09-16T15:13:46.677020+00:00 app[worker.3]: Test message",
        )
        .unwrap();

        assert!(filter.matches(&entry1));
        assert!(!filter.matches(&entry2));
    }

    #[test]
    fn test_source_filter() {
        let filter = Filter::Source("heroku".to_string());
        let entry1 = parse_log_line(
            "2010-09-16T15:13:46.677020+00:00 heroku[router]: Test message",
        )
        .unwrap();
        let entry2 = parse_log_line(
            "2010-09-16T15:13:46.677020+00:00 app[web.1]: Test message",
        )
        .unwrap();

        assert!(filter.matches(&entry1));
        assert!(!filter.matches(&entry2));
    }

    #[test]
    fn test_log_level_filter() {
        let filter = Filter::LogLevel(LogLevel::Error);
        let entry1 = create_test_entry("Error: Connection failed");
        let entry2 = create_test_entry("Info: Processing request");

        assert!(filter.matches(&entry1));
        assert!(!filter.matches(&entry2));
    }

    #[test]
    fn test_filter_display() {
        let filter1 = Filter::TextSearch("error".to_string());
        assert_eq!(filter1.display(), "Text: \"error\"");

        let filter2 = Filter::Dyno("web.1".to_string());
        assert_eq!(filter2.display(), "Dyno: web.1");

        let filter3 = Filter::Source("app".to_string());
        assert_eq!(filter3.display(), "Source: app");
    }
}
