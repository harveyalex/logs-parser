//! Filter bar component for adding and managing log filters

use dioxus::prelude::*;
use logs_parser::filters::Filter;

#[derive(Props, Clone, PartialEq)]
pub struct FilterBarProps {
    pub filters: Vec<Filter>,
    pub on_add_filter: EventHandler<String>,
    pub on_clear_filters: EventHandler<()>,
    pub on_toggle_mode: EventHandler<()>,
    pub filter_mode_and: bool,
}

#[component]
pub fn FilterBar(props: FilterBarProps) -> Element {
    let mut input_value = use_signal(String::new);
    let filter_mode = if props.filter_mode_and { "AND" } else { "OR" };

    let on_input = move |evt: Event<FormData>| {
        input_value.set(evt.value());
    };

    let on_key_press = move |evt: Event<KeyboardData>| {
        if evt.key() == Key::Enter {
            let value = input_value();
            if !value.is_empty() {
                props.on_add_filter.call(value.clone());
                input_value.set(String::new());
            }
        }
    };

    let on_add_click = move |_| {
        let value = input_value();
        if !value.is_empty() {
            props.on_add_filter.call(value.clone());
            input_value.set(String::new());
        }
    };

    let on_clear_click = move |_| {
        props.on_clear_filters.call(());
    };

    let on_toggle_click = move |_| {
        props.on_toggle_mode.call(());
    };

    rsx! {
        div {
            class: "toolbar-bar filter-bar",
            style: "padding: 10px;",

            div {
                style: "display: flex; gap: 10px; align-items: center; margin-bottom: 10px;",

                input {
                    r#type: "text",
                    class: "themed-input",
                    value: "{input_value}",
                    placeholder: "Enter filter (text, dyno:web.1, source:app, level:error, /regex/)",
                    oninput: on_input,
                    onkeydown: on_key_press,
                    style: "flex: 1;",
                }

                button {
                    class: "btn btn-connect",
                    style: "padding: 8px 16px;",
                    onclick: on_add_click,
                    "Add Filter"
                }

                button {
                    class: "btn btn-disconnect",
                    style: "padding: 8px 16px;",
                    onclick: on_clear_click,
                    "Clear"
                }

                button {
                    class: "btn btn-neutral",
                    style: "padding: 8px 16px;",
                    onclick: on_toggle_click,
                    "Toggle {filter_mode}"
                }
            }

            if !props.filters.is_empty() {
                div {
                    style: "display: flex; flex-wrap: wrap; gap: 8px;",
                    for filter in props.filters.iter() {
                        div {
                            class: "filter-tag",
                            "{filter.display()}"
                        }
                    }
                }
            }
        }
    }
}
