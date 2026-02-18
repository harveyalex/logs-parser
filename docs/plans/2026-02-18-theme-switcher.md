# Theme Switcher Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a theme switcher dropdown to the connection panel that switches between three nostalgic Windows aesthetics (WMP 2008, Win2K High Contrast, Win7 Aero), persisted across restarts.

**Architecture:** CSS custom properties per `.theme-xxx` class on the root div. All hardcoded hex colours in components become `var(--x)` references or semantic CSS class names. Theme is stored in `~/.config/logs-parser/theme`.

**Tech Stack:** CSS custom properties, Dioxus 0.6 signals, `std::fs` for persistence.

---

### Task 1: Rewrite styles.css with full theme system

**Files:**
- Modify: `src/desktop/styles.css`

**Step 1: Replace the entire file**

```css
/* ── Reset ── */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

/* ── WMP 2008 (default) ── */
.theme-wmp {
    --bg-primary:    #080c14;
    --bg-secondary:  #0d1520;
    --bg-tertiary:   #060a10;
    --border:        #1a3a5c;
    --text-primary:  #e0e8f0;
    --text-dim:      #5a7a9a;
    --accent:        #00b4d8;
    --accent-text:   #000;
    --danger:        #ff4444;
    --success:       #00d4aa;
    --warning:       #ffa500;
    --log-error-bg:  #1a0a0a;
    --font-ui:       'Segoe UI', Arial, sans-serif;
    --font-mono:     'Consolas', 'Courier New', monospace;
    --radius:        2px;
    --btn-gradient:  linear-gradient(180deg, #1a6a8a 0%, #0a3a5a 50%, #062a4a 100%);
    --btn-shine:     linear-gradient(180deg, rgba(255,255,255,0.25) 0%, rgba(255,255,255,0) 50%);
    --panel-shadow:  0 0 8px rgba(0,180,216,0.2), inset 0 1px 0 rgba(0,180,216,0.15);
    --panel-blur:    none;
}

/* ── Win2K High Contrast ── */
.theme-win2k {
    --bg-primary:    #000000;
    --bg-secondary:  #000000;
    --bg-tertiary:   #000000;
    --border:        #ffffff;
    --text-primary:  #ffffff;
    --text-dim:      #ffffff;
    --accent:        #ffff00;
    --accent-text:   #000000;
    --danger:        #ff0000;
    --success:       #00ff00;
    --warning:       #ffff00;
    --log-error-bg:  #200000;
    --font-ui:       'MS Sans Serif', 'Arial', sans-serif;
    --font-mono:     'Courier New', monospace;
    --radius:        0px;
    --btn-gradient:  #000000;
    --btn-shine:     none;
    --panel-shadow:  none;
    --panel-blur:    none;
}

/* ── Win7 Aero ── */
.theme-win7 {
    --bg-primary:    #1a2535;
    --bg-secondary:  rgba(255,255,255,0.06);
    --bg-tertiary:   rgba(0,0,0,0.4);
    --border:        rgba(255,255,255,0.15);
    --text-primary:  #ffffff;
    --text-dim:      #a0b4c8;
    --accent:        #3399ff;
    --accent-text:   #ffffff;
    --danger:        #ff5555;
    --success:       #55cc88;
    --warning:       #ffaa33;
    --log-error-bg:  rgba(180,0,0,0.15);
    --font-ui:       'Segoe UI', Arial, sans-serif;
    --font-mono:     'Consolas', 'Courier New', monospace;
    --radius:        6px;
    --btn-gradient:  linear-gradient(180deg, rgba(255,255,255,0.18) 0%, rgba(255,255,255,0.04) 100%);
    --btn-shine:     none;
    --panel-shadow:  0 4px 16px rgba(0,80,200,0.25), inset 0 1px 0 rgba(255,255,255,0.12);
    --panel-blur:    blur(12px);
}

/* ── App layout ── */
body {
    background: #080c14;
}

.app-container {
    width: 100%;
    height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-ui);
    display: flex;
    flex-direction: column;
}

/* ── Toolbar bars ── */
.toolbar-bar {
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    box-shadow: var(--panel-shadow);
    backdrop-filter: var(--panel-blur);
}

/* ── Buttons ── */
.btn {
    padding: 10px 20px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    font-family: var(--font-ui);
    position: relative;
    overflow: hidden;
    transition: opacity 0.1s;
    background: var(--btn-gradient);
    color: var(--text-primary);
}

.btn:disabled {
    opacity: 0.5;
    cursor: default;
}

/* WMP gloss */
.theme-wmp .btn::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 50%;
    background: var(--btn-shine);
    pointer-events: none;
    border-radius: var(--radius) var(--radius) 0 0;
}

/* Win2K 3D raised */
.theme-win2k .btn {
    border-top:    2px solid #ffffff;
    border-left:   2px solid #ffffff;
    border-bottom: 2px solid #808080;
    border-right:  2px solid #808080;
    border-radius: 0;
}
.theme-win2k .btn:active {
    border-top:    2px solid #808080;
    border-left:   2px solid #808080;
    border-bottom: 2px solid #ffffff;
    border-right:  2px solid #ffffff;
}

/* Win7 glass */
.theme-win7 .btn {
    backdrop-filter: blur(4px);
    box-shadow: 0 1px 4px rgba(0,0,0,0.3), inset 0 1px 0 rgba(255,255,255,0.15);
}

/* Button colour variants */
.btn-connect {
    background: var(--btn-gradient);
    color: var(--accent-text);
    border-color: var(--accent);
}
.theme-win2k .btn-connect {
    background: var(--accent);
    color: var(--accent-text);
}

.btn-disconnect {
    background: var(--danger);
    color: #ffffff;
    border-color: var(--danger);
}
.theme-win2k .btn-disconnect {
    background: var(--bg-primary);
    color: var(--danger);
    border-color: var(--danger);
}

.btn-neutral {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-color: var(--border);
}

/* ── Select / input ── */
.themed-select,
.themed-input {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border);
    padding: 8px;
    border-radius: var(--radius);
    font-size: 14px;
    font-family: var(--font-ui);
}
.themed-select:focus,
.themed-input:focus {
    outline: 1px solid var(--accent);
}

/* ── Log view ── */
.log-view {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
    padding: 10px;
}

.log-view-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-dim);
    font-size: 16px;
    background: var(--bg-tertiary);
}

.log-entry {
    padding: 4px 0;
}

.log-entry-error {
    background: var(--log-error-bg);
}

/* Log level colours */
.level-error   { color: var(--danger);       font-weight: bold; }
.level-warn    { color: var(--warning);      font-weight: bold; }
.level-info    { color: var(--success);      font-weight: bold; }
.level-debug   { color: var(--text-dim);     font-weight: bold; }
.level-unknown { color: var(--text-primary); font-weight: bold; }

/* Log field colours */
.log-time   { color: var(--text-dim);  margin-right: 8px; }
.log-source { color: var(--accent);    margin-right: 8px; }
.log-dyno   { color: var(--success);   margin-right: 8px; }
.log-msg    { color: var(--text-primary); }

/* ── Status indicator ── */
.status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
}
.status-dot-ok      { background: var(--success); }
.status-dot-warning { background: var(--warning); }
.status-dot-error   { background: var(--danger); }
.status-dot-dim     { background: var(--text-dim); }

.status-ok      { color: var(--success); }
.status-warning { color: var(--warning); }
.status-error   { color: var(--danger); }
.status-dim     { color: var(--text-dim); }

/* ── Filter tag ── */
.filter-tag {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border);
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 14px;
}

/* ── Theme picker in toolbar ── */
.theme-picker {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: auto;
}

.theme-picker-label {
    color: var(--text-dim);
    font-size: 12px;
    white-space: nowrap;
}
```

**Step 2: Build to verify CSS is valid (no Rust errors, just a compile check)**

```bash
cargo build --bin logs-parser-desktop 2>&1 | head -30
```
Expected: compiles successfully.

**Step 3: Commit**

```bash
git add src/desktop/styles.css
git commit -m "feat(theme): add CSS variable theme system with three themes"
```

---

### Task 2: Add theme signal and persistence to main.rs

**Files:**
- Modify: `src/desktop/main.rs`

**Step 1: Add theme helper functions before `fn main()`**

Add these three functions right above `fn main()` (after the `parse_filter` function):

```rust
fn theme_config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&home)
        .join(".config")
        .join("logs-parser")
        .join("theme")
}

fn read_theme() -> String {
    std::fs::read_to_string(theme_config_path())
        .unwrap_or_else(|_| "wmp".to_string())
        .trim()
        .to_string()
}

fn write_theme(theme: &str) {
    let path = theme_config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, theme);
}
```

**Step 2: Add theme signal to App, after `let mut login_process = ...`**

Add this line:
```rust
let mut theme = use_signal(read_theme);
```

**Step 3: Add on_theme_change handler in App, after the `on_toggle_mode` handler**

```rust
let on_theme_change = move |new_theme: String| {
    write_theme(&new_theme);
    theme.set(new_theme);
};
```

**Step 4: Update root div in `rsx!` to include theme class and pass new props to ConnectionPanel**

Change the root `div` opening from:
```rust
div {
    class: "app-container",
    style: "display: flex; flex-direction: column; height: 100vh;",
```
To:
```rust
div {
    class: format!("app-container theme-{}", theme()),
```
(Remove the `style:` line — layout is now in CSS `.app-container`.)

**Step 5: Add `theme` and `on_theme_change` props to the ConnectionPanel call**

Change the ConnectionPanel block from:
```rust
ConnectionPanel {
    available_apps: available_apps(),
    selected_app: selected_app(),
    is_connected: is_connected,
    is_connecting: is_connecting,
    is_auth_error: is_auth_error,
    is_logging_in: is_logging_in,
    on_app_select: on_app_select,
    on_connect: on_connect,
    on_disconnect: on_disconnect,
    on_login: on_login,
    on_cancel_login: on_cancel_login,
}
```
To:
```rust
ConnectionPanel {
    available_apps: available_apps(),
    selected_app: selected_app(),
    is_connected: is_connected,
    is_connecting: is_connecting,
    is_auth_error: is_auth_error,
    is_logging_in: is_logging_in,
    theme: theme(),
    on_app_select: on_app_select,
    on_connect: on_connect,
    on_disconnect: on_disconnect,
    on_login: on_login,
    on_cancel_login: on_cancel_login,
    on_theme_change: on_theme_change,
}
```

**Step 6: Build**

```bash
cargo build --bin logs-parser-desktop 2>&1 | head -50
```
Expected: fails with "struct ConnectionPanel has no field `theme`" — this is correct, we fix it in Task 3.

**Step 7: Commit** (will fail to build until Task 3; commit anyway as work-in-progress is fine since we're on a branch)

```bash
git add src/desktop/main.rs
git commit -m "feat(theme): add theme signal and persistence to main.rs"
```

---

### Task 3: Add theme dropdown to connection_panel.rs

**Files:**
- Modify: `src/desktop/components/connection_panel.rs`

**Step 1: Replace the entire file**

```rust
//! Connection panel component for app selection and connect/disconnect

use crate::heroku_cli::AppInfo;
use dioxus::prelude::*;

#[component]
pub fn ConnectionPanel(
    available_apps: Vec<AppInfo>,
    selected_app: Option<String>,
    is_connected: bool,
    is_connecting: bool,
    is_auth_error: bool,
    is_logging_in: bool,
    theme: String,
    on_app_select: EventHandler<String>,
    on_connect: EventHandler<()>,
    on_disconnect: EventHandler<()>,
    on_login: EventHandler<()>,
    on_cancel_login: EventHandler<()>,
    on_theme_change: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "toolbar-bar",
            style: "padding: 12px 16px; display: flex; align-items: center; gap: 12px;",

            if is_logging_in {
                span {
                    style: "font-size: 14px; flex: 1; color: var(--warning);",
                    "Waiting for browser login..."
                }
                button {
                    class: "btn btn-neutral",
                    onclick: move |_| on_cancel_login.call(()),
                    "Cancel"
                }
            } else if is_auth_error {
                button {
                    class: "btn btn-connect",
                    onclick: move |_| on_login.call(()),
                    "Login to Heroku"
                }
            } else {
                div {
                    style: "display: flex; flex-direction: column; gap: 4px; flex: 1;",

                    label {
                        style: "color: var(--text-dim); font-size: 12px; font-weight: 500;",
                        "Heroku App"
                    }

                    select {
                        class: "themed-select",
                        disabled: is_connected || is_connecting,
                        onchange: move |evt| on_app_select.call(evt.value().clone()),

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

                if is_connected {
                    button {
                        class: "btn btn-disconnect",
                        onclick: move |_| on_disconnect.call(()),
                        "Disconnect"
                    }
                } else if is_connecting {
                    button {
                        class: "btn btn-neutral",
                        disabled: true,
                        "Connecting..."
                    }
                } else {
                    button {
                        class: "btn btn-connect",
                        disabled: selected_app.is_none(),
                        onclick: move |_| on_connect.call(()),
                        "Connect"
                    }
                }
            }

            // Theme picker — always visible on right side
            div {
                class: "theme-picker",

                span {
                    class: "theme-picker-label",
                    "Theme:"
                }

                select {
                    class: "themed-select",
                    style: "padding: 6px 8px; font-size: 13px;",
                    value: "{theme}",
                    onchange: move |evt| on_theme_change.call(evt.value().clone()),

                    option { value: "wmp",   selected: theme == "wmp",   "🎵 WMP 2008" }
                    option { value: "win2k", selected: theme == "win2k", "🖥 Win2K High Contrast" }
                    option { value: "win7",  selected: theme == "win7",  "🪟 Win7 Aero" }
                }
            }
        }
    }
}
```

**Step 2: Build**

```bash
cargo build --bin logs-parser-desktop 2>&1 | head -50
```
Expected: compiles successfully (or fails only in other files).

**Step 3: Commit**

```bash
git add src/desktop/components/connection_panel.rs
git commit -m "feat(theme): add theme dropdown and migrate connection_panel to CSS vars"
```

---

### Task 4: Migrate status_indicator.rs to CSS classes

**Files:**
- Modify: `src/desktop/components/status_indicator.rs`

**Step 1: Replace the entire file**

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
    NotAuthenticated,
    LoggingIn,
    Error(String),
}

#[component]
pub fn StatusIndicator(status: ConnectionStatus) -> Element {
    let (text_class, dot_class, text) = match &status {
        ConnectionStatus::Loading => (
            "status-warning",
            "status-dot status-dot-warning",
            "Loading...".to_string(),
        ),
        ConnectionStatus::Ready => (
            "status-dim",
            "status-dot status-dot-dim",
            "Ready to connect".to_string(),
        ),
        ConnectionStatus::Connecting => (
            "status-warning",
            "status-dot status-dot-warning",
            "Connecting...".to_string(),
        ),
        ConnectionStatus::Streaming => (
            "status-ok",
            "status-dot status-dot-ok",
            "Streaming".to_string(),
        ),
        ConnectionStatus::Reconnecting(n) => (
            "status-warning",
            "status-dot status-dot-warning",
            format!("Reconnecting (attempt {}/5)...", n),
        ),
        ConnectionStatus::NotAuthenticated => (
            "status-error",
            "status-dot status-dot-error",
            "Not logged in to Heroku".to_string(),
        ),
        ConnectionStatus::LoggingIn => (
            "status-warning",
            "status-dot status-dot-warning",
            "Logging in to Heroku...".to_string(),
        ),
        ConnectionStatus::Error(msg) => (
            "status-error",
            "status-dot status-dot-error",
            msg.clone(),
        ),
    };

    rsx! {
        div {
            class: "toolbar-bar",
            style: "display: flex; align-items: center; gap: 8px; padding: 8px 16px;",

            div { class: "{dot_class}" }

            span {
                class: "{text_class}",
                style: "font-size: 14px; font-weight: 500;",
                "{text}"
            }
        }
    }
}
```

**Step 2: Build**

```bash
cargo build --bin logs-parser-desktop 2>&1 | head -30
```
Expected: compiles successfully.

**Step 3: Commit**

```bash
git add src/desktop/components/status_indicator.rs
git commit -m "feat(theme): migrate status_indicator to CSS variables"
```

---

### Task 5: Migrate stats_header.rs to CSS variables

**Files:**
- Modify: `src/desktop/components/stats_header.rs`

**Step 1: Replace the entire file**

```rust
use dioxus::prelude::*;

#[component]
pub fn StatsHeader(total_logs: usize, filtered_logs: usize, filter_mode_and: bool) -> Element {
    let filter_text = if filter_mode_and { "AND" } else { "OR" };

    rsx! {
        div {
            class: "toolbar-bar",
            style: "padding: 12px 16px; border-bottom: 2px solid var(--accent); display: flex; justify-content: space-between; align-items: center;",

            div {
                style: "display: flex; gap: 24px;",

                div {
                    span {
                        style: "color: var(--text-dim); font-size: 12px;",
                        "Total Logs: "
                    }
                    span {
                        style: "color: var(--accent); font-weight: bold; font-size: 14px;",
                        "{total_logs}"
                    }
                }

                div {
                    span {
                        style: "color: var(--text-dim); font-size: 12px;",
                        "Filtered: "
                    }
                    span {
                        style: "color: var(--success); font-weight: bold; font-size: 14px;",
                        "{filtered_logs}"
                    }
                }

                div {
                    span {
                        style: "color: var(--text-dim); font-size: 12px;",
                        "Filter Mode: "
                    }
                    span {
                        style: "color: var(--warning); font-weight: bold; font-size: 14px;",
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

**Step 2: Build**

```bash
cargo build --bin logs-parser-desktop 2>&1 | head -30
```
Expected: compiles successfully.

**Step 3: Commit**

```bash
git add src/desktop/components/stats_header.rs
git commit -m "feat(theme): migrate stats_header to CSS variables"
```

---

### Task 6: Migrate filter_bar.rs to CSS variables

**Files:**
- Modify: `src/desktop/components/filter_bar.rs`

**Step 1: Replace the entire file**

```rust
//! Filter bar component for adding and managing log filters

use dioxus::prelude::*;
use logs_parser::filters::Filter;

#[derive(Props, Clone, PartialEq)]
pub struct FilterBarProps {
    pub filters: Vec<Filter>,
    pub on_add_filter: EventHandler<String>,
    pub on_clear_filters: EventHandler<()>,
    pub on_toggle_mode: EventHandler<()>,
    pub filter_mode_and: bool,
}

#[component]
pub fn FilterBar(props: FilterBarProps) -> Element {
    let mut input_value = use_signal(String::new);
    let filter_mode = if props.filter_mode_and { "AND" } else { "OR" };

    let on_input = move |evt: Event<FormData>| {
        input_value.set(evt.value());
    };

    let on_key_press = move |evt: Event<KeyboardData>| {
        if evt.key() == Key::Enter {
            let value = input_value();
            if !value.is_empty() {
                props.on_add_filter.call(value.clone());
                input_value.set(String::new());
            }
        }
    };

    let on_add_click = move |_| {
        let value = input_value();
        if !value.is_empty() {
            props.on_add_filter.call(value.clone());
            input_value.set(String::new());
        }
    };

    let on_clear_click = move |_| {
        props.on_clear_filters.call(());
    };

    let on_toggle_click = move |_| {
        props.on_toggle_mode.call(());
    };

    rsx! {
        div {
            class: "toolbar-bar filter-bar",
            style: "padding: 10px; border-bottom: 1px solid var(--border);",

            div {
                style: "display: flex; gap: 10px; align-items: center; margin-bottom: 10px;",

                input {
                    r#type: "text",
                    class: "themed-input",
                    value: "{input_value}",
                    placeholder: "Enter filter (text, dyno:web.1, source:app, level:error, /regex/)",
                    oninput: on_input,
                    onkeydown: on_key_press,
                    style: "flex: 1;",
                }

                button {
                    class: "btn btn-connect",
                    style: "padding: 8px 16px;",
                    onclick: on_add_click,
                    "Add Filter"
                }

                button {
                    class: "btn btn-disconnect",
                    style: "padding: 8px 16px;",
                    onclick: on_clear_click,
                    "Clear"
                }

                button {
                    class: "btn btn-neutral",
                    style: "padding: 8px 16px;",
                    onclick: on_toggle_click,
                    "Toggle {filter_mode}"
                }
            }

            if !props.filters.is_empty() {
                div {
                    style: "display: flex; flex-wrap: wrap; gap: 8px;",
                    for filter in props.filters.iter() {
                        div {
                            class: "filter-tag",
                            "{filter.display()}"
                        }
                    }
                }
            }
        }
    }
}
```

**Step 2: Build**

```bash
cargo build --bin logs-parser-desktop 2>&1 | head -30
```
Expected: compiles successfully.

**Step 3: Commit**

```bash
git add src/desktop/components/filter_bar.rs
git commit -m "feat(theme): migrate filter_bar to CSS variables"
```

---

### Task 7: Migrate log_view.rs to CSS classes

**Files:**
- Modify: `src/desktop/components/log_view.rs`

**Step 1: Replace the entire file**

```rust
//! Log view component for displaying filtered log entries

use dioxus::prelude::*;
use logs_parser::parser::{LogEntry, LogLevel};

#[derive(Props, Clone, PartialEq)]
pub struct LogViewProps {
    pub logs: Vec<LogEntry>,
    pub scroll_position: f64,
}

fn log_row_class(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error => "log-entry log-entry-error",
        _ => "log-entry",
    }
}

fn level_class(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error   => "level-error",
        LogLevel::Warn    => "level-warn",
        LogLevel::Info    => "level-info",
        LogLevel::Debug   => "level-debug",
        LogLevel::Unknown => "level-unknown",
    }
}

#[component]
pub fn LogView(props: LogViewProps) -> Element {
    if props.logs.is_empty() {
        return rsx! {
            div {
                class: "log-view-empty",
                "No logs to display. Waiting for log input..."
            }
        };
    }

    rsx! {
        div {
            class: "log-view",

            for (idx, log) in props.logs.iter().enumerate() {
                div {
                    key: "{idx}",
                    class: "{log_row_class(log.level)}",

                    span { class: "log-time",   "{log.format_time()}" }
                    span { class: "log-source", "{log.source}" }
                    span { class: "log-dyno",   "[{log.dyno}]" }
                    span { class: "{level_class(log.level)} log-level", "{log.level:?}:" }
                    span { class: "log-msg",    "{log.message}" }
                }
            }
        }
    }
}
```

**Step 2: Build**

```bash
cargo build --bin logs-parser-desktop 2>&1 | head -30
```
Expected: compiles successfully.

**Step 3: Commit**

```bash
git add src/desktop/components/log_view.rs
git commit -m "feat(theme): migrate log_view to CSS classes"
```

---

### Task 8: Final build verification

**Step 1: Clean build**

```bash
cargo build --bin logs-parser-desktop 2>&1
```
Expected: no errors, no warnings about unused variables.

**Step 2: Run the app and verify visually**

```bash
cargo run --bin logs-parser-desktop
```

Check:
1. App starts with WMP 2008 theme (dark navy/teal)
2. Theme dropdown is visible at right of connection toolbar
3. Switching to "Win2K High Contrast" — pure black/white/yellow, square corners, 3D button borders
4. Switching to "Win7 Aero" — blue-tinted with glass effect, rounded corners
5. Quit and relaunch — chosen theme persists

**Step 3: Final commit if any fixes were needed**

```bash
git add -p
git commit -m "fix(theme): correct any visual issues found during testing"
```
