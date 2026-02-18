//! Desktop application entry point

use dioxus::prelude::*;
use logs_parser::filters::Filter;
use logs_parser::parser::{LogEntry, LogLevel};
use regex::Regex;

mod components;
mod log_reader;
mod export;
use components::{FilterBar, LogView, StatsHeader};
use log_reader::read_log_file;
use export::{copy_to_clipboard, export_to_file};

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
    // State for all logs and filtered logs
    let mut all_logs = use_signal(|| Vec::<LogEntry>::new());
    let mut filtered_logs = use_signal(|| Vec::<LogEntry>::new());

    // State for filters
    let mut filters = use_signal(|| Vec::<Filter>::new());
    let mut filter_mode_and = use_signal(|| true);

    // State for stats
    let mut paused = use_signal(|| false);
    let scroll_position = use_signal(|| 0.0);

    // Effect to apply filters when logs, filters, or mode change
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
                        // AND logic: all filters must match
                        active_filters.iter().all(|f| f.matches(log))
                    } else {
                        // OR logic: at least one filter must match
                        active_filters.iter().any(|f| f.matches(log))
                    }
                })
                .cloned()
                .collect();
            filtered_logs.set(filtered);
        }
    });

    // Start with empty logs - user must open a file
    // (Sample logs removed in favor of file picker)

    // Event handlers
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

    // Keyboard shortcuts handler
    let on_keydown = move |evt: KeyboardEvent| {
        let key = evt.key();
        let ctrl = evt.modifiers().ctrl();

        if ctrl {
            match key {
                dioxus::events::Key::Character(ref s) if s == "o" => {
                    evt.prevent_default();
                    spawn(async move {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("Log Files", &["log", "txt"])
                            .pick_file()
                            .await;

                        if let Some(file) = file {
                            let path = file.path().to_path_buf();
                            match read_log_file(path).await {
                                Ok(logs) => {
                                    all_logs.set(logs);
                                }
                                Err(e) => {
                                    eprintln!("Error reading log file: {}", e);
                                }
                            }
                        }
                    });
                }
                dioxus::events::Key::Character(ref s) if s == "c" => {
                    evt.prevent_default();
                    let logs = filtered_logs();
                    match copy_to_clipboard(&logs) {
                        Ok(_) => println!("Copied {} logs to clipboard", logs.len()),
                        Err(e) => eprintln!("Failed to copy to clipboard: {}", e),
                    }
                }
                dioxus::events::Key::Character(ref s) if s == "s" => {
                    evt.prevent_default();
                    let logs = filtered_logs();
                    spawn(async move {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("Log Files", &["log", "txt"])
                            .set_file_name("exported_logs.log")
                            .save_file()
                            .await;

                        if let Some(file) = file {
                            let path = file.path().to_path_buf();
                            match export_to_file(&logs, path).await {
                                Ok(_) => println!("Exported {} logs", logs.len()),
                                Err(e) => eprintln!("Failed to export: {}", e),
                            }
                        }
                    });
                }
                dioxus::events::Key::Character(ref s) if s == "q" => {
                    evt.prevent_default();
                    std::process::exit(0);
                }
                _ => {}
            }
        } else {
            match key {
                dioxus::events::Key::Character(ref s) if s == "c" => {
                    filters.set(Vec::new());
                }
                dioxus::events::Key::Character(ref s) if s == "p" || s == " " => {
                    paused.set(!paused());
                }
                _ => {}
            }
        }
    };

    rsx! {
        style { {include_str!("styles.css")} }

        div {
            class: "app-container",
            style: "display: flex; flex-direction: column; height: 100vh;",
            onkeydown: on_keydown,
            tabindex: 0,

            // Stats Header
            StatsHeader {
                total_logs: total_logs,
                filtered_logs: filtered_count,
                paused: paused(),
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
                scroll_position: scroll_position(),
            }

            // Status Bar with actions and help text
            div {
                style: "background: #2d2d2d; color: #888; padding: 8px; font-size: 12px; border-top: 1px solid #555; display: flex; justify-content: space-between; align-items: center;",

                div {
                    style: "display: flex; gap: 8px;",

                    button {
                        style: "background: #4a9eff; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;",
                        onclick: move |_| {
                            spawn(async move {
                                let file = rfd::AsyncFileDialog::new()
                                    .add_filter("Log Files", &["log", "txt"])
                                    .pick_file()
                                    .await;

                                if let Some(file) = file {
                                    let path = file.path().to_path_buf();
                                    match read_log_file(path).await {
                                        Ok(logs) => {
                                            all_logs.set(logs);
                                        }
                                        Err(e) => {
                                            eprintln!("Error reading log file: {}", e);
                                        }
                                    }
                                }
                            });
                        },
                        "Open (Ctrl+O)"
                    }

                    button {
                        style: "background: #50c878; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;",
                        onclick: move |_| {
                            let logs = filtered_logs();
                            match copy_to_clipboard(&logs) {
                                Ok(_) => println!("Copied {} logs to clipboard", logs.len()),
                                Err(e) => eprintln!("Failed to copy to clipboard: {}", e),
                            }
                        },
                        disabled: filtered_count == 0,
                        "Copy (Ctrl+C)"
                    }

                    button {
                        style: "background: #ff8c00; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px;",
                        onclick: move |_| {
                            let logs = filtered_logs();
                            spawn(async move {
                                let file = rfd::AsyncFileDialog::new()
                                    .add_filter("Log Files", &["log", "txt"])
                                    .set_file_name("exported_logs.log")
                                    .save_file()
                                    .await;

                                if let Some(file) = file {
                                    let path = file.path().to_path_buf();
                                    match export_to_file(&logs, path).await {
                                        Ok(_) => println!("Exported {} logs to file", logs.len()),
                                        Err(e) => eprintln!("Failed to export to file: {}", e),
                                    }
                                }
                            });
                        },
                        disabled: filtered_count == 0,
                        "Export (Ctrl+S)"
                    }
                }

                span {
                    style: "font-size: 11px;",
                    "Ctrl+O: Open | Ctrl+C: Copy | Ctrl+S: Export | Ctrl+Q: Quit | C: Clear filters | P: Pause"
                }
            }
        }
    }
}
