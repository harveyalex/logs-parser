//! Log file reading functionality

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use logs_parser::parser::{parse_log_line, LogEntry};

/// Read and parse a log file from disk
///
/// Opens the file, reads it line by line, and parses each line using parse_log_line.
/// Lines that cannot be parsed are skipped.
///
/// # Arguments
/// * `path` - Path to the log file to read
///
/// # Returns
/// * `Ok(Vec<LogEntry>)` - Vector of successfully parsed log entries
/// * `Err` - If file cannot be opened or read
pub async fn read_log_file(path: PathBuf) -> Result<Vec<LogEntry>> {
    let file = File::open(&path)
        .await
        .context(format!("Failed to open log file: {:?}", path))?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut log_entries = Vec::new();

    while let Some(line) = lines
        .next_line()
        .await
        .context("Failed to read line from file")?
    {
        // Parse the line and add to collection if valid
        if let Some(entry) = parse_log_line(&line) {
            log_entries.push(entry);
        }
    }

    Ok(log_entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_read_log_file() {
        // Create a temporary file
        let temp_path = PathBuf::from("/tmp/test_logs.log");
        let mut file = File::create(&temp_path).await.unwrap();

        // Write sample log lines
        let sample_logs = "2024-02-17T10:30:45.123456+00:00 app[web.1]: Starting application\n\
                           2024-02-17T10:30:46.234567+00:00 heroku[router]: at=info method=GET path=/\n\
                           2024-02-17T10:30:47.345678+00:00 app[web.1]: Error: Connection timeout\n";

        file.write_all(sample_logs.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        // Read and parse the file
        let entries = read_log_file(temp_path.clone()).await.unwrap();

        // Verify results
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].source, "app");
        assert_eq!(entries[1].source, "heroku");
        assert_eq!(entries[2].message, "Error: Connection timeout");

        // Clean up
        let _ = tokio::fs::remove_file(temp_path).await;
    }

    #[tokio::test]
    async fn test_read_log_file_with_invalid_lines() {
        let temp_path = PathBuf::from("/tmp/test_logs_invalid.log");
        let mut file = File::create(&temp_path).await.unwrap();

        // Mix of valid and invalid lines
        let sample_logs = "2024-02-17T10:30:45.123456+00:00 app[web.1]: Valid log\n\
                           This is not a valid log line\n\
                           2024-02-17T10:30:46.234567+00:00 heroku[router]: Another valid log\n\
                           Another invalid line\n";

        file.write_all(sample_logs.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        // Read and parse the file
        let entries = read_log_file(temp_path.clone()).await.unwrap();

        // Should only have the 2 valid entries
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].message, "Valid log");
        assert_eq!(entries[1].message, "Another valid log");

        // Clean up
        let _ = tokio::fs::remove_file(temp_path).await;
    }
}
