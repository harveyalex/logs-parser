use crate::parser::{LogEntry, LogLevel};
use chrono::{DateTime, FixedOffset};
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
    /// Filter by time range
    TimeRange {
        start: Option<DateTime<FixedOffset>>,
        end: Option<DateTime<FixedOffset>>,
    },
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
            (Filter::TimeRange { start: s1, end: e1 }, Filter::TimeRange { start: s2, end: e2 }) => {
                s1 == s2 && e1 == e2
            }
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
            Filter::TimeRange { start, end } => {
                let after_start = start.map_or(true, |s| entry.timestamp >= s);
                let before_end = end.map_or(true, |e| entry.timestamp <= e);
                after_start && before_end
            }
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
            Filter::TimeRange { start, end } => {
                let start_str = start
                    .map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "start".to_string());
                let end_str = end
                    .map(|e| e.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "end".to_string());
                format!("Time: {} to {}", start_str, end_str)
            }
        }
    }
}

/// Mode for combining multiple filters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    /// All filters must match (AND logic)
    And,
    /// At least one filter must match (OR logic)
    Or,
}

/// Engine for applying multiple filters to log entries
#[derive(Debug)]
pub struct FilterEngine {
    filters: Vec<Filter>,
    mode: FilterMode,
}

impl FilterEngine {
    /// Create a new filter engine with AND mode
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            mode: FilterMode::And,
        }
    }

    /// Add a filter to the engine
    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    /// Remove a filter at the specified index
    pub fn remove_filter(&mut self, index: usize) -> Option<Filter> {
        if index < self.filters.len() {
            Some(self.filters.remove(index))
        } else {
            None
        }
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.filters.clear();
    }

    /// Get the number of active filters
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Check if there are any active filters
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Get the current filter mode
    pub fn mode(&self) -> FilterMode {
        self.mode
    }

    /// Set the filter mode
    pub fn set_mode(&mut self, mode: FilterMode) {
        self.mode = mode;
    }

    /// Toggle between AND and OR modes
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            FilterMode::And => FilterMode::Or,
            FilterMode::Or => FilterMode::And,
        };
    }

    /// Check if a log entry matches the active filters
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if self.filters.is_empty() {
            return true;
        }

        match self.mode {
            FilterMode::And => self.filters.iter().all(|f| f.matches(entry)),
            FilterMode::Or => self.filters.iter().any(|f| f.matches(entry)),
        }
    }

    /// Filter a collection of log entries and return indices of matching entries
    pub fn filter_indices(&self, entries: &[&LogEntry]) -> Vec<usize> {
        entries
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| {
                if self.matches(entry) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all active filters
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }
}

impl Default for FilterEngine {
    fn default() -> Self {
        Self::new()
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
    fn test_filter_engine_and_mode() {
        let mut engine = FilterEngine::new();
        engine.add_filter(Filter::TextSearch("error".to_string()));
        engine.add_filter(Filter::Source("app".to_string()));
        engine.set_mode(FilterMode::And);

        let entry1 = create_test_entry("error message");
        let entry2 = parse_log_line(
            "2010-09-16T15:13:46.677020+00:00 heroku[router]: error message",
        )
        .unwrap();
        let entry3 = create_test_entry("normal message");

        assert!(engine.matches(&entry1)); // Has "error" and source is "app"
        assert!(!engine.matches(&entry2)); // Has "error" but source is "heroku"
        assert!(!engine.matches(&entry3)); // No "error"
    }

    #[test]
    fn test_filter_engine_or_mode() {
        let mut engine = FilterEngine::new();
        engine.add_filter(Filter::TextSearch("error".to_string()));
        engine.add_filter(Filter::Source("heroku".to_string()));
        engine.set_mode(FilterMode::Or);

        let entry1 = create_test_entry("error message");
        let entry2 = parse_log_line(
            "2010-09-16T15:13:46.677020+00:00 heroku[router]: normal message",
        )
        .unwrap();
        let entry3 = create_test_entry("normal message");

        assert!(engine.matches(&entry1)); // Has "error"
        assert!(engine.matches(&entry2)); // Source is "heroku"
        assert!(!engine.matches(&entry3)); // Neither condition
    }

    #[test]
    fn test_filter_engine_no_filters() {
        let engine = FilterEngine::new();
        let entry = create_test_entry("any message");

        assert!(engine.matches(&entry)); // No filters = match all
    }

    #[test]
    fn test_filter_engine_toggle_mode() {
        let mut engine = FilterEngine::new();
        assert_eq!(engine.mode(), FilterMode::And);

        engine.toggle_mode();
        assert_eq!(engine.mode(), FilterMode::Or);

        engine.toggle_mode();
        assert_eq!(engine.mode(), FilterMode::And);
    }

    #[test]
    fn test_filter_engine_clear() {
        let mut engine = FilterEngine::new();
        engine.add_filter(Filter::TextSearch("test".to_string()));
        engine.add_filter(Filter::Source("app".to_string()));

        assert_eq!(engine.len(), 2);

        engine.clear();
        assert_eq!(engine.len(), 0);
        assert!(engine.is_empty());
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
