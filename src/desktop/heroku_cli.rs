//! Heroku CLI wrapper for checking installation, authentication, and fetching apps

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::process::Command;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AppInfo {
    pub name: String,
    pub id: String,
}

/// Build a PATH string that includes common macOS install locations.
/// GUI apps launched from the Dock/Finder don't inherit the user's shell PATH,
/// so Homebrew-installed binaries (/opt/homebrew/bin, /usr/local/bin) are missing.
fn gui_path() -> String {
    let base = std::env::var("PATH").unwrap_or_default();
    format!("/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:{}", base)
}

/// Returns a Command with an augmented PATH so GUI apps can find Homebrew binaries.
fn cmd(program: &str) -> Command {
    let mut c = Command::new(program);
    c.env("PATH", gui_path());
    c
}

/// Check if Heroku CLI is installed
pub async fn check_cli_installed() -> Result<bool> {
    let output = cmd("which")
        .arg("heroku")
        .output()
        .await
        .context("Failed to execute 'which' command")?;

    Ok(output.status.success() && !output.stdout.is_empty())
}

/// Check if user is authenticated with Heroku
/// Returns the authenticated user's email
pub async fn check_authentication() -> Result<String> {
    let output = cmd("heroku")
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
    let output = cmd("heroku")
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
    let mut c = Command::new("heroku");
    c.env("PATH", gui_path());
    c.arg("login")
        .spawn()
        .context("Failed to spawn 'heroku login'")
}
