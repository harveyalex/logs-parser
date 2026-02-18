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

/// Find the heroku binary, returning an absolute path.
///
/// Strategy:
/// 1. Check well-known install locations directly (fast, no subprocess).
/// 2. Run `/usr/bin/which heroku` with an augmented PATH — uses the absolute
///    path to `which` so we don't need PATH resolution to find `which` itself.
/// 3. Fall back to "heroku" and hope the OS can resolve it.
pub fn find_heroku_binary() -> String {
    // 1. Static probe — covers Homebrew (Apple Silicon + Intel) and old Toolbelt.
    for path in &[
        "/opt/homebrew/bin/heroku",
        "/usr/local/bin/heroku",
        "/usr/local/heroku/bin/heroku",
    ] {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }

    // 2. Ask `which` via its absolute path so the lookup is PATH-independent.
    if let Ok(output) = std::process::Command::new("/usr/bin/which")
        .env("PATH", gui_path())
        .arg("heroku")
        .output()
    {
        if output.status.success() {
            let found = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            if !found.is_empty() {
                return found;
            }
        }
    }

    "heroku".to_string()
}

/// Returns a Command for the heroku binary with an augmented PATH.
fn heroku_cmd() -> Command {
    let mut c = Command::new(find_heroku_binary());
    c.env("PATH", gui_path());
    c
}

/// Check if Heroku CLI is installed.
/// Uses find_heroku_binary() — if it resolves to anything other than the bare
/// "heroku" fallback, we know heroku is present.
pub async fn check_cli_installed() -> Result<bool> {
    let binary = find_heroku_binary();
    if binary != "heroku" {
        return Ok(true);
    }
    // Last resort: try spawning heroku and see if it runs.
    Ok(heroku_cmd()
        .arg("version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false))
}

/// Check if user is authenticated with Heroku
/// Returns the authenticated user's email
pub async fn check_authentication() -> Result<String> {
    let output = heroku_cmd()
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
    let output = heroku_cmd()
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
    heroku_cmd()
        .arg("login")
        .spawn()
        .context("Failed to spawn 'heroku login'")
}
