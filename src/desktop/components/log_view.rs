//! Log view component for displaying filtered log entries

use dioxus::prelude::*;
use crate::parser::{LogEntry, LogLevel};

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
                    span { class: "{level_class(log.level)}", "{log.level:?}:" }
                    span { class: "log-msg",    "{log.message}" }
                }
            }
        }
    }
}
