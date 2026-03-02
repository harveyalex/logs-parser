//! UI components for desktop app

pub mod connection_panel;
pub mod custom_select;
pub mod filter_bar;
pub mod log_view;
pub mod stats_header;
mod status_indicator;

pub use connection_panel::ConnectionPanel;
pub use filter_bar::FilterBar;
pub use log_view::LogView;
pub use stats_header::StatsHeader;
pub use status_indicator::{ConnectionStatus, LoadingStep, StatusIndicator};
