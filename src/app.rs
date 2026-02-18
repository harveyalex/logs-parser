use crate::buffer::CircularBuffer;
use crate::filters::{FilterEngine, FilterMode};
use crate::parser::{parse_log_line, LogEntry};

/// View mode for displaying logs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// List view showing all logs in a scrollable list
    List,
    /// Detail view showing a single log entry with full information
    Detail,
    /// Split view showing both list and detail
    Split,
}

/// Input mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal mode for navigation and commands
    Normal,
    /// Search input mode for entering filter queries
    Search,
}

/// Messages that can be sent to update the application state
#[derive(Debug, Clone)]
pub enum Message {
    /// A new log line arrived from stdin
    LogLine(String),
    /// User pressed a key
    KeyPress(crossterm::event::KeyEvent),
    /// Scroll up in the log list
    ScrollUp,
    /// Scroll down in the log list
    ScrollDown,
    /// Scroll to the top
    ScrollToTop,
    /// Scroll to the bottom
    ScrollToBottom,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Toggle pause/resume streaming
    TogglePause,
    /// Add a filter
    AddFilter(String),
    /// Clear all filters
    ClearFilters,
    /// Toggle filter mode (AND/OR)
    ToggleFilterMode,
    /// Change view mode
    SetViewMode(ViewMode),
    /// Enter search mode
    EnterSearchMode,
    /// Exit search mode
    ExitSearchMode,
    /// Copy to clipboard
    CopyToClipboard,
    /// Export to file
    ExportToFile,
    /// Quit the application
    Quit,
    /// Tick for periodic updates
    Tick,
}

/// Application state
pub struct AppState {
    /// Log buffer storing all log entries
    pub log_buffer: CircularBuffer,
    /// Filtered log indices (indices into log_buffer)
    pub filtered_logs: Vec<usize>,
    /// Current scroll offset in the filtered logs
    pub scroll_offset: usize,
    /// Currently selected log index in filtered view
    pub selected_index: usize,
    /// Whether log streaming is paused
    pub is_paused: bool,
    /// Filter engine for filtering logs
    pub filter_engine: FilterEngine,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Current input mode
    pub input_mode: InputMode,
    /// Search input buffer
    pub search_input: String,
    /// Status message to display
    pub status_message: Option<String>,
    /// Whether the application should quit
    pub should_quit: bool,
}

impl AppState {
    /// Create a new application state with default buffer size
    pub fn new() -> Self {
        Self::with_capacity(10_000)
    }

    /// Create a new application state with specified buffer capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            log_buffer: CircularBuffer::new(capacity),
            filtered_logs: Vec::new(),
            scroll_offset: 0,
            selected_index: 0,
            is_paused: false,
            filter_engine: FilterEngine::new(),
            view_mode: ViewMode::List,
            input_mode: InputMode::Normal,
            search_input: String::new(),
            status_message: None,
            should_quit: false,
        }
    }

    /// Update the application state based on a message
    pub fn update(&mut self, message: Message) {
        match message {
            Message::LogLine(line) => {
                if !self.is_paused {
                    if let Some(entry) = parse_log_line(&line) {
                        self.log_buffer.push(entry);
                        self.recompute_filtered_logs();

                        // Auto-scroll to bottom if we're already at the bottom
                        if self.is_at_bottom() {
                            self.scroll_to_bottom();
                        }
                    }
                }
            }
            Message::ScrollUp => {
                // Scroll up = go back in history = increase offset
                self.scroll_offset += 1;
                self.clamp_scroll();
            }
            Message::ScrollDown => {
                // Scroll down = go forward to recent = decrease offset
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            Message::ScrollToTop => {
                // Top = oldest logs = maximum offset
                if !self.filtered_logs.is_empty() {
                    self.scroll_offset = self.filtered_logs.len().saturating_sub(1);
                }
                self.selected_index = 0;
            }
            Message::ScrollToBottom => {
                // Bottom = newest logs = offset 0
                self.scroll_to_bottom();
            }
            Message::PageUp => {
                // Page up = go back in history
                self.scroll_offset = self.scroll_offset.saturating_add(20);
                self.clamp_scroll();
            }
            Message::PageDown => {
                // Page down = go forward to recent
                self.scroll_offset = self.scroll_offset.saturating_sub(20);
            }
            Message::TogglePause => {
                self.is_paused = !self.is_paused;
                self.set_status(if self.is_paused {
                    "Paused".to_string()
                } else {
                    "Resumed".to_string()
                });
            }
            Message::AddFilter(query) => {
                // Simple text search filter for now
                use crate::filters::Filter;
                self.filter_engine.add_filter(Filter::TextSearch(query.clone()));
                self.recompute_filtered_logs();
                self.set_status(format!("Added filter: {}", query));
            }
            Message::ClearFilters => {
                self.filter_engine.clear();
                self.recompute_filtered_logs();
                self.set_status("Filters cleared".to_string());
            }
            Message::ToggleFilterMode => {
                self.filter_engine.toggle_mode();
                self.recompute_filtered_logs();
                let mode = match self.filter_engine.mode() {
                    FilterMode::And => "AND",
                    FilterMode::Or => "OR",
                };
                self.set_status(format!("Filter mode: {}", mode));
            }
            Message::SetViewMode(mode) => {
                self.view_mode = mode;
            }
            Message::EnterSearchMode => {
                self.input_mode = InputMode::Search;
                self.search_input.clear();
                self.set_status("SEARCH MODE - Type your query and press Enter".to_string());
            }
            Message::ExitSearchMode => {
                self.input_mode = InputMode::Normal;
                if !self.search_input.is_empty() {
                    self.update(Message::AddFilter(self.search_input.clone()));
                }
                self.search_input.clear();
            }
            Message::CopyToClipboard => {
                let visible_logs = self.get_all_filtered_logs();
                match crate::export::copy_to_clipboard(&visible_logs) {
                    Ok(msg) => self.set_status(msg),
                    Err(e) => self.set_status(format!("Copy failed: {}", e)),
                }
            }
            Message::ExportToFile => {
                let visible_logs = self.get_all_filtered_logs();
                match crate::export::export_to_file(&visible_logs) {
                    Ok(msg) => self.set_status(msg),
                    Err(e) => self.set_status(format!("Export failed: {}", e)),
                }
            }
            Message::Quit => {
                self.should_quit = true;
            }
            Message::Tick => {
                // Periodic update - can be used for animations, etc.
            }
            Message::KeyPress(_) => {
                // Handled by event handler
            }
        }
    }

    /// Recompute the filtered logs based on current filters
    fn recompute_filtered_logs(&mut self) {
        let all_logs: Vec<&LogEntry> = self.log_buffer.all();
        self.filtered_logs = self.filter_engine.filter_indices(&all_logs);
    }

    /// Check if we're at the bottom of the log list
    fn is_at_bottom(&self) -> bool {
        if self.filtered_logs.is_empty() {
            return true;
        }
        // Consider at bottom if scroll_offset is 0 or within 5 entries of the end
        self.scroll_offset == 0 || self.scroll_offset + 5 >= self.filtered_logs.len()
    }

    /// Scroll to the bottom of the log list
    fn scroll_to_bottom(&mut self) {
        // Keep scroll at 0 to show most recent logs (they're added at the end)
        self.scroll_offset = 0;
    }

    /// Clamp scroll offset to valid range
    /// Maximum offset is (total_logs - 1) to show oldest logs
    fn clamp_scroll(&mut self) {
        if !self.filtered_logs.is_empty() {
            let max_offset = self.filtered_logs.len().saturating_sub(1);
            if self.scroll_offset > max_offset {
                self.scroll_offset = max_offset;
            }
        }
    }

    /// Set a status message
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clear the status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Get the currently visible logs for rendering
    /// Shows most recent logs at the bottom (scroll_offset=0 shows latest)
    pub fn get_visible_logs(&self, viewport_height: usize) -> Vec<&LogEntry> {
        if self.filtered_logs.is_empty() {
            return Vec::new();
        }

        let total = self.filtered_logs.len();

        // Calculate the window to show
        // scroll_offset=0 means show the most recent logs (from the end)
        // scroll_offset=10 means show logs from 10 entries back from the end
        let end = total.saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(viewport_height);

        self.filtered_logs[start..end]
            .iter()
            .filter_map(|&idx| self.log_buffer.get(idx))
            .collect()
    }

    /// Get all filtered logs (for export/clipboard)
    pub fn get_all_filtered_logs(&self) -> Vec<&LogEntry> {
        self.filtered_logs
            .iter()
            .filter_map(|&idx| self.log_buffer.get(idx))
            .collect()
    }

    /// Get the currently selected log entry
    pub fn get_selected_log(&self) -> Option<&LogEntry> {
        if self.selected_index < self.filtered_logs.len() {
            let buffer_idx = self.filtered_logs[self.selected_index];
            self.log_buffer.get(buffer_idx)
        } else {
            None
        }
    }

    /// Get statistics for display
    pub fn stats(&self) -> Stats {
        Stats {
            total_logs: self.log_buffer.len(),
            filtered_logs: self.filtered_logs.len(),
            buffer_capacity: self.log_buffer.capacity(),
            active_filters: self.filter_engine.len(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the application state
#[derive(Debug, Clone)]
pub struct Stats {
    pub total_logs: usize,
    pub filtered_logs: usize,
    pub buffer_capacity: usize,
    pub active_filters: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_app_state() {
        let state = AppState::new();
        assert_eq!(state.scroll_offset, 0);
        assert!(!state.is_paused);
        assert_eq!(state.view_mode, ViewMode::List);
        assert!(!state.should_quit);
    }

    #[test]
    fn test_add_log_line() {
        let mut state = AppState::new();
        let line = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Test message".to_string();

        state.update(Message::LogLine(line));

        assert_eq!(state.log_buffer.len(), 1);
        assert_eq!(state.filtered_logs.len(), 1);
    }

    #[test]
    fn test_pause_resume() {
        let mut state = AppState::new();

        assert!(!state.is_paused);

        state.update(Message::TogglePause);
        assert!(state.is_paused);

        state.update(Message::TogglePause);
        assert!(!state.is_paused);
    }

    #[test]
    fn test_pause_blocks_new_logs() {
        let mut state = AppState::new();
        let line = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Test message".to_string();

        state.update(Message::TogglePause);
        state.update(Message::LogLine(line));

        assert_eq!(state.log_buffer.len(), 0);
    }

    #[test]
    fn test_scroll_up_down() {
        let mut state = AppState::new();

        // Add some logs
        for i in 0..10 {
            let line = format!(
                "2010-09-16T15:13:46.677020+00:00 app[web.1]: Message {}",
                i
            );
            state.update(Message::LogLine(line));
        }

        state.scroll_offset = 5;

        // ScrollUp = go back in history = increase offset
        state.update(Message::ScrollUp);
        assert_eq!(state.scroll_offset, 6);

        // ScrollDown = go forward to recent = decrease offset
        state.update(Message::ScrollDown);
        assert_eq!(state.scroll_offset, 5);
    }

    #[test]
    fn test_add_filter() {
        let mut state = AppState::new();

        // Add some logs
        let line1 = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Error message".to_string();
        let line2 = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Info message".to_string();

        state.update(Message::LogLine(line1));
        state.update(Message::LogLine(line2));

        assert_eq!(state.filtered_logs.len(), 2);

        state.update(Message::AddFilter("error".to_string()));

        assert_eq!(state.filtered_logs.len(), 1);
        assert_eq!(state.filter_engine.len(), 1);
    }

    #[test]
    fn test_clear_filters() {
        let mut state = AppState::new();

        state.update(Message::AddFilter("error".to_string()));
        assert_eq!(state.filter_engine.len(), 1);

        state.update(Message::ClearFilters);
        assert_eq!(state.filter_engine.len(), 0);
    }

    #[test]
    fn test_quit() {
        let mut state = AppState::new();

        assert!(!state.should_quit);

        state.update(Message::Quit);
        assert!(state.should_quit);
    }

    #[test]
    fn test_view_mode() {
        let mut state = AppState::new();

        assert_eq!(state.view_mode, ViewMode::List);

        state.update(Message::SetViewMode(ViewMode::Detail));
        assert_eq!(state.view_mode, ViewMode::Detail);

        state.update(Message::SetViewMode(ViewMode::Split));
        assert_eq!(state.view_mode, ViewMode::Split);
    }
}
