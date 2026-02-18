use crate::app::{InputMode, Message, ViewMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle a keyboard event and convert it to an application message
pub fn handle_key_event(key: KeyEvent, input_mode: InputMode, search_input: &str) -> Option<Message> {
    match input_mode {
        InputMode::Normal => handle_normal_mode(key),
        InputMode::Search => handle_search_mode(key, search_input),
    }
}

/// Handle key events in normal mode
fn handle_normal_mode(key: KeyEvent) -> Option<Message> {
    match (key.code, key.modifiers) {
        // Navigation
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
            Some(Message::ScrollDown)
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => Some(Message::ScrollUp),
        (KeyCode::PageDown, _) => Some(Message::PageDown),
        (KeyCode::PageUp, _) => Some(Message::PageUp),
        (KeyCode::Home, _) | (KeyCode::Char('g'), KeyModifiers::NONE) => {
            Some(Message::ScrollToTop)
        }
        (KeyCode::End, _) | (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
            Some(Message::ScrollToBottom)
        }

        // Actions
        (KeyCode::Char('p'), KeyModifiers::NONE) | (KeyCode::Char(' '), KeyModifiers::NONE) => {
            Some(Message::TogglePause)
        }
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Message::CopyToClipboard),
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => Some(Message::ExportToFile),
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Esc, _) => Some(Message::Quit),

        // Filter controls
        (KeyCode::Char('/'), KeyModifiers::NONE) => Some(Message::EnterSearchMode),
        (KeyCode::Char('f'), KeyModifiers::NONE) => Some(Message::ToggleFilterMode),
        (KeyCode::Char('c'), KeyModifiers::NONE) => Some(Message::ClearFilters),

        // View mode controls
        (KeyCode::Char('1'), KeyModifiers::NONE) => Some(Message::SetViewMode(ViewMode::List)),
        (KeyCode::Char('2'), KeyModifiers::NONE) => Some(Message::SetViewMode(ViewMode::Detail)),
        (KeyCode::Char('3'), KeyModifiers::NONE) => Some(Message::SetViewMode(ViewMode::Split)),

        _ => None,
    }
}

/// Handle key events in search mode
fn handle_search_mode(key: KeyEvent, _search_input: &str) -> Option<Message> {
    match key.code {
        KeyCode::Enter => Some(Message::ExitSearchMode),
        KeyCode::Esc => {
            // Cancel search without applying
            Some(Message::ExitSearchMode)
        }
        _ => {
            // Input handled directly in main loop
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_down() {
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let message = handle_key_event(key, InputMode::Normal, "");
        assert!(matches!(message, Some(Message::ScrollDown)));
    }

    #[test]
    fn test_scroll_up() {
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let message = handle_key_event(key, InputMode::Normal, "");
        assert!(matches!(message, Some(Message::ScrollUp)));
    }

    #[test]
    fn test_quit() {
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let message = handle_key_event(key, InputMode::Normal, "");
        assert!(matches!(message, Some(Message::Quit)));
    }

    #[test]
    fn test_pause() {
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        let message = handle_key_event(key, InputMode::Normal, "");
        assert!(matches!(message, Some(Message::TogglePause)));
    }

    #[test]
    fn test_enter_search_mode() {
        let key = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        let message = handle_key_event(key, InputMode::Normal, "");
        assert!(matches!(message, Some(Message::EnterSearchMode)));
    }

    #[test]
    fn test_view_mode_keys() {
        let key1 = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        let message1 = handle_key_event(key1, InputMode::Normal, "");
        assert!(matches!(
            message1,
            Some(Message::SetViewMode(ViewMode::List))
        ));

        let key2 = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
        let message2 = handle_key_event(key2, InputMode::Normal, "");
        assert!(matches!(
            message2,
            Some(Message::SetViewMode(ViewMode::Detail))
        ));

        let key3 = KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE);
        let message3 = handle_key_event(key3, InputMode::Normal, "");
        assert!(matches!(
            message3,
            Some(Message::SetViewMode(ViewMode::Split))
        ));
    }
}
