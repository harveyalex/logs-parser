use crate::parser::LogEntry;
use anyhow::Result;
use arboard::Clipboard;
use chrono::Local;
use std::fs::File;
use std::io::Write;

/// Copy filtered logs to clipboard
pub fn copy_to_clipboard(logs: &[&LogEntry]) -> Result<String> {
    if logs.is_empty() {
        return Ok("No logs to copy".to_string());
    }

    let content = format_logs_for_export(logs);

    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(&content)?;

    Ok(format!("Copied {} log entries to clipboard", logs.len()))
}

/// Export filtered logs to a timestamped file
pub fn export_to_file(logs: &[&LogEntry]) -> Result<String> {
    if logs.is_empty() {
        return Ok("No logs to export".to_string());
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("heroku_logs_{}.log", timestamp);

    let content = format_logs_for_export(logs);

    let mut file = File::create(&filename)?;
    file.write_all(content.as_bytes())?;

    Ok(format!(
        "Exported {} log entries to {}",
        logs.len(),
        filename
    ))
}

/// Format logs for export (uses raw format to preserve original)
fn format_logs_for_export(logs: &[&LogEntry]) -> String {
    logs.iter()
        .map(|log| log.raw.as_str())
        .collect::<Vec<_>>()
        .join("\n")
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
    fn test_format_logs_for_export() {
        let entry1 = create_test_entry("Message 1");
        let entry2 = create_test_entry("Message 2");
        let logs = vec![&entry1, &entry2];

        let content = format_logs_for_export(&logs);

        assert!(content.contains("Message 1"));
        assert!(content.contains("Message 2"));
        assert!(content.contains("app[web.1]"));
    }

    #[test]
    fn test_format_empty_logs() {
        let logs: Vec<&LogEntry> = vec![];
        let content = format_logs_for_export(&logs);
        assert_eq!(content, "");
    }

    #[test]
    fn test_export_to_file_empty() {
        let logs: Vec<&LogEntry> = vec![];
        let result = export_to_file(&logs);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No logs to export");
    }

    // Note: Clipboard tests are skipped as they require a display environment
    // and would fail in CI/headless environments
}
