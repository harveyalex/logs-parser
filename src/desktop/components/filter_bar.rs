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
            class: "filter-bar",
            style: "background: #3a3a3a; padding: 10px; border-bottom: 1px solid #555;",

            div {
                style: "display: flex; gap: 10px; align-items: center; margin-bottom: 10px;",

                input {
                    r#type: "text",
                    value: "{input_value}",
                    placeholder: "Enter filter (text, dyno:web.1, source:app, level:error, /regex/)",
                    oninput: on_input,
                    onkeydown: on_key_press,
                    style: "flex: 1; padding: 8px; background: #2d2d2d; border: 1px solid #555; color: #fff; border-radius: 4px;",
                }

                button {
                    onclick: on_add_click,
                    style: "padding: 8px 16px; background: #4a9eff; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Add Filter"
                }

                button {
                    onclick: on_clear_click,
                    style: "padding: 8px 16px; background: #ff6b6b; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Clear"
                }

                button {
                    onclick: on_toggle_click,
                    style: "padding: 8px 16px; background: #51cf66; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "Toggle {filter_mode}"
                }
            }

            // Display active filters as tags
            if !props.filters.is_empty() {
                div {
                    style: "display: flex; flex-wrap: wrap; gap: 8px;",
                    for filter in props.filters.iter() {
                        div {
                            style: "background: #555; color: #fff; padding: 4px 12px; border-radius: 12px; font-size: 14px;",
                            "{filter.display()}"
                        }
                    }
                }
            }
        }
    }
}
