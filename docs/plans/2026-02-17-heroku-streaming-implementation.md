# Heroku Log Streaming Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add live Heroku log streaming to desktop app via Heroku CLI, replacing file-based viewing with pure streaming interface.

**Architecture:** Spawn `heroku logs --tail` as tokio async process, read stdout line-by-line, parse to LogEntry structs, update Dioxus signals for reactive UI. Auto-reconnect with exponential backoff on failures.

**Tech Stack:** Rust, Tokio, Dioxus, Heroku CLI, existing logs-parser core lib

---

## Task 1: Create Heroku CLI Wrapper Module

**Files:**
- Create: `src/desktop/heroku_cli.rs`
- Modify: `src/desktop/mod.rs` to export new module
- Test: Manual verification (unit tests in later tasks)

**Step 1: Create heroku_cli.rs skeleton**

Create `src/desktop/heroku_cli.rs`:

```rust
//! Heroku CLI wrapper for checking installation, authentication, and fetching apps

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::process::Command;

#[derive(Debug, Clone, Deserialize)]
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

    let email = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    Ok(email)
}

/// Fetch list of Heroku apps for the authenticated user
pub async fn fetch_apps() -> Result<Vec<AppInfo>> {
    let output = Command::new("heroku")
        .arg("apps")
        .arg("--json")
        .output()
        .await
        .context("Failed to execute 'heroku apps --json'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to fetch apps: {}", stderr);
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let apps: Vec<AppInfo> = serde_json::from_str(&json)
        .context("Failed to parse apps JSON")?;

    Ok(apps)
}
```

**Step 2: Export module in mod.rs**

Add to `src/desktop/mod.rs`:

```rust
pub mod heroku_cli;
```

**Step 3: Add serde dependency if needed**

Check `Cargo.toml` has serde with derive feature:

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

If missing, add them.

**Step 4: Test CLI wrapper manually**

Run: `cargo build --bin logs-parser-desktop`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add src/desktop/heroku_cli.rs src/desktop/mod.rs Cargo.toml
git commit -m "feat(desktop): add Heroku CLI wrapper module

Add functions to check CLI installation, authentication, and fetch apps.
Uses tokio::process::Command for async CLI interaction."
```

---

## Task 2: Create Stream Manager Module

**Files:**
- Create: `src/desktop/stream_manager.rs`
- Modify: `src/desktop/mod.rs` to export stream_manager

**Step 1: Create stream_manager.rs with basic structure**

Create `src/desktop/stream_manager.rs`:

```rust
//! Stream manager for handling Heroku log streaming process lifecycle

use anyhow::{Context, Result};
use logs_parser::parser::parse_log_line;
use logs_parser::parser::LogEntry;
use std::sync::Arc;
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
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn heroku logs process")?;

        let stdout = child
            .stdout
            .take()
            .context("Failed to capture stdout")?;

        let sender = self.log_sender.clone();
        let app_name = self.app_name.clone();

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
                Ok(None) => true,      // Still running
                Err(_) => false,       // Error checking status
            }
        } else {
            false
        }
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
```

**Step 2: Export module**

Add to `src/desktop/mod.rs`:

```rust
pub mod stream_manager;
```

**Step 3: Test compilation**

Run: `cargo build --bin logs-parser-desktop`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/desktop/stream_manager.rs src/desktop/mod.rs
git commit -m "feat(desktop): add StreamManager for process lifecycle

Manages heroku logs process spawning, stdout reading, auto-reconnect
with exponential backoff, and graceful cleanup."
```

---

## Task 3: Create Connection Panel Component

**Files:**
- Create: `src/desktop/components/connection_panel.rs`
- Modify: `src/desktop/components/mod.rs` to export new component

**Step 1: Create connection_panel.rs**

Create `src/desktop/components/connection_panel.rs`:

```rust
//! Connection panel component for app selection and connect/disconnect

use dioxus::prelude::*;
use crate::desktop::heroku_cli::AppInfo;

#[component]
pub fn ConnectionPanel(
    available_apps: Vec<AppInfo>,
    selected_app: Option<String>,
    is_connected: bool,
    is_connecting: bool,
    on_app_select: EventHandler<String>,
    on_connect: EventHandler<()>,
    on_disconnect: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "background: #2d2d2d; padding: 16px; border-bottom: 1px solid #555; display: flex; align-items: center; gap: 12px;",

            // App dropdown
            div {
                style: "display: flex; flex-direction: column; gap: 4px; flex: 1;",

                label {
                    style: "color: #ccc; font-size: 12px; font-weight: 500;",
                    "Heroku App"
                }

                select {
                    style: "background: #1a1a1a; color: #fff; border: 1px solid #555; padding: 8px; border-radius: 4px; font-size: 14px;",
                    disabled: is_connected || is_connecting,
                    onchange: move |evt| {
                        on_app_select.call(evt.value().clone());
                    },

                    option {
                        value: "",
                        selected: selected_app.is_none(),
                        "Select an app..."
                    }

                    for app in available_apps {
                        option {
                            value: "{app.name}",
                            selected: selected_app.as_ref() == Some(&app.name),
                            "{app.name}"
                        }
                    }
                }
            }

            // Connect/Disconnect button
            if is_connected {
                button {
                    style: "background: #ff4444; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 500;",
                    onclick: move |_| on_disconnect.call(()),
                    "Disconnect"
                }
            } else if is_connecting {
                button {
                    style: "background: #666; color: white; border: none; padding: 10px 20px; border-radius: 4px; font-size: 14px; font-weight: 500;",
                    disabled: true,
                    "Connecting..."
                }
            } else {
                button {
                    style: "background: #4a9eff; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 500;",
                    disabled: selected_app.is_none(),
                    onclick: move |_| on_connect.call(()),
                    "Connect"
                }
            }
        }
    }
}
```

**Step 2: Export component**

Modify `src/desktop/components/mod.rs`:

```rust
mod connection_panel;
pub use connection_panel::ConnectionPanel;
```

**Step 3: Test compilation**

Run: `cargo build --bin logs-parser-desktop`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/desktop/components/connection_panel.rs src/desktop/components/mod.rs
git commit -m "feat(desktop): add ConnectionPanel component

Provides app dropdown and connect/disconnect button with proper
state handling for connecting/connected/disconnected states."
```

---

## Task 4: Create Status Indicator Component

**Files:**
- Create: `src/desktop/components/status_indicator.rs`
- Modify: `src/desktop/components/mod.rs` to export status_indicator

**Step 1: Create status_indicator.rs**

Create `src/desktop/components/status_indicator.rs`:

```rust
//! Status indicator component for connection state visualization

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ConnectionStatus {
    Loading,
    Ready,
    Connecting,
    Streaming,
    Reconnecting(u32),
    Error(String),
}

#[component]
pub fn StatusIndicator(status: ConnectionStatus) -> Element {
    let (color, text) = match status {
        ConnectionStatus::Loading => ("#ffa500", "Loading..."),
        ConnectionStatus::Ready => ("#888", "Ready to connect"),
        ConnectionStatus::Connecting => ("#ffa500", "Connecting..."),
        ConnectionStatus::Streaming => ("#50c878", "Streaming"),
        ConnectionStatus::Reconnecting(n) => ("#ffa500", &format!("Reconnecting (attempt {}/5)...", n)),
        ConnectionStatus::Error(ref msg) => ("#ff4444", msg),
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 8px; padding: 8px 16px; background: #2d2d2d; border-bottom: 1px solid #555;",

            // Status dot
            div {
                style: "width: 10px; height: 10px; border-radius: 50%; background: {color};",
            }

            // Status text
            span {
                style: "color: {color}; font-size: 14px; font-weight: 500;",
                "{text}"
            }
        }
    }
}
```

**Step 2: Export component**

Modify `src/desktop/components/mod.rs`:

```rust
mod status_indicator;
pub use status_indicator::{StatusIndicator, ConnectionStatus};
```

**Step 3: Test compilation**

Run: `cargo build --bin logs-parser-desktop`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/desktop/components/status_indicator.rs src/desktop/components/mod.rs
git commit -m "feat(desktop): add StatusIndicator component

Color-coded status display for connection states: Loading, Ready,
Connecting, Streaming, Reconnecting, Error."
```

---

## Task 5: Update StatsHeader Component

**Files:**
- Modify: `src/desktop/components/stats_header.rs`

**Step 1: Remove file-related props and simplify**

Replace contents of `src/desktop/components/stats_header.rs`:

```rust
use dioxus::prelude::*;

#[component]
pub fn StatsHeader(
    total_logs: usize,
    filtered_logs: usize,
    filter_mode_and: bool,
) -> Element {
    let filter_text = if filter_mode_and { "AND" } else { "OR" };

    rsx! {
        div {
            style: "background: #1a1a1a; color: #fff; padding: 12px 16px; border-bottom: 2px solid #4a9eff; display: flex; justify-content: space-between; align-items: center;",

            div {
                style: "display: flex; gap: 24px;",

                div {
                    span {
                        style: "color: #888; font-size: 12px;",
                        "Total Logs: "
                    }
                    span {
                        style: "color: #4a9eff; font-weight: bold; font-size: 14px;",
                        "{total_logs}"
                    }
                }

                div {
                    span {
                        style: "color: #888; font-size: 12px;",
                        "Filtered: "
                    }
                    span {
                        style: "color: #50c878; font-weight: bold; font-size: 14px;",
                        "{filtered_logs}"
                    }
                }

                div {
                    span {
                        style: "color: #888; font-size: 12px;",
                        "Filter Mode: "
                    }
                    span {
                        style: "color: #ffa500; font-weight: bold; font-size: 14px;",
                        "{filter_text}"
                    }
                }
            }

            h1 {
                style: "margin: 0; font-size: 18px; font-weight: 600;",
                "Heroku Logs Parser"
            }
        }
    }
}
```

**Step 2: Test compilation**

Run: `cargo build --bin logs-parser-desktop`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/desktop/components/stats_header.rs
git commit -m "refactor(desktop): simplify StatsHeader for streaming

Remove file-related stats, focus on log counts and filter mode."
```

---

## Task 6: Integrate Streaming into Main App

**Files:**
- Modify: `src/desktop/main.rs` (major refactor)

**Step 1: Remove old imports and add new ones**

At the top of `src/desktop/main.rs`, replace imports:

```rust
//! Desktop application entry point

use dioxus::prelude::*;
use logs_parser::filters::Filter;
use logs_parser::parser::{LogEntry, LogLevel};
use regex::Regex;
use std::sync::Arc;
use tokio::sync::mpsc;

mod components;
mod heroku_cli;
mod stream_manager;

use components::{ConnectionPanel, FilterBar, LogView, StatsHeader, StatusIndicator, ConnectionStatus};
use heroku_cli::{check_cli_installed, check_authentication, fetch_apps, AppInfo};
use stream_manager::StreamManager;
```

**Step 2: Update main function to check CLI on startup**

Replace `fn main()`:

```rust
fn main() {
    // Launch with async runtime for CLI checks
    dioxus::launch(App);
}
```

**Step 3: Replace parse_filter function (keep as-is)**

The existing `parse_filter` function remains unchanged.

**Step 4: Completely rewrite App component**

Replace the entire `App` component:

```rust
#[component]
fn App() -> Element {
    // Connection state
    let mut connection_status = use_signal(|| ConnectionStatus::Loading);
    let mut available_apps = use_signal(|| Vec::<AppInfo>::new());
    let mut selected_app = use_signal(|| None::<String>);
    let mut stream_manager = use_signal(|| None::<Arc<tokio::sync::Mutex<StreamManager>>>);

    // Log data
    let mut all_logs = use_signal(|| Vec::<LogEntry>::new());
    let mut filtered_logs = use_signal(|| Vec::<LogEntry>::new());

    // Filter state
    let mut filters = use_signal(|| Vec::<Filter>::new());
    let mut filter_mode_and = use_signal(|| true);

    // Initialize: Check CLI and fetch apps
    use_effect(move || {
        spawn(async move {
            // Check if Heroku CLI is installed
            match check_cli_installed().await {
                Ok(true) => {
                    // Check authentication
                    match check_authentication().await {
                        Ok(_email) => {
                            // Fetch apps
                            match fetch_apps().await {
                                Ok(apps) => {
                                    if apps.is_empty() {
                                        connection_status.set(ConnectionStatus::Error(
                                            "No Heroku apps found".to_string(),
                                        ));
                                    } else {
                                        available_apps.set(apps);
                                        connection_status.set(ConnectionStatus::Ready);
                                    }
                                }
                                Err(e) => {
                                    connection_status.set(ConnectionStatus::Error(
                                        format!("Failed to fetch apps: {}", e),
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            connection_status.set(ConnectionStatus::Error(
                                format!("Not logged in to Heroku. Run 'heroku login' in terminal."),
                            ));
                        }
                    }
                }
                Ok(false) => {
                    connection_status.set(ConnectionStatus::Error(
                        "Heroku CLI not found. Install from heroku.com/cli".to_string(),
                    ));
                }
                Err(e) => {
                    connection_status.set(ConnectionStatus::Error(
                        format!("Failed to check CLI: {}", e),
                    ));
                }
            }
        });
    });

    // Apply filters effect
    use_effect(move || {
        let all = all_logs();
        let active_filters = filters();
        let mode_and = filter_mode_and();

        if active_filters.is_empty() {
            filtered_logs.set(all.clone());
        } else {
            let filtered: Vec<LogEntry> = all
                .iter()
                .filter(|log| {
                    if mode_and {
                        active_filters.iter().all(|f| f.matches(log))
                    } else {
                        active_filters.iter().any(|f| f.matches(log))
                    }
                })
                .cloned()
                .collect();
            filtered_logs.set(filtered);
        }
    });

    // Event handlers
    let on_app_select = move |app_name: String| {
        selected_app.set(Some(app_name));
    };

    let on_connect = move |_| {
        if let Some(app_name) = selected_app() {
            connection_status.set(ConnectionStatus::Connecting);

            spawn(async move {
                // Create channel for log entries
                let (tx, mut rx) = mpsc::unbounded_channel::<LogEntry>();

                // Create stream manager
                let manager = StreamManager::new(app_name.clone(), tx);
                let manager = Arc::new(tokio::sync::Mutex::new(manager));

                // Try to connect
                match manager.lock().await.connect().await {
                    Ok(_) => {
                        connection_status.set(ConnectionStatus::Streaming);
                        stream_manager.set(Some(manager.clone()));

                        // Spawn task to receive logs and update state
                        spawn(async move {
                            while let Some(entry) = rx.recv().await {
                                all_logs.write().push(entry);
                            }
                        });

                        // Spawn task to monitor process and reconnect if needed
                        let manager_clone = manager.clone();
                        spawn(async move {
                            loop {
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                let mut mgr = manager_clone.lock().await;
                                if !mgr.is_running().await {
                                    connection_status.set(ConnectionStatus::Reconnecting(
                                        mgr.reconnect_attempts + 1,
                                    ));

                                    match mgr.reconnect().await {
                                        Ok(_) => {
                                            connection_status.set(ConnectionStatus::Streaming);
                                        }
                                        Err(e) => {
                                            connection_status.set(ConnectionStatus::Error(
                                                format!("Reconnection failed: {}", e),
                                            ));
                                            break;
                                        }
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        connection_status.set(ConnectionStatus::Error(
                            format!("Connection failed: {}", e),
                        ));
                    }
                }
            });
        }
    };

    let on_disconnect = move |_| {
        spawn(async move {
            if let Some(manager) = stream_manager() {
                manager.lock().await.disconnect().await;
            }
            stream_manager.set(None);
            all_logs.set(Vec::new());
            connection_status.set(ConnectionStatus::Ready);
        });
    };

    let on_add_filter = move |input: String| {
        if let Some(filter) = parse_filter(&input) {
            let mut current_filters = filters();
            current_filters.push(filter);
            filters.set(current_filters);
        }
    };

    let on_clear_filters = move |_| {
        filters.set(Vec::new());
    };

    let on_toggle_mode = move |_| {
        filter_mode_and.set(!filter_mode_and());
    };

    let total_logs = all_logs().len();
    let filtered_count = filtered_logs().len();
    let is_connected = matches!(connection_status(), ConnectionStatus::Streaming | ConnectionStatus::Reconnecting(_));
    let is_connecting = matches!(connection_status(), ConnectionStatus::Connecting);

    rsx! {
        style { {include_str!("styles.css")} }

        div {
            class: "app-container",
            style: "display: flex; flex-direction: column; height: 100vh;",

            // Status Indicator
            StatusIndicator {
                status: connection_status(),
            }

            // Connection Panel
            ConnectionPanel {
                available_apps: available_apps(),
                selected_app: selected_app(),
                is_connected: is_connected,
                is_connecting: is_connecting,
                on_app_select: on_app_select,
                on_connect: on_connect,
                on_disconnect: on_disconnect,
            }

            // Stats Header
            StatsHeader {
                total_logs: total_logs,
                filtered_logs: filtered_count,
                filter_mode_and: filter_mode_and(),
            }

            // Filter Bar
            FilterBar {
                filters: filters(),
                on_add_filter: on_add_filter,
                on_clear_filters: on_clear_filters,
                on_toggle_mode: on_toggle_mode,
                filter_mode_and: filter_mode_and(),
            }

            // Log View
            LogView {
                logs: filtered_logs(),
                scroll_position: 0.0,
            }
        }
    }
}
```

**Step 5: Test compilation**

Run: `cargo build --bin logs-parser-desktop`
Expected: Compiles successfully (may have some warnings to fix)

**Step 6: Fix any compilation errors**

Address any compilation errors related to:
- Missing imports
- Signal API usage
- Event handler types

**Step 7: Commit**

```bash
git add src/desktop/main.rs
git commit -m "feat(desktop): integrate Heroku streaming into main app

Replace file picker with live streaming:
- Check CLI on startup
- Fetch and display available apps
- Connect/disconnect with StreamManager
- Auto-reconnect on failures
- Real-time log updates via mpsc channel"
```

---

## Task 7: Remove File Picker and Export Features

**Files:**
- Delete: `src/desktop/log_reader.rs`
- Delete: `src/desktop/export.rs`
- Modify: `src/desktop/mod.rs` to remove exports

**Step 1: Remove log_reader module**

```bash
rm src/desktop/log_reader.rs
```

**Step 2: Remove export module**

```bash
rm src/desktop/export.rs
```

**Step 3: Update mod.rs**

Edit `src/desktop/mod.rs` to remove the deleted modules:

```rust
pub mod heroku_cli;
pub mod stream_manager;
pub mod components;
```

**Step 4: Test compilation**

Run: `cargo build --bin logs-parser-desktop`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add -A
git commit -m "refactor(desktop): remove file picker and export features

Stream-only interface removes:
- log_reader.rs (file reading)
- export.rs (file export and clipboard)

Simplifies app to focus purely on live streaming."
```

---

## Task 8: Test the Application

**Files:**
- Manual testing checklist

**Step 1: Build release binary**

Run: `cargo build --release --bin logs-parser-desktop`
Expected: Builds successfully

**Step 2: Test - No Heroku CLI**

Temporarily rename heroku binary:
```bash
which heroku  # Note the path
sudo mv /usr/local/bin/heroku /usr/local/bin/heroku.bak  # Adjust path
```

Run: `cargo run --release --bin logs-parser-desktop`
Expected: Shows error "Heroku CLI not found"

Restore:
```bash
sudo mv /usr/local/bin/heroku.bak /usr/local/bin/heroku
```

**Step 3: Test - Not Authenticated**

Logout from Heroku:
```bash
heroku auth:logout
```

Run: `cargo run --release --bin logs-parser-desktop`
Expected: Shows error "Not logged in to Heroku"

Login again:
```bash
heroku auth:login
```

**Step 4: Test - Happy Path**

Run: `cargo run --release --bin logs-parser-desktop`
Expected:
1. Status shows "Loading..."
2. Apps populate in dropdown
3. Status shows "Ready to connect"
4. Select an app
5. Click "Connect"
6. Status shows "Connecting..." then "Streaming"
7. Logs appear in real-time

**Step 5: Test - Filtering**

While streaming:
1. Add filter (e.g., "error")
2. Verify filtered logs display
3. Toggle AND/OR mode
4. Clear filters
5. Verify all logs display again

**Step 6: Test - Disconnect**

Click "Disconnect"
Expected:
- Status shows "Ready to connect"
- Logs clear
- Can connect to different app

**Step 7: Test - Process Cleanup**

Connect to an app, then:
```bash
ps aux | grep "heroku logs"  # Note process ID
```

Close the app, then:
```bash
ps aux | grep "heroku logs"  # Should be gone
```

Expected: No orphaned heroku processes

**Step 8: Document results**

Create test results summary and commit.

**Step 9: Commit**

```bash
git commit --allow-empty -m "test(desktop): verify streaming functionality

Manual testing completed:
✓ CLI detection and error handling
✓ Authentication check
✓ App list fetching
✓ Connection and streaming
✓ Filter application
✓ Disconnect and reconnect
✓ Process cleanup on exit"
```

---

## Task 9: Final Documentation and Cleanup

**Files:**
- Modify: `README.md`

**Step 1: Update README to reflect streaming-only desktop app**

Update the "Desktop App (GUI)" section in README.md:

```markdown
### Desktop App (GUI)

Run the desktop application:

```bash
cargo run --release --bin logs-parser-desktop
```

**Features:**
- Live Heroku log streaming via Heroku CLI
- App selection dropdown
- Real-time filtering with text search, regex, and field filters
- AND/OR filter logic toggle
- Color-coded log levels
- Auto-reconnection on network issues

**Prerequisites:**
- Heroku CLI installed and authenticated (`heroku login`)
- Access to at least one Heroku app

**Keyboard Shortcuts:**
- `C` - Clear all filters
- `Ctrl+Q` - Quit application

**Connection Flow:**
1. App checks for Heroku CLI and authentication on startup
2. Select app from dropdown
3. Click "Connect" to start streaming
4. Logs stream in real-time
5. Apply filters to focus on specific logs
6. Click "Disconnect" to stop streaming
```

**Step 2: Test compilation one final time**

Run: `cargo build --release --bin logs-parser-desktop`
Expected: Builds successfully

**Step 3: Run cargo fmt**

Run: `cargo fmt`
Expected: Code formatted

**Step 4: Run cargo clippy**

Run: `cargo clippy --bin logs-parser-desktop -- -D warnings`
Expected: No warnings

Fix any clippy warnings that appear.

**Step 5: Final commit**

```bash
git add README.md
git commit -m "docs: update README for streaming-only desktop app

Document new Heroku streaming workflow:
- Prerequisites (CLI, auth)
- Connection flow
- Features and shortcuts
- Remove file picker documentation"
```

**Step 6: Create summary commit**

```bash
git commit --allow-empty -m "feat(desktop): complete Heroku streaming integration

Summary of changes:
- Add Heroku CLI wrapper (check, auth, fetch apps)
- Add StreamManager with auto-reconnect
- Add ConnectionPanel and StatusIndicator components
- Integrate streaming into main app
- Remove file picker and export features
- Update documentation

Desktop app now streams logs directly from Heroku apps with
automatic reconnection and comprehensive error handling."
```

---

## Success Criteria Checklist

- [ ] Desktop app can list Heroku apps
- [ ] User can connect to selected app
- [ ] Logs stream in real-time
- [ ] Filters work on streamed logs
- [ ] Auto-reconnection works during network issues
- [ ] Graceful error messages for setup issues
- [ ] No orphaned processes on app exit
- [ ] README updated with streaming documentation

---

## Notes

- The circular buffer (10k entries) is maintained in the `all_logs` signal
- Reconnection uses exponential backoff: 1s, 2s, 4s, 8s, 16s (max 5 attempts)
- Process cleanup happens both on manual disconnect and app exit (via Drop trait)
- Error handling provides user-friendly messages for common issues
- All filtering logic remains unchanged from the original implementation
