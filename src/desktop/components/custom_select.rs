//! Custom dropdown select component — replaces native <select> for full styling control

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CustomSelectProps {
    pub options: Vec<SelectOption>,
    pub value: Option<String>,
    pub placeholder: String,
    pub on_change: EventHandler<String>,
    #[props(default = false)]
    pub disabled: bool,
}

#[component]
pub fn CustomSelect(props: CustomSelectProps) -> Element {
    let mut is_open = use_signal(|| false);

    let selected_label = props
        .options
        .iter()
        .find(|o| Some(&o.value) == props.value.as_ref())
        .map(|o| o.label.clone())
        .unwrap_or_else(|| props.placeholder.clone());

    let is_placeholder = props.value.is_none();
    let disabled = props.disabled;
    let options = props.options.clone();
    let current_value = props.value.clone();

    let trigger_class = if disabled {
        "select-trigger select-trigger--disabled"
    } else {
        "select-trigger"
    };

    rsx! {
        div {
            class: "custom-select",

            // Backdrop: catches clicks outside the dropdown to close it
            if is_open() {
                div {
                    class: "select-backdrop",
                    onclick: move |_| is_open.set(false),
                }
            }

            // Trigger button
            button {
                class: "{trigger_class}",
                r#type: "button",
                disabled,
                onclick: move |_| {
                    if !disabled {
                        is_open.set(!is_open());
                    }
                },
                span {
                    class: if is_placeholder { "select-value select-value--placeholder" } else { "select-value" },
                    "{selected_label}"
                }
                span {
                    class: "select-chevron",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "12",
                        height: "8",
                        view_box: "0 0 12 8",
                        fill: "none",
                        path {
                            d: "M1 1l5 5 5-5",
                            stroke: "currentColor",
                            stroke_width: "1.5",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                        }
                    }
                }
            }

            // Dropdown list
            if is_open() {
                div {
                    class: "select-dropdown",
                    for opt in options {
                        {
                            let is_selected = current_value.as_ref() == Some(&opt.value);
                            let opt_value = opt.value.clone();
                            let opt_class = if is_selected {
                                "select-option select-option--selected"
                            } else {
                                "select-option"
                            };
                            rsx! {
                                div {
                                    class: "{opt_class}",
                                    onclick: move |_| {
                                        props.on_change.call(opt_value.clone());
                                        is_open.set(false);
                                    },
                                    "{opt.label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
