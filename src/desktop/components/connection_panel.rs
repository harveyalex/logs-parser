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
