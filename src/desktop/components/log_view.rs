//! Log view component for displaying filtered log entries

use dioxus::prelude::*;
use logs_parser::parser::{LogEntry, LogLevel};

#[derive(Props, Clone, PartialEq)]
pub struct LogViewProps {
    pub logs: Vec<LogEntry>,
    pub scroll_position: f64,
}

// Helper function to get background style for error logs
fn get_log_style(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error => "background: #3d1f1f;",
        _ => "",
    }
}

// Helper function to get color for log level text
fn get_level_color(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error => "#ff6b6b",
        LogLevel::Warn => "#ffd93d",
        LogLevel::Info => "#51cf66",
        LogLevel::Debug => "#a9a9a9",
        LogLevel::Unknown => "#ffffff",
    }
}

#[component]
pub fn LogView(props: LogViewProps) -> Element {
    if props.logs.is_empty() {
        return rsx! {
            div {
                class: "log-view-empty",
                style: "flex: 1; display: flex; align-items: center; justify-content: center; color: #888; font-size: 16px;",
                "No logs to display. Waiting for log input..."
            }
        };
    }

    rsx! {
        div {
            class: "log-view",
            style: "flex: 1; overflow-y: auto; background: #1e1e1e; color: #d4d4d4; font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace; font-size: 13px; padding: 10px;",

            for (idx, log) in props.logs.iter().enumerate() {
                div {
                    key: "{idx}",
                    class: "log-entry",
                    style: "padding: 4px 0; {get_log_style(log.level)}",

                    span {
                        style: "color: #808080; margin-right: 8px;",
                        "{log.format_time()}"
                    }

                    span {
                        style: "color: #4a9eff; margin-right: 8px;",
                        "{log.source}"
                    }

                    span {
                        style: "color: #ff79c6; margin-right: 8px;",
                        "[{log.dyno}]"
                    }

                    span {
                        style: "color: {get_level_color(log.level)}; margin-right: 8px; font-weight: bold;",
                        "{log.level:?}:"
                    }

                    span {
                        style: "color: #d4d4d4;",
                        "{log.message}"
                    }
                }
            }
        }
    }
}
