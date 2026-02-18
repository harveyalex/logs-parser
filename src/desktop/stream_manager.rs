//! Stream manager for handling Heroku log streaming process lifecycle

use anyhow::{Context, Result};
use logs_parser::parser::parse_log_line;
use logs_parser::parser::LogEntry;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub struct StreamManager {
    app_name: String,
    process: Option<Child>,
    log_sender: mpsc::UnboundedSender<LogEntry>,
    reconnect_attempts: u32,
}

impl StreamManager {
    pub fn new(app_name: String, log_sender: mpsc::UnboundedSender<LogEntry>) -> Self {
        Self {
            app_name,
            process: None,
            log_sender,
            reconnect_attempts: 0,
        }
    }

    /// Connect to Heroku app and start streaming logs
    pub async fn connect(&mut self) -> Result<()> {
        // Kill existing process if any
        self.disconnect().await;

        // Spawn heroku logs process
        let mut child = Command::new("heroku")
            .arg("logs")
            .arg("--tail")
            .arg("--app")
            .arg(&self.app_name)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("Failed to spawn heroku logs process")?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;

        let sender = self.log_sender.clone();

        // Spawn task to read stdout line by line
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(entry) = parse_log_line(&line) {
                    if sender.send(entry).is_err() {
                        break;
                    }
                }
            }
        });

        self.process = Some(child);
        self.reconnect_attempts = 0;

        Ok(())
    }

    /// Disconnect and kill the process
    pub async fn disconnect(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill().await;
        }
        self.reconnect_attempts = 0;
    }

    /// Reconnect with exponential backoff
    pub async fn reconnect(&mut self) -> Result<()> {
        self.reconnect_attempts += 1;

        if self.reconnect_attempts > 5 {
            anyhow::bail!("Max reconnection attempts reached (5)");
        }

        // Exponential backoff: 1s, 2s, 4s, 8s, 16s
        let delay = 2_u64.pow(self.reconnect_attempts - 1);
        sleep(Duration::from_secs(delay)).await;

        self.connect().await
    }

    /// Check if process is still running
    pub async fn is_running(&mut self) -> bool {
        if let Some(process) = &mut self.process {
            match process.try_wait() {
                Ok(Some(_)) => false, // Process exited
                Ok(None) => true,     // Still running
                Err(_) => false,      // Error checking status
            }
        } else {
            false
        }
    }

    /// Get the current reconnect attempt count
    pub fn get_reconnect_attempts(&self) -> u32 {
        self.reconnect_attempts
    }
}

impl Drop for StreamManager {
    fn drop(&mut self) {
        // Best effort cleanup - kill process if still running
        if let Some(mut process) = self.process.take() {
            let _ = process.start_kill();
        }
    }
}
