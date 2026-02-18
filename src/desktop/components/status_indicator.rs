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
    let (color, text) = match &status {
        ConnectionStatus::Loading => ("#ffa500", "Loading...".to_string()),
        ConnectionStatus::Ready => ("#888", "Ready to connect".to_string()),
        ConnectionStatus::Connecting => ("#ffa500", "Connecting...".to_string()),
        ConnectionStatus::Streaming => ("#50c878", "Streaming".to_string()),
        ConnectionStatus::Reconnecting(n) => {
            ("#ffa500", format!("Reconnecting (attempt {}/5)...", n))
        }
        ConnectionStatus::NotAuthenticated => {
            ("#ff4444", "Not logged in to Heroku".to_string())
        }
        ConnectionStatus::LoggingIn => ("#ffa500", "Logging in to Heroku...".to_string()),
        ConnectionStatus::Error(msg) => ("#ff4444", msg.clone()),
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
