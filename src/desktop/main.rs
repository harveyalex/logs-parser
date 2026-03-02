//! Desktop application entry point

use dioxus::prelude::*;
use filters::Filter;
use parser::{LogEntry, LogLevel};
use regex::Regex;
use std::sync::Arc;
use tokio::sync::mpsc;

mod components;
mod filters;
mod heroku_cli;
mod parser;
mod stream_manager;

use components::{
    ConnectionPanel, ConnectionStatus, FilterBar, LoadingStep, LogView, StatsHeader,
    StatusIndicator,
};
use heroku_cli::{spawn_login, AppInfo};
use stream_manager::StreamManager;

async fn init_heroku(
    mut connection_status: Signal<ConnectionStatus>,
    mut available_apps: Signal<Vec<AppInfo>>,
) {
    connection_status.set(ConnectionStatus::Loading(LoadingStep::CheckingCli));
    match heroku_cli::check_cli_installed().await {
        Ok(true) => {
            connection_status.set(ConnectionStatus::Loading(LoadingStep::VerifyingAuth));
            match heroku_cli::check_authentication().await {
                Ok(_email) => {
                    connection_status.set(ConnectionStatus::Loading(LoadingStep::FetchingApps));
                    match heroku_cli::fetch_apps().await {
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
                            connection_status.set(ConnectionStatus::Error(format!(
                                "Failed to fetch apps: {}",
                                e
                            )));
                        }
                    }
                }
                Err(_) => {
                    connection_status.set(ConnectionStatus::NotAuthenticated);
                }
            }
        }
        Ok(false) => {
            connection_status.set(ConnectionStatus::Error(
                "Heroku CLI not found. Install from heroku.com/cli".to_string(),
            ));
        }
        Err(e) => {
            connection_status.set(ConnectionStatus::Error(format!(
                "Failed to check CLI: {}",
                e
            )));
        }
    }
}

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

fn main() {
    dioxus::launch(App);
}

/// Parse a filter string into a Filter enum
/// Formats:
/// - dyno:web.1 -> Filter by dyno
/// - source:app -> Filter by source
/// - level:error -> Filter by level (error, warn, info, debug)
/// - /regex/ -> Regex filter
/// - anything else -> Text search
fn parse_filter(input: &str) -> Option<Filter> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Check for dyno: prefix
    if let Some(dyno) = trimmed.strip_prefix("dyno:") {
        return Some(Filter::Dyno(dyno.to_string()));
    }

    // Check for source: prefix
    if let Some(source) = trimmed.strip_prefix("source:") {
        return Some(Filter::Source(source.to_string()));
    }

    // Check for level: prefix
    if let Some(level_str) = trimmed.strip_prefix("level:") {
        let level = match level_str.to_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warn" | "warning" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            _ => LogLevel::Unknown,
        };
        return Some(Filter::LogLevel(level));
    }

    // Check for regex (starts and ends with /)
    if trimmed.starts_with('/') && trimmed.ends_with('/') && trimmed.len() > 2 {
        let pattern = &trimmed[1..trimmed.len() - 1];
        if let Ok(regex) = Regex::new(pattern) {
            return Some(Filter::Regex(regex));
        }
    }

    // Default to text search
    Some(Filter::TextSearch(trimmed.to_string()))
}

#[component]
fn App() -> Element {
    // Connection state
    let mut connection_status = use_signal(|| ConnectionStatus::Loading(LoadingStep::CheckingCli));
    let available_apps = use_signal(Vec::<AppInfo>::new);
    let mut selected_app = use_signal(|| None::<String>);
    let mut stream_manager = use_signal(|| None::<Arc<tokio::sync::Mutex<StreamManager>>>);
    let mut should_monitor = use_signal(|| false);

    // Log data
    let mut all_logs = use_signal(Vec::<LogEntry>::new);
    let mut filtered_logs = use_signal(Vec::<LogEntry>::new);

    // Filter state
    let mut filters = use_signal(Vec::<Filter>::new);
    let mut filter_mode_and = use_signal(|| true);
    let mut login_process =
        use_signal(|| None::<std::sync::Arc<tokio::sync::Mutex<tokio::process::Child>>>);
    let mut theme = use_signal(read_theme);

    // Initialize: Check CLI and fetch apps
    use_effect(move || {
        spawn(async move {
            init_heroku(connection_status, available_apps).await;
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
                let connect_result = {
                    let mut mgr = manager.lock().await;
                    mgr.connect().await
                };

                match connect_result {
                    Ok(_) => {
                        connection_status.set(ConnectionStatus::Streaming);
                        stream_manager.set(Some(manager.clone()));
                        should_monitor.set(true);

                        // Spawn task to receive logs and update state
                        spawn(async move {
                            while let Some(entry) = rx.recv().await {
                                let mut logs = all_logs.write();
                                logs.push(entry);
                                if logs.len() > 10000 {
                                    logs.remove(0); // Remove oldest entry
                                }
                            }
                        });

                        // Spawn task to monitor process and reconnect if needed
                        let manager_clone = manager.clone();
                        spawn(async move {
                            loop {
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                if !should_monitor() {
                                    break;
                                }

                                let mut mgr = manager_clone.lock().await;
                                if !mgr.is_running().await {
                                    connection_status.set(ConnectionStatus::Reconnecting(
                                        mgr.get_reconnect_attempts() + 1,
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
                        connection_status
                            .set(ConnectionStatus::Error(format!("Connection failed: {}", e)));
                    }
                }
            });
        }
    };

    let on_disconnect = move |_| {
        spawn(async move {
            should_monitor.set(false);
            if let Some(manager) = stream_manager() {
                manager.lock().await.disconnect().await;
            }
            stream_manager.set(None);
            all_logs.set(Vec::new());
            connection_status.set(ConnectionStatus::Ready);
        });
    };

    let on_login = move |_| {
        match spawn_login() {
            Ok(child) => {
                let child = std::sync::Arc::new(tokio::sync::Mutex::new(child));
                login_process.set(Some(child.clone()));
                connection_status.set(ConnectionStatus::LoggingIn);

                spawn(async move {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                        // If login_process was cleared (cancelled), stop monitoring
                        if login_process().is_none() {
                            break;
                        }

                        let exit_status = {
                            let mut proc = child.lock().await;
                            proc.try_wait()
                        };

                        match exit_status {
                            Ok(Some(status)) => {
                                login_process.set(None);
                                if status.success() {
                                    init_heroku(connection_status, available_apps).await;
                                } else {
                                    connection_status.set(ConnectionStatus::NotAuthenticated);
                                }
                                break;
                            }
                            Ok(None) => continue, // still running
                            Err(e) => {
                                login_process.set(None);
                                connection_status.set(ConnectionStatus::Error(format!(
                                    "Login error: {}",
                                    e
                                )));
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                connection_status
                    .set(ConnectionStatus::Error(format!("Failed to start login: {}", e)));
            }
        }
    };

    let on_cancel_login = move |_| {
        spawn(async move {
            if let Some(process) = login_process() {
                let mut proc = process.lock().await;
                let _ = proc.kill().await;
            }
            login_process.set(None);
            connection_status.set(ConnectionStatus::NotAuthenticated);
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

    let on_theme_change = move |new_theme: String| {
        write_theme(&new_theme);
        theme.set(new_theme);
    };

    let total_logs = all_logs().len();
    let filtered_count = filtered_logs().len();
    let is_connected = matches!(
        connection_status(),
        ConnectionStatus::Streaming | ConnectionStatus::Reconnecting(_)
    );
    let is_connecting = matches!(connection_status(), ConnectionStatus::Connecting);
    let is_auth_error = matches!(connection_status(), ConnectionStatus::NotAuthenticated);
    let is_logging_in = matches!(connection_status(), ConnectionStatus::LoggingIn);

    rsx! {
        style { {include_str!("styles.css")} }

        div {
            class: format!("app-container theme-{}", theme()),

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
                is_auth_error: is_auth_error,
                is_logging_in: is_logging_in,
                on_app_select: on_app_select,
                on_connect: on_connect,
                on_disconnect: on_disconnect,
                on_login: on_login,
                on_cancel_login: on_cancel_login,
                theme: theme(),
                on_theme_change: on_theme_change,
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
