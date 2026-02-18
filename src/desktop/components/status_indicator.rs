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
