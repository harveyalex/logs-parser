use dioxus::prelude::*;

#[component]
pub fn StatsHeader(total_logs: usize, filtered_logs: usize, filter_mode_and: bool) -> Element {
    let filter_text = if filter_mode_and { "AND" } else { "OR" };

    rsx! {
        div {
            style: "background: #1a1a1a; color: #fff; padding: 12px 16px; border-bottom: 2px solid #4a9eff; display: flex; justify-content: space-between; align-items: center;",

            div {
                style: "display: flex; gap: 24px;",

                div {
                    span {
                        style: "color: #888; font-size: 12px;",
                        "Total Logs: "
                    }
                    span {
                        style: "color: #4a9eff; font-weight: bold; font-size: 14px;",
                        "{total_logs}"
                    }
                }

                div {
                    span {
                        style: "color: #888; font-size: 12px;",
                        "Filtered: "
                    }
                    span {
                        style: "color: #50c878; font-weight: bold; font-size: 14px;",
                        "{filtered_logs}"
                    }
                }

                div {
                    span {
                        style: "color: #888; font-size: 12px;",
                        "Filter Mode: "
                    }
                    span {
                        style: "color: #ffa500; font-weight: bold; font-size: 14px;",
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
