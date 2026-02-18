//! Header component showing log statistics

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct StatsHeaderProps {
    pub total_logs: usize,
    pub filtered_logs: usize,
    pub paused: bool,
    pub filter_mode_and: bool,
}

#[component]
pub fn StatsHeader(props: StatsHeaderProps) -> Element {
    let pause_text = if props.paused { "PAUSED" } else { "LIVE" };
    let filter_mode = if props.filter_mode_and { "AND" } else { "OR" };

    rsx! {
        div {
            class: "stats-header",
            style: "background: #2d2d2d; color: #ffffff; padding: 10px; display: flex; justify-content: space-between;",

            div {
                style: "font-weight: bold;",
                "Heroku Logs Parser"
            }

            div {
                style: "display: flex; gap: 20px;",
                span { "Total: {props.total_logs}" }
                span { "Filtered: {props.filtered_logs}" }
                span {
                    style: if props.paused { "color: #ff6b6b;" } else { "color: #51cf66;" },
                    "{pause_text}"
                }
                span { "Filter Mode: {filter_mode}" }
            }
        }
    }
}
