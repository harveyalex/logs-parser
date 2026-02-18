use crate::app::{AppState, InputMode, ViewMode};
use crate::parser::LogLevel;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Render the application UI
pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Filter bar
            Constraint::Length(1),  // Status bar
        ])
        .split(area);

    render_header(f, chunks[0], state);

    match state.view_mode {
        ViewMode::List => render_log_list(f, chunks[1], state),
        ViewMode::Detail => render_detail_view(f, chunks[1], state),
        ViewMode::Split => render_split_view(f, chunks[1], state),
    }

    render_filter_bar(f, chunks[2], state);
    render_status_bar(f, chunks[3], state);
}

/// Render the header with title and stats
fn render_header(f: &mut Frame, area: Rect, state: &AppState) {
    let stats = state.stats();

    let title = if state.input_mode == InputMode::Search {
        " Heroku Logs Parser [SEARCH MODE] "
    } else if state.is_paused {
        " Heroku Logs Parser [PAUSED] "
    } else {
        " Heroku Logs Parser "
    };

    let info = format!(
        " Logs: {}/{} | Filters: {} ({:?}) | View: {:?} ",
        stats.filtered_logs,
        stats.total_logs,
        stats.active_filters,
        state.filter_engine.mode(),
        state.view_mode
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_alignment(ratatui::layout::Alignment::Left)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(info)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

/// Render the log list view
fn render_log_list(f: &mut Frame, area: Rect, state: &AppState) {
    let viewport_height = area.height.saturating_sub(2) as usize;
    let visible_logs = state.get_visible_logs(viewport_height);

    let items: Vec<ListItem> = visible_logs
        .iter()
        .map(|log| {
            let color = level_color(log.level);
            let content = log.format_display();

            ListItem::new(Line::from(vec![Span::styled(
                content,
                Style::default().fg(color),
            )]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Logs ")
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

/// Render the detail view showing a single log entry
fn render_detail_view(f: &mut Frame, area: Rect, state: &AppState) {
    let content = if let Some(log) = state.get_selected_log() {
        format!(
            "Timestamp: {}\nSource: {}\nDyno: {}\nLevel: {:?}\n\nMessage:\n{}",
            log.timestamp.to_rfc3339(),
            log.source,
            log.dyno,
            log.level,
            log.message
        )
    } else {
        "No log selected".to_string()
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Detail ")
                .border_style(Style::default().fg(Color::White)),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

/// Render split view with list and detail side by side
fn render_split_view(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_log_list(f, chunks[0], state);
    render_detail_view(f, chunks[1], state);
}

/// Render the filter bar showing active filters
fn render_filter_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let filters = state.filter_engine.filters();

    let content = if filters.is_empty() {
        "No filters active. Press '/' to search, 'f' to toggle AND/OR, 'c' to clear".to_string()
    } else {
        let filter_strings: Vec<String> = filters.iter().map(|f| f.display()).collect();
        format!("Filters: {}", filter_strings.join(" | "))
    };

    let style = if state.input_mode == InputMode::Search {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if state.input_mode == InputMode::Search {
                    format!(" Search: {} ", state.search_input)
                } else {
                    " Filters ".to_string()
                })
                .border_style(style),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

/// Render the status bar at the bottom
fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let status = if let Some(ref msg) = state.status_message {
        msg.clone()
    } else {
        "Press '?' for help | 'q' to quit | '/' to search | 'p' to pause | ↑↓ to scroll"
            .to_string()
    };

    let paragraph = Paragraph::new(status).style(Style::default().fg(Color::Gray));

    f.render_widget(paragraph, area);
}

/// Get the color for a log level
fn level_color(level: LogLevel) -> Color {
    match level {
        LogLevel::Error => Color::Red,
        LogLevel::Warn => Color::Yellow,
        LogLevel::Info => Color::Green,
        LogLevel::Debug => Color::Blue,
        LogLevel::Unknown => Color::White,
    }
}
