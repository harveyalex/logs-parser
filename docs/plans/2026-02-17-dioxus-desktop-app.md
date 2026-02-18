# Dioxus Desktop App Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Convert the existing TUI logs parser into a native desktop application using Dioxus while reusing core parsing, filtering, and state management logic.

**Architecture:** Build a Dioxus desktop UI that replaces the ratatui terminal interface. The desktop app will run as a native window with the same functionality (log viewing, filtering, export). Core modules (parser, filters, buffer) remain unchanged. The app state will be adapted to Dioxus's reactive model while preserving the TEA pattern principles.

**Tech Stack:** Rust, Dioxus (desktop), Tokio (async runtime), existing parser/filter/buffer modules

---

## Task 1: Setup Dioxus Desktop Dependencies

**Files:**
- Modify: `Cargo.toml:1-16`

**Step 1: Add Dioxus dependencies**

Edit `Cargo.toml` to add Dioxus desktop dependencies:

```toml
[package]
name = "logs-parser"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "logs-parser-tui"
path = "src/main.rs"

[[bin]]
name = "logs-parser-desktop"
path = "src/desktop/main.rs"

[dependencies]
# TUI dependencies
ratatui = "0.26"
crossterm = { version = "0.27", features = ["event-stream"] }

# Desktop dependencies
dioxus = "0.6"
dioxus-desktop = "0.6"

# Shared dependencies
tokio = { version = "1", features = ["full"] }
futures = "0.3"
regex = "1"
chrono = "0.4"
arboard = "3"
anyhow = "1"
clap = { version = "4", features = ["derive"] }
```

**Step 2: Verify dependencies compile**

Run: `cargo check`
Expected: SUCCESS - all dependencies resolve

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "build: add dioxus desktop dependencies and separate binaries"
```

---

## Task 2: Create Desktop Module Structure

**Files:**
- Create: `src/desktop/mod.rs`
- Create: `src/desktop/main.rs`
- Create: `src/desktop/components/mod.rs`
- Create: `src/desktop/state.rs`

**Step 1: Create desktop directory structure**

Run: `mkdir -p src/desktop/components`
Expected: Directories created

**Step 2: Create desktop mod.rs**

Create `src/desktop/mod.rs`:

```rust
//! Desktop UI module for Dioxus-based GUI

pub mod state;
pub mod components;
```

**Step 3: Create main entry point**

Create `src/desktop/main.rs`:

```rust
//! Desktop application entry point

use dioxus::prelude::*;

fn main() {
    dioxus_desktop::launch(App);
}

fn App(cx: Scope) -> Element {
    render! {
        div {
            class: "container",
            h1 { "Heroku Logs Parser - Desktop" }
            p { "Desktop app starting..." }
        }
    }
}
```

**Step 4: Create components module**

Create `src/desktop/components/mod.rs`:

```rust
//! UI components for desktop app

pub mod log_view;
pub mod filter_bar;
pub mod stats_header;
```

**Step 5: Create state module**

Create `src/desktop/state.rs`:

```rust
//! Application state for desktop UI

use crate::{LogEntry, Buffer, Filter};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct DesktopAppState {
    pub logs: Arc<RwLock<Buffer<LogEntry>>>,
    pub filters: Arc<RwLock<Vec<Filter>>>,
    pub paused: Arc<RwLock<bool>>,
    pub filter_mode_and: Arc<RwLock<bool>>,
    pub scroll_position: Arc<RwLock<usize>>,
}

impl DesktopAppState {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Buffer::new(10_000))),
            filters: Arc::new(RwLock::new(Vec::new())),
            paused: Arc::new(RwLock::new(false)),
            filter_mode_and: Arc::new(RwLock::new(true)),
            scroll_position: Arc::new(RwLock::new(0)),
        }
    }
}
```

**Step 6: Verify structure compiles**

Run: `cargo check --bin logs-parser-desktop`
Expected: SUCCESS

**Step 7: Run desktop app to verify**

Run: `cargo run --bin logs-parser-desktop`
Expected: Window opens showing "Heroku Logs Parser - Desktop"

**Step 8: Commit**

```bash
git add src/desktop/
git commit -m "feat: create desktop module structure with basic app"
```

---

## Task 3: Create Stats Header Component

**Files:**
- Create: `src/desktop/components/stats_header.rs`
- Modify: `src/desktop/components/mod.rs:5`

**Step 1: Write the stats header component**

Create `src/desktop/components/stats_header.rs`:

```rust
//! Header component showing log statistics

use dioxus::prelude::*;

#[derive(Props)]
pub struct StatsHeaderProps<'a> {
    pub total_logs: usize,
    pub filtered_logs: usize,
    pub paused: bool,
    pub filter_mode_and: bool,
}

pub fn StatsHeader<'a>(cx: Scope<'a, StatsHeaderProps<'a>>) -> Element {
    let pause_text = if cx.props.paused { "PAUSED" } else { "LIVE" };
    let filter_mode = if cx.props.filter_mode_and { "AND" } else { "OR" };

    render! {
        div {
            class: "stats-header",
            style: "background: #2d2d2d; color: #ffffff; padding: 10px; display: flex; justify-content: space-between;",

            div {
                style: "font-weight: bold;",
                "Heroku Logs Parser"
            }

            div {
                style: "display: flex; gap: 20px;",
                span { "Total: {cx.props.total_logs}" }
                span { "Filtered: {cx.props.filtered_logs}" }
                span {
                    style: if cx.props.paused { "color: #ff6b6b;" } else { "color: #51cf66;" },
                    "{pause_text}"
                }
                span { "Filter Mode: {filter_mode}" }
            }
        }
    }
}
```

**Step 2: Export component from mod.rs**

Modify `src/desktop/components/mod.rs`:

```rust
//! UI components for desktop app

pub mod log_view;
pub mod filter_bar;
pub mod stats_header;

pub use stats_header::StatsHeader;
```

**Step 3: Verify component compiles**

Run: `cargo check --bin logs-parser-desktop`
Expected: SUCCESS

**Step 4: Commit**

```bash
git add src/desktop/components/stats_header.rs src/desktop/components/mod.rs
git commit -m "feat: add stats header component"
```

---

## Task 4: Create Filter Bar Component

**Files:**
- Create: `src/desktop/components/filter_bar.rs`
- Modify: `src/desktop/components/mod.rs:6`

**Step 1: Write the filter bar component**

Create `src/desktop/components/filter_bar.rs`:

```rust
//! Filter input and display component

use dioxus::prelude::*;
use crate::Filter;

#[derive(Props)]
pub struct FilterBarProps<'a> {
    pub filters: &'a Vec<Filter>,
    pub on_add_filter: EventHandler<'a, String>,
    pub on_clear_filters: EventHandler<'a, ()>,
    pub on_toggle_mode: EventHandler<'a, ()>,
}

pub fn FilterBar<'a>(cx: Scope<'a, FilterBarProps<'a>>) -> Element {
    let filter_input = use_state(cx, || String::new());

    render! {
        div {
            class: "filter-bar",
            style: "background: #3d3d3d; color: #ffffff; padding: 10px; display: flex; gap: 10px; align-items: center;",

            input {
                r#type: "text",
                placeholder: "Enter filter (text, regex, or field:value)",
                value: "{filter_input}",
                style: "flex: 1; padding: 5px; background: #2d2d2d; color: #ffffff; border: 1px solid #555; border-radius: 3px;",
                oninput: move |evt| filter_input.set(evt.value.clone()),
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter && !filter_input.is_empty() {
                        cx.props.on_add_filter.call(filter_input.get().clone());
                        filter_input.set(String::new());
                    }
                },
            }

            button {
                style: "padding: 5px 15px; background: #4a9eff; color: white; border: none; border-radius: 3px; cursor: pointer;",
                onclick: move |_| {
                    if !filter_input.is_empty() {
                        cx.props.on_add_filter.call(filter_input.get().clone());
                        filter_input.set(String::new());
                    }
                },
                "Add Filter"
            }

            button {
                style: "padding: 5px 15px; background: #ff6b6b; color: white; border: none; border-radius: 3px; cursor: pointer;",
                onclick: move |_| cx.props.on_clear_filters.call(()),
                "Clear"
            }

            button {
                style: "padding: 5px 15px; background: #51cf66; color: white; border: none; border-radius: 3px; cursor: pointer;",
                onclick: move |_| cx.props.on_toggle_mode.call(()),
                "Toggle AND/OR"
            }

            if !cx.props.filters.is_empty() {
                div {
                    style: "display: flex; gap: 5px; flex-wrap: wrap;",
                    for filter in cx.props.filters {
                        span {
                            style: "background: #555; padding: 3px 8px; border-radius: 3px; font-size: 12px;",
                            "{filter}"
                        }
                    }
                }
            }
        }
    }
}
```

**Step 2: Export component from mod.rs**

Modify `src/desktop/components/mod.rs`:

```rust
//! UI components for desktop app

pub mod log_view;
pub mod filter_bar;
pub mod stats_header;

pub use stats_header::StatsHeader;
pub use filter_bar::FilterBar;
```

**Step 3: Verify component compiles**

Run: `cargo check --bin logs-parser-desktop`
Expected: Compilation may fail due to missing Display trait for Filter - this is expected

**Step 4: Add Display trait to Filter**

Modify `src/filters.rs` to add Display implementation for Filter enum. Find the Filter enum and add:

```rust
impl std::fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Text(s) => write!(f, "text:{}", s),
            Filter::Regex(r) => write!(f, "regex:{}", r.as_str()),
            Filter::Dyno(d) => write!(f, "dyno:{}", d),
            Filter::Source(s) => write!(f, "source:{}", s),
            Filter::Level(l) => write!(f, "level:{:?}", l),
        }
    }
}
```

**Step 5: Verify now compiles**

Run: `cargo check --bin logs-parser-desktop`
Expected: SUCCESS

**Step 6: Commit**

```bash
git add src/desktop/components/filter_bar.rs src/desktop/components/mod.rs src/filters.rs
git commit -m "feat: add filter bar component with Display trait"
```

---

## Task 5: Create Log View Component

**Files:**
- Create: `src/desktop/components/log_view.rs`
- Modify: `src/desktop/components/mod.rs:7`

**Step 1: Write the log view component**

Create `src/desktop/components/log_view.rs`:

```rust
//! Log display component with scrolling

use dioxus::prelude::*;
use crate::LogEntry;

#[derive(Props)]
pub struct LogViewProps<'a> {
    pub logs: &'a Vec<LogEntry>,
    pub scroll_position: usize,
}

pub fn LogView<'a>(cx: Scope<'a, LogViewProps<'a>>) -> Element {
    render! {
        div {
            class: "log-view",
            style: "flex: 1; overflow-y: auto; background: #1e1e1e; color: #d4d4d4; font-family: 'Courier New', monospace; font-size: 13px; padding: 10px;",

            if cx.props.logs.is_empty() {
                div {
                    style: "text-align: center; padding: 50px; color: #888;",
                    "No logs to display. Waiting for log input..."
                }
            } else {
                for (idx, log) in cx.props.logs.iter().enumerate() {
                    div {
                        key: "{idx}",
                        style: "padding: 2px 0; {get_log_style(log)}",
                        span {
                            style: "color: #888;",
                            "{log.timestamp.format(\"%Y-%m-%d %H:%M:%S\")} "
                        }
                        span {
                            style: "color: #4a9eff;",
                            "[{log.source}] "
                        }
                        span {
                            style: "color: #ff79c6;",
                            "({log.dyno}) "
                        }
                        span {
                            style: get_level_color(&log.level),
                            "[{log.level:?}] "
                        }
                        span { "{log.message}" }
                    }
                }
            }
        }
    }
}

fn get_log_style(log: &LogEntry) -> &'static str {
    match log.level {
        crate::LogLevel::Error => "background: #3d1f1f;",
        _ => "",
    }
}

fn get_level_color(level: &crate::LogLevel) -> &'static str {
    match level {
        crate::LogLevel::Error => "color: #ff6b6b; font-weight: bold;",
        crate::LogLevel::Warn => "color: #ffd93d;",
        crate::LogLevel::Info => "color: #51cf66;",
        crate::LogLevel::Debug => "color: #4a9eff;",
    }
}
```

**Step 2: Export component from mod.rs**

Modify `src/desktop/components/mod.rs`:

```rust
//! UI components for desktop app

pub mod log_view;
pub mod filter_bar;
pub mod stats_header;

pub use stats_header::StatsHeader;
pub use filter_bar::FilterBar;
pub use log_view::LogView;
```

**Step 3: Verify component compiles**

Run: `cargo check --bin logs-parser-desktop`
Expected: SUCCESS

**Step 4: Commit**

```bash
git add src/desktop/components/log_view.rs src/desktop/components/mod.rs
git commit -m "feat: add log view component with syntax highlighting"
```

---

## Task 6: Integrate Components into Main App

**Files:**
- Modify: `src/desktop/main.rs:1-20`
- Modify: `src/desktop/state.rs:1-30`

**Step 1: Update main.rs with integrated UI**

Replace content of `src/desktop/main.rs`:

```rust
//! Desktop application entry point

#![allow(non_snake_case)]

use dioxus::prelude::*;
use logs_parser::desktop::{state::DesktopAppState, components::*};
use logs_parser::{Filter, FilterPredicate};

fn main() {
    dioxus_desktop::launch_cfg(
        App,
        dioxus_desktop::Config::new()
            .with_window(
                dioxus_desktop::WindowBuilder::new()
                    .with_title("Heroku Logs Parser")
                    .with_inner_size(dioxus_desktop::LogicalSize::new(1200.0, 800.0))
            )
    );
}

fn App(cx: Scope) -> Element {
    let app_state = use_ref(cx, DesktopAppState::new);
    let logs = use_state(cx, Vec::new);
    let filters = use_state(cx, Vec::new);
    let total_count = use_state(cx, || 0);
    let filtered_count = use_state(cx, || 0);
    let paused = use_state(cx, || false);
    let filter_mode_and = use_state(cx, || true);

    render! {
        style { include_str!("./styles.css") }

        div {
            class: "app-container",
            style: "display: flex; flex-direction: column; height: 100vh; background: #1e1e1e;",

            StatsHeader {
                total_logs: *total_count.get(),
                filtered_logs: *filtered_count.get(),
                paused: *paused.get(),
                filter_mode_and: *filter_mode_and.get(),
            }

            FilterBar {
                filters: filters.get(),
                on_add_filter: move |filter_str: String| {
                    let filter = parse_filter(&filter_str);
                    filters.modify(|f| {
                        let mut new_filters = f.clone();
                        new_filters.push(filter);
                        new_filters
                    });
                },
                on_clear_filters: move |_| {
                    filters.set(Vec::new());
                },
                on_toggle_mode: move |_| {
                    filter_mode_and.modify(|m| !*m);
                },
            }

            LogView {
                logs: logs.get(),
                scroll_position: 0,
            }

            div {
                class: "status-bar",
                style: "background: #2d2d2d; color: #888; padding: 5px 10px; font-size: 12px;",
                "Press Ctrl+O to open log file | Ctrl+S to export | Ctrl+Q to quit"
            }
        }
    }
}

fn parse_filter(s: &str) -> Filter {
    if let Some(dyno) = s.strip_prefix("dyno:") {
        Filter::Dyno(dyno.to_string())
    } else if let Some(source) = s.strip_prefix("source:") {
        Filter::Source(source.to_string())
    } else if let Some(level) = s.strip_prefix("level:") {
        use logs_parser::LogLevel;
        let level = match level.to_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warn" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            _ => LogLevel::Info,
        };
        Filter::Level(level)
    } else if s.starts_with('/') && s.ends_with('/') && s.len() > 2 {
        let pattern = &s[1..s.len()-1];
        if let Ok(regex) = regex::Regex::new(pattern) {
            Filter::Regex(regex)
        } else {
            Filter::Text(s.to_string())
        }
    } else {
        Filter::Text(s.to_string())
    }
}
```

**Step 2: Create styles.css**

Create `src/desktop/styles.css`:

```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.app-container {
    width: 100%;
    height: 100vh;
}
```

**Step 3: Verify app compiles**

Run: `cargo check --bin logs-parser-desktop`
Expected: SUCCESS

**Step 4: Run desktop app**

Run: `cargo run --bin logs-parser-desktop`
Expected: Window opens with header, filter bar, empty log view, and status bar

**Step 5: Commit**

```bash
git add src/desktop/main.rs src/desktop/styles.css
git commit -m "feat: integrate components into main desktop app"
```

---

## Task 7: Add Log Input via File Picker

**Files:**
- Modify: `src/desktop/main.rs:20-100`
- Create: `src/desktop/log_reader.rs`
- Modify: `src/desktop/mod.rs:4`

**Step 1: Add file dialog dependency**

Modify `Cargo.toml` dependencies:

```toml
# Desktop dependencies
dioxus = "0.6"
dioxus-desktop = "0.6"
rfd = "0.15"  # File picker
```

**Step 2: Create log reader module**

Create `src/desktop/log_reader.rs`:

```rust
//! Background log file reader

use crate::{LogEntry, parse_log_line};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use anyhow::Result;

pub async fn read_log_file(path: PathBuf) -> Result<Vec<LogEntry>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut logs = Vec::new();

    while let Some(line) = lines.next_line().await? {
        if let Some(entry) = parse_log_line(&line) {
            logs.push(entry);
        }
    }

    Ok(logs)
}

pub async fn tail_log_file(
    path: PathBuf,
    mut tx: tokio::sync::mpsc::Sender<LogEntry>
) -> Result<()> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if let Some(entry) = parse_log_line(&line) {
            let _ = tx.send(entry).await;
        }
    }

    Ok(())
}
```

**Step 3: Export log_reader from mod**

Modify `src/desktop/mod.rs`:

```rust
//! Desktop UI module for Dioxus-based GUI

pub mod state;
pub mod components;
pub mod log_reader;
```

**Step 4: Add file picker to main app**

Modify `src/desktop/main.rs` to add file picker button in status bar:

```rust
// In the App function, add this to the status bar:
div {
    class: "status-bar",
    style: "background: #2d2d2d; color: #888; padding: 5px 10px; font-size: 12px; display: flex; justify-content: space-between;",

    span { "Press Ctrl+O to open log file | Ctrl+S to export | Ctrl+Q to quit" }

    button {
        style: "padding: 3px 10px; background: #4a9eff; color: white; border: none; border-radius: 3px; cursor: pointer;",
        onclick: move |_| {
            cx.spawn({
                let logs = logs.to_owned();
                let total_count = total_count.to_owned();
                async move {
                    if let Some(path) = rfd::AsyncFileDialog::new()
                        .add_filter("logs", &["log", "txt"])
                        .pick_file()
                        .await
                    {
                        use logs_parser::desktop::log_reader::read_log_file;
                        if let Ok(new_logs) = read_log_file(path.path().to_path_buf()).await {
                            total_count.set(new_logs.len());
                            logs.set(new_logs);
                        }
                    }
                }
            });
        },
        "Open Log File"
    }
}
```

**Step 5: Verify compiles**

Run: `cargo check --bin logs-parser-desktop`
Expected: SUCCESS

**Step 6: Test file picker**

Run: `cargo run --bin logs-parser-desktop`
Expected: Click "Open Log File" button, file picker opens

**Step 7: Commit**

```bash
git add Cargo.toml src/desktop/log_reader.rs src/desktop/mod.rs src/desktop/main.rs
git commit -m "feat: add log file picker and reader"
```

---

## Task 8: Add Real-time Filtering

**Files:**
- Modify: `src/desktop/main.rs:40-90`

**Step 1: Add filter application logic**

Modify `src/desktop/main.rs` App function to apply filters when logs or filters change:

```rust
fn App(cx: Scope) -> Element {
    let app_state = use_ref(cx, DesktopAppState::new);
    let all_logs = use_state(cx, Vec::new);
    let filtered_logs = use_state(cx, Vec::new);
    let filters = use_state(cx, Vec::new);
    let total_count = use_state(cx, || 0);
    let paused = use_state(cx, || false);
    let filter_mode_and = use_state(cx, || true);

    // Apply filters whenever logs or filters change
    use_effect(cx, (all_logs, filters, filter_mode_and), |(logs, filters, mode_and)| {
        to_owned![filtered_logs];
        async move {
            let logs_vec = logs.get().clone();
            let filters_vec = filters.get().clone();
            let is_and = *mode_and.get();

            if filters_vec.is_empty() {
                filtered_logs.set(logs_vec);
            } else {
                use logs_parser::FilterPredicate;
                let filtered: Vec<_> = logs_vec.into_iter()
                    .filter(|log| {
                        if is_and {
                            filters_vec.iter().all(|f| f.matches(log))
                        } else {
                            filters_vec.iter().any(|f| f.matches(log))
                        }
                    })
                    .collect();
                filtered_logs.set(filtered);
            }
        }
    });

    let filtered_count = filtered_logs.len();

    render! {
        style { include_str!("./styles.css") }

        div {
            class: "app-container",
            style: "display: flex; flex-direction: column; height: 100vh; background: #1e1e1e;",

            StatsHeader {
                total_logs: *total_count.get(),
                filtered_logs: filtered_count,
                paused: *paused.get(),
                filter_mode_and: *filter_mode_and.get(),
            }

            FilterBar {
                filters: filters.get(),
                on_add_filter: move |filter_str: String| {
                    let filter = parse_filter(&filter_str);
                    filters.modify(|f| {
                        let mut new_filters = f.clone();
                        new_filters.push(filter);
                        new_filters
                    });
                },
                on_clear_filters: move |_| {
                    filters.set(Vec::new());
                },
                on_toggle_mode: move |_| {
                    filter_mode_and.modify(|m| !*m);
                },
            }

            LogView {
                logs: filtered_logs.get(),
                scroll_position: 0,
            }

            div {
                class: "status-bar",
                style: "background: #2d2d2d; color: #888; padding: 5px 10px; font-size: 12px; display: flex; justify-content: space-between;",

                span { "Press Ctrl+O to open log file | Ctrl+S to export | Ctrl+Q to quit" }

                button {
                    style: "padding: 3px 10px; background: #4a9eff; color: white; border: none; border-radius: 3px; cursor: pointer;",
                    onclick: move |_| {
                        cx.spawn({
                            let all_logs = all_logs.to_owned();
                            let total_count = total_count.to_owned();
                            async move {
                                if let Some(path) = rfd::AsyncFileDialog::new()
                                    .add_filter("logs", &["log", "txt"])
                                    .pick_file()
                                    .await
                                {
                                    use logs_parser::desktop::log_reader::read_log_file;
                                    if let Ok(new_logs) = read_log_file(path.path().to_path_buf()).await {
                                        total_count.set(new_logs.len());
                                        all_logs.set(new_logs);
                                    }
                                }
                            }
                        });
                    },
                    "Open Log File"
                }
            }
        }
    }
}
```

**Step 2: Verify compiles**

Run: `cargo check --bin logs-parser-desktop`
Expected: SUCCESS

**Step 3: Test filtering**

Run: `cargo run --bin logs-parser-desktop`
Then:
1. Open a log file
2. Add filter "error"
3. Verify only error logs show

Expected: Filtering works in real-time

**Step 4: Commit**

```bash
git add src/desktop/main.rs
git commit -m "feat: add real-time filtering with AND/OR logic"
```

---

## Task 9: Add Export Functionality

**Files:**
- Create: `src/desktop/export.rs`
- Modify: `src/desktop/mod.rs:5`
- Modify: `src/desktop/main.rs:110-140`

**Step 1: Create export module**

Create `src/desktop/export.rs`:

```rust
//! Export logs to file and clipboard

use crate::LogEntry;
use anyhow::Result;
use std::path::PathBuf;

pub fn format_logs_for_export(logs: &[LogEntry]) -> String {
    logs.iter()
        .map(|log| {
            format!(
                "{} [{}] ({}) [{:?}] {}",
                log.timestamp.format("%Y-%m-%d %H:%M:%S"),
                log.source,
                log.dyno,
                log.level,
                log.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn export_to_file(logs: &[LogEntry], path: PathBuf) -> Result<()> {
    let content = format_logs_for_export(logs);
    tokio::fs::write(path, content).await?;
    Ok(())
}

pub fn copy_to_clipboard(logs: &[LogEntry]) -> Result<()> {
    let content = format_logs_for_export(logs);
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(content)?;
    Ok(())
}
```

**Step 2: Export from mod.rs**

Modify `src/desktop/mod.rs`:

```rust
//! Desktop UI module for Dioxus-based GUI

pub mod state;
pub mod components;
pub mod log_reader;
pub mod export;
```

**Step 3: Add export buttons to status bar**

Modify `src/desktop/main.rs` status bar section:

```rust
div {
    class: "status-bar",
    style: "background: #2d2d2d; color: #888; padding: 5px 10px; font-size: 12px; display: flex; justify-content: space-between;",

    span { "Logs: {filtered_count} / {*total_count.get()}" }

    div {
        style: "display: flex; gap: 10px;",

        button {
            style: "padding: 3px 10px; background: #4a9eff; color: white; border: none; border-radius: 3px; cursor: pointer;",
            onclick: move |_| {
                cx.spawn({
                    let all_logs = all_logs.to_owned();
                    let total_count = total_count.to_owned();
                    async move {
                        if let Some(path) = rfd::AsyncFileDialog::new()
                            .add_filter("logs", &["log", "txt"])
                            .pick_file()
                            .await
                        {
                            use logs_parser::desktop::log_reader::read_log_file;
                            if let Ok(new_logs) = read_log_file(path.path().to_path_buf()).await {
                                total_count.set(new_logs.len());
                                all_logs.set(new_logs);
                            }
                        }
                    }
                });
            },
            "Open"
        }

        button {
            style: "padding: 3px 10px; background: #51cf66; color: white; border: none; border-radius: 3px; cursor: pointer;",
            onclick: move |_| {
                let logs = filtered_logs.get().clone();
                cx.spawn(async move {
                    use logs_parser::desktop::export::copy_to_clipboard;
                    let _ = copy_to_clipboard(&logs);
                });
            },
            "Copy"
        }

        button {
            style: "padding: 3px 10px; background: #ffd93d; color: black; border: none; border-radius: 3px; cursor: pointer;",
            onclick: move |_| {
                cx.spawn({
                    let logs = filtered_logs.to_owned();
                    async move {
                        if let Some(path) = rfd::AsyncFileDialog::new()
                            .add_filter("logs", &["log", "txt"])
                            .set_file_name(&format!("logs-export-{}.log", chrono::Local::now().format("%Y%m%d-%H%M%S")))
                            .save_file()
                            .await
                        {
                            use logs_parser::desktop::export::export_to_file;
                            let _ = export_to_file(logs.get(), path.path().to_path_buf()).await;
                        }
                    }
                });
            },
            "Export"
        }
    }
}
```

**Step 4: Verify compiles**

Run: `cargo check --bin logs-parser-desktop`
Expected: SUCCESS

**Step 5: Test export**

Run: `cargo run --bin logs-parser-desktop`
Then:
1. Open log file
2. Click "Copy" - verify clipboard has content
3. Click "Export" - verify file is saved

Expected: Both copy and export work

**Step 6: Commit**

```bash
git add src/desktop/export.rs src/desktop/mod.rs src/desktop/main.rs
git commit -m "feat: add export to file and clipboard functionality"
```

---

## Task 10: Add Keyboard Shortcuts

**Files:**
- Modify: `src/desktop/main.rs:15-30`

**Step 1: Add global event listener**

Modify `src/desktop/main.rs` App function to add keyboard shortcuts:

```rust
fn App(cx: Scope) -> Element {
    let app_state = use_ref(cx, DesktopAppState::new);
    let all_logs = use_state(cx, Vec::new);
    let filtered_logs = use_state(cx, Vec::new);
    let filters = use_state(cx, Vec::new);
    let total_count = use_state(cx, || 0);
    let paused = use_state(cx, || false);
    let filter_mode_and = use_state(cx, || true);

    // Apply filters whenever logs or filters change
    use_effect(cx, (all_logs, filters, filter_mode_and), |(logs, filters, mode_and)| {
        to_owned![filtered_logs];
        async move {
            let logs_vec = logs.get().clone();
            let filters_vec = filters.get().clone();
            let is_and = *mode_and.get();

            if filters_vec.is_empty() {
                filtered_logs.set(logs_vec);
            } else {
                use logs_parser::FilterPredicate;
                let filtered: Vec<_> = logs_vec.into_iter()
                    .filter(|log| {
                        if is_and {
                            filters_vec.iter().all(|f| f.matches(log))
                        } else {
                            filters_vec.iter().any(|f| f.matches(log))
                        }
                    })
                    .collect();
                filtered_logs.set(filtered);
            }
        }
    });

    let filtered_count = filtered_logs.len();

    render! {
        style { include_str!("./styles.css") }

        div {
            class: "app-container",
            style: "display: flex; flex-direction: column; height: 100vh; background: #1e1e1e;",
            tabindex: 0,

            // Global keyboard shortcuts
            onkeydown: move |evt| {
                match evt.key() {
                    Key::Character(c) if evt.modifiers().ctrl() => {
                        match c.as_str() {
                            "o" | "O" => {
                                // Open file
                                cx.spawn({
                                    let all_logs = all_logs.to_owned();
                                    let total_count = total_count.to_owned();
                                    async move {
                                        if let Some(path) = rfd::AsyncFileDialog::new()
                                            .add_filter("logs", &["log", "txt"])
                                            .pick_file()
                                            .await
                                        {
                                            use logs_parser::desktop::log_reader::read_log_file;
                                            if let Ok(new_logs) = read_log_file(path.path().to_path_buf()).await {
                                                total_count.set(new_logs.len());
                                                all_logs.set(new_logs);
                                            }
                                        }
                                    }
                                });
                            }
                            "s" | "S" => {
                                // Save/Export
                                cx.spawn({
                                    let logs = filtered_logs.to_owned();
                                    async move {
                                        if let Some(path) = rfd::AsyncFileDialog::new()
                                            .add_filter("logs", &["log", "txt"])
                                            .set_file_name(&format!("logs-export-{}.log", chrono::Local::now().format("%Y%m%d-%H%M%S")))
                                            .save_file()
                                            .await
                                        {
                                            use logs_parser::desktop::export::export_to_file;
                                            let _ = export_to_file(logs.get(), path.path().to_path_buf()).await;
                                        }
                                    }
                                });
                            }
                            "c" | "C" => {
                                // Copy to clipboard
                                let logs = filtered_logs.get().clone();
                                cx.spawn(async move {
                                    use logs_parser::desktop::export::copy_to_clipboard;
                                    let _ = copy_to_clipboard(&logs);
                                });
                            }
                            "q" | "Q" => {
                                // Quit
                                std::process::exit(0);
                            }
                            _ => {}
                        }
                    }
                    Key::Character(c) => {
                        match c.as_str() {
                            "c" => {
                                // Clear filters
                                filters.set(Vec::new());
                            }
                            "p" | " " => {
                                // Toggle pause
                                paused.modify(|p| !*p);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            },

            // Rest of UI...
            StatsHeader {
                total_logs: *total_count.get(),
                filtered_logs: filtered_count,
                paused: *paused.get(),
                filter_mode_and: *filter_mode_and.get(),
            }

            // ... rest of components
        }
    }
}
```

**Step 2: Update status bar help text**

Update the status bar to reflect keyboard shortcuts:

```rust
div {
    class: "status-bar",
    style: "background: #2d2d2d; color: #888; padding: 5px 10px; font-size: 12px; display: flex; justify-content: space-between;",

    span { "Ctrl+O: Open | Ctrl+C: Copy | Ctrl+S: Export | Ctrl+Q: Quit | C: Clear filters | P: Pause" }

    span { "Logs: {filtered_count} / {*total_count.get()}" }

    // ... buttons
}
```

**Step 3: Verify compiles**

Run: `cargo check --bin logs-parser-desktop`
Expected: SUCCESS

**Step 4: Test keyboard shortcuts**

Run: `cargo run --bin logs-parser-desktop`
Test: Ctrl+O, Ctrl+C, Ctrl+S, Ctrl+Q, C, P
Expected: All shortcuts work

**Step 5: Commit**

```bash
git add src/desktop/main.rs
git commit -m "feat: add keyboard shortcuts for common actions"
```

---

## Task 11: Update README and Documentation

**Files:**
- Modify: `README.md:55-65`

**Step 1: Add desktop app section to README**

Modify `README.md` after the Installation section:

```markdown
## Installation

```bash
# Build the TUI version
cargo build --release --bin logs-parser-tui

# Build the desktop version
cargo build --release --bin logs-parser-desktop

# Run tests
cargo test
```

## Usage

### Desktop App (GUI)

Run the desktop application:

```bash
cargo run --release --bin logs-parser-desktop
```

**Features:**
- Native window interface
- File picker to open log files
- Real-time filtering with text search, regex, and field filters
- AND/OR filter logic toggle
- Color-coded log levels
- Export filtered logs to file
- Copy filtered logs to clipboard

**Keyboard Shortcuts:**
- `Ctrl+O` - Open log file
- `Ctrl+C` - Copy filtered logs to clipboard
- `Ctrl+S` - Export filtered logs to file
- `Ctrl+Q` - Quit application
- `C` - Clear all filters
- `P` or `Space` - Pause/resume (placeholder for live streaming)

### Terminal App (TUI)

Pipe Heroku logs into the parser:
```

**Step 2: Verify changes**

Run: `cat README.md | grep -A 20 "Desktop App"`
Expected: Desktop app section is visible

**Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add desktop app usage to README"
```

---

## Task 12: Create Quick Start Script

**Files:**
- Create: `run_desktop.sh`

**Step 1: Create run script**

Create `run_desktop.sh`:

```bash
#!/bin/bash
set -e

echo "Building Dioxus Desktop App..."
cargo build --release --bin logs-parser-desktop

echo ""
echo "Starting desktop application..."
./target/release/logs-parser-desktop
```

**Step 2: Make executable**

Run: `chmod +x run_desktop.sh`
Expected: File is now executable

**Step 3: Test script**

Run: `./run_desktop.sh`
Expected: App builds and runs

**Step 4: Commit**

```bash
git add run_desktop.sh
git commit -m "chore: add desktop app run script"
```

---

## Summary

This plan creates a complete Dioxus desktop application that:

1. ✅ Reuses existing core logic (parser, filters, buffer)
2. ✅ Provides native desktop UI with file picker
3. ✅ Real-time filtering with AND/OR logic
4. ✅ Export to file and clipboard
5. ✅ Keyboard shortcuts for power users
6. ✅ Professional appearance with syntax highlighting
7. ✅ Maintains separation from TUI version (separate binaries)

**Verification:**

Run final tests:
```bash
cargo test
cargo build --release --bin logs-parser-desktop
./target/release/logs-parser-desktop
```

All features should work as expected.
