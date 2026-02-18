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
    on_app_select: EventHandler<String>,
    on_connect: EventHandler<()>,
    on_disconnect: EventHandler<()>,
    on_login: EventHandler<()>,
    on_cancel_login: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "background: #2d2d2d; padding: 16px; border-bottom: 1px solid #555; display: flex; align-items: center; gap: 12px;",

            if is_logging_in {
                // Login in progress: show waiting message + cancel button
                span {
                    style: "color: #ffa500; font-size: 14px; flex: 1;",
                    "Waiting for browser login..."
                }
                button {
                    style: "background: #666; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 500;",
                    onclick: move |_| on_cancel_login.call(()),
                    "Cancel"
                }
            } else if is_auth_error {
                // Not authenticated: show login button
                button {
                    style: "background: #4a9eff; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 500;",
                    onclick: move |_| on_login.call(()),
                    "Login to Heroku"
                }
            } else {
                // Normal state: app dropdown + connect/disconnect
                div {
                    style: "display: flex; flex-direction: column; gap: 4px; flex: 1;",

                    label {
                        style: "color: #ccc; font-size: 12px; font-weight: 500;",
                        "Heroku App"
                    }

                    select {
                        style: "background: #1a1a1a; color: #fff; border: 1px solid #555; padding: 8px; border-radius: 4px; font-size: 14px;",
                        disabled: is_connected || is_connecting,
                        onchange: move |evt| {
                            on_app_select.call(evt.value().clone());
                        },

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
                        style: "background: #ff4444; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 500;",
                        onclick: move |_| on_disconnect.call(()),
                        "Disconnect"
                    }
                } else if is_connecting {
                    button {
                        style: "background: #666; color: white; border: none; padding: 10px 20px; border-radius: 4px; font-size: 14px; font-weight: 500;",
                        disabled: true,
                        "Connecting..."
                    }
                } else {
                    button {
                        style: "background: #4a9eff; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 500;",
                        disabled: selected_app.is_none(),
                        onclick: move |_| on_connect.call(()),
                        "Connect"
                    }
                }
            }
        }
    }
}
