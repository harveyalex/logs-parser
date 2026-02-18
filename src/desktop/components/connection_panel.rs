//! Connection panel component for app selection and connect/disconnect

use crate::heroku_cli::AppInfo;
use dioxus::prelude::*;

use super::custom_select::{CustomSelect, SelectOption};

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
    let app_options: Vec<SelectOption> = available_apps
        .iter()
        .map(|app| SelectOption::new(app.name.clone(), app.name.clone()))
        .collect();

    let theme_options = vec![
        SelectOption::new("wmp", "🎵 WMP 2008"),
        SelectOption::new("win2k", "🖥 Win2K High Contrast"),
        SelectOption::new("win7", "🪟 Win7 Aero"),
    ];

    rsx! {
        div {
            class: "toolbar-bar",
            style: "padding: 12px 16px; display: flex; align-items: center; gap: 12px; position: relative; z-index: 10;",

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

                    CustomSelect {
                        options: app_options,
                        value: selected_app.clone(),
                        placeholder: "Select an app...".to_string(),
                        disabled: is_connected || is_connecting,
                        on_change: move |val: String| {
                            on_app_select.call(val);
                        },
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

                CustomSelect {
                    options: theme_options,
                    value: Some(theme.clone()),
                    placeholder: "Select theme...".to_string(),
                    on_change: move |val: String| {
                        on_theme_change.call(val);
                    },
                }
            }
        }
    }
}
