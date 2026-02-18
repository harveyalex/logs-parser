//! Heroku CLI wrapper for checking installation, authentication, and fetching apps

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::process::Command;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AppInfo {
    pub name: String,
    pub id: String,
}

/// Check if Heroku CLI is installed
pub async fn check_cli_installed() -> Result<bool> {
    let output = Command::new("which")
        .arg("heroku")
        .output()
        .await
        .context("Failed to execute 'which' command")?;

    Ok(output.status.success() && !output.stdout.is_empty())
}

/// Check if user is authenticated with Heroku
/// Returns the authenticated user's email
pub async fn check_authentication() -> Result<String> {
    let output = Command::new("heroku")
        .arg("auth:whoami")
        .output()
        .await
        .context("Failed to execute 'heroku auth:whoami'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Not authenticated: {}", stderr);
    }

    let email = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(email)
}

/// Fetch list of Heroku apps for the authenticated user
pub async fn fetch_apps() -> Result<Vec<AppInfo>> {
    let output = Command::new("heroku")
        .arg("apps")
        .arg("--all")
        .arg("--json")
        .output()
        .await
        .context("Failed to execute 'heroku apps --all --json'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to fetch apps: {}", stderr);
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let apps: Vec<AppInfo> = serde_json::from_str(&json).context("Failed to parse apps JSON")?;

    Ok(apps)
}

/// Spawn the interactive Heroku login flow.
/// Opens the user's browser for OAuth. Returns the child process immediately
/// — the caller is responsible for waiting on it.
pub fn spawn_login() -> Result<tokio::process::Child> {
    Command::new("heroku")
        .arg("login")
        .spawn()
        .context("Failed to spawn 'heroku login'")
}
