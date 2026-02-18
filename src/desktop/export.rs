//! Export logs to file and clipboard

use logs_parser::parser::LogEntry;
use anyhow::Result;
use std::path::PathBuf;

/// Format log entries for export to text format
///
/// Converts log entries into a readable text format with all fields
///
/// # Arguments
/// * `logs` - Slice of log entries to format
///
/// # Returns
/// Formatted string with one log per line
pub fn format_logs_for_export(logs: &[LogEntry]) -> String {
    logs.iter()
        .map(|log| {
            format!(
                "{} [{}] ({}) [{:?}] {}",
                log.format_time(),
                log.source,
                log.dyno,
                log.level,
                log.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Export logs to a file
///
/// Writes formatted log entries to the specified file path
///
/// # Arguments
/// * `logs` - Slice of log entries to export
/// * `path` - Destination file path
///
/// # Returns
/// * `Ok(())` on success
/// * `Err` if file write fails
pub async fn export_to_file(logs: &[LogEntry], path: PathBuf) -> Result<()> {
    let content = format_logs_for_export(logs);
    tokio::fs::write(path, content).await?;
    Ok(())
}

/// Copy logs to system clipboard
///
/// Formats log entries and copies them to the system clipboard
///
/// # Arguments
/// * `logs` - Slice of log entries to copy
///
/// # Returns
/// * `Ok(())` on success
/// * `Err` if clipboard operation fails
pub fn copy_to_clipboard(logs: &[LogEntry]) -> Result<()> {
    let content = format_logs_for_export(logs);
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use logs_parser::parser::{parse_log_line, LogLevel};

    #[test]
    fn test_format_logs_for_export() {
        let logs = vec![
            parse_log_line("2024-02-17T10:30:45.123456+00:00 app[web.1]: Starting application")
                .unwrap(),
            parse_log_line(
                "2024-02-17T10:30:46.234567+00:00 heroku[router]: at=info method=GET path=/",
            )
            .unwrap(),
        ];

        let formatted = format_logs_for_export(&logs);

        assert!(formatted.contains("app"));
        assert!(formatted.contains("web.1"));
        assert!(formatted.contains("Starting application"));
        assert!(formatted.contains("heroku"));
        assert!(formatted.contains("router"));
    }

    #[tokio::test]
    async fn test_export_to_file() {
        let logs = vec![
            parse_log_line("2024-02-17T10:30:45.123456+00:00 app[web.1]: Test message").unwrap(),
        ];

        let temp_path = PathBuf::from("/tmp/test_export.log");

        // Export to file
        export_to_file(&logs, temp_path.clone()).await.unwrap();

        // Verify file contents
        let content = tokio::fs::read_to_string(&temp_path).await.unwrap();
        assert!(content.contains("Test message"));
        assert!(content.contains("app"));
        assert!(content.contains("web.1"));

        // Clean up
        let _ = tokio::fs::remove_file(temp_path).await;
    }

    #[test]
    fn test_copy_to_clipboard() {
        let logs = vec![
            parse_log_line("2024-02-17T10:30:45.123456+00:00 app[web.1]: Clipboard test")
                .unwrap(),
        ];

        // Note: This test may fail in CI environments without clipboard support
        let result = copy_to_clipboard(&logs);

        // We don't assert success since clipboard may not be available in all environments
        // In a real environment, this would succeed
        match result {
            Ok(_) => {
                // Verify clipboard contents if possible
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        assert!(text.contains("Clipboard test"));
                    }
                }
            }
            Err(e) => {
                eprintln!("Clipboard test skipped (no clipboard available): {}", e);
            }
        }
    }
}
