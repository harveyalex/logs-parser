//! Status indicator component for connection state visualization

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum LoadingStep {
    CheckingCli,
    VerifyingAuth,
    FetchingApps,
}

impl LoadingStep {
    pub fn index(&self) -> u8 {
        match self {
            LoadingStep::CheckingCli   => 0,
            LoadingStep::VerifyingAuth => 1,
            LoadingStep::FetchingApps  => 2,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ConnectionStatus {
    Loading(LoadingStep),
    Ready,
    Connecting,
    Streaming,
    Reconnecting(u32),
    NotAuthenticated,
    LoggingIn,
    Error(String),
}

fn step_class(step_index: u8, current: u8) -> &'static str {
    if step_index < current {
        "loading-step-done"
    } else if step_index == current {
        "loading-step-active"
    } else {
        "loading-step-pending"
    }
}

#[component]
fn LoadingProgress(step: LoadingStep) -> Element {
    let i = step.index();
    rsx! {
        div {
            class: "loading-progress",
            span { class: "{step_class(0, i)}", "Checking CLI" }
            span { class: "loading-arrow", "→" }
            span { class: "{step_class(1, i)}", "Verifying auth" }
            span { class: "loading-arrow", "→" }
            span { class: "{step_class(2, i)}", "Fetching apps" }
        }
    }
}

#[component]
pub fn StatusIndicator(status: ConnectionStatus) -> Element {
    let (text_class, dot_class, text) = match status {
        ConnectionStatus::Loading(step) => {
            return rsx! {
                div {
                    class: "toolbar-bar",
                    style: "display: flex; align-items: center; gap: 8px; padding: 8px 16px;",
                    div { class: "status-dot status-dot-warning" }
                    LoadingProgress { step }
                }
            };
        }
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
            msg,
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
