use dioxus::prelude::*;

#[component]
pub fn StatsHeader(total_logs: usize, filtered_logs: usize, filter_mode_and: bool) -> Element {
    let filter_text = if filter_mode_and { "AND" } else { "OR" };

    rsx! {
        div {
            class: "toolbar-bar",
            style: "padding: 12px 16px; border-bottom: 2px solid var(--accent); display: flex; justify-content: space-between; align-items: center;",

            div {
                style: "display: flex; gap: 24px;",

                div {
                    span {
                        style: "color: var(--text-dim); font-size: 12px;",
                        "Total Logs: "
                    }
                    span {
                        style: "color: var(--accent); font-weight: bold; font-size: 14px;",
                        "{total_logs}"
                    }
                }

                div {
                    span {
                        style: "color: var(--text-dim); font-size: 12px;",
                        "Filtered: "
                    }
                    span {
                        style: "color: var(--success); font-weight: bold; font-size: 14px;",
                        "{filtered_logs}"
                    }
                }

                div {
                    span {
                        style: "color: var(--text-dim); font-size: 12px;",
                        "Filter Mode: "
                    }
                    span {
                        style: "color: var(--warning); font-weight: bold; font-size: 14px;",
                        "{filter_text}"
                    }
                }
            }

            h1 {
                style: "margin: 0; font-size: 18px; font-weight: 600;",
                "Heroku Logs Parser"
            }
        }
    }
}
