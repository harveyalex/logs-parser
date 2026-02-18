use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use logs_parser::{
    app::{AppState, InputMode, Message},
    events::handle_key_event,
    ui,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, BufRead};
use tokio::{
    sync::mpsc,
    time::{interval, Duration},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal - use stderr for TUI output so stdin can be piped
    enable_raw_mode().map_err(|e| {
        eprintln!("Failed to enable raw mode: {}", e);
        e
    })?;

    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen).map_err(|e| {
        let _ = disable_raw_mode();
        eprintln!("Failed to enter alternate screen: {}", e);
        e
    })?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // Create application state
    let mut app = AppState::new();

    // Create channels for events
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Create shutdown signal
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

    // Spawn stdin reader task
    let stdin_tx = tx.clone();
    let mut shutdown_stdin = shutdown_rx.clone();
    tokio::task::spawn_blocking(move || {
        use std::io::IsTerminal;
        let stdin = io::stdin();

        // Only read from stdin if it's actually piped input (not a TTY)
        if !stdin.is_terminal() {
            let reader = stdin.lock();
            for line in reader.lines() {
                // Check if we should shutdown
                if *shutdown_stdin.borrow() {
                    break;
                }
                if let Ok(line) = line {
                    if stdin_tx.send(Message::LogLine(line)).is_err() {
                        break;
                    }
                }
            }
        }
        // If stdin is a terminal (no piped input), just exit this task
    });

    // Spawn keyboard event task - read directly from /dev/tty to avoid stdin conflicts
    let event_tx = tx.clone();
    let mut shutdown_events = shutdown_rx.clone();
    tokio::task::spawn_blocking(move || {
        use std::time::Duration;

        // Try to open /dev/tty for reading keyboard input
        // This ensures we read from the terminal even when stdin is piped
        let _tty = std::fs::File::open("/dev/tty").ok();

        loop {
            // Check if we should shutdown
            if *shutdown_events.borrow() {
                break;
            }
            // Poll for events with a short timeout
            if let Ok(true) = event::poll(Duration::from_millis(100)) {
                if let Ok(Event::Key(key)) = event::read() {
                    // Only handle key press events (not release)
                    if key.kind == KeyEventKind::Press {
                        if event_tx.send(Message::KeyPress(key)).is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Tick interval for periodic updates
    let tick_tx = tx;
    let mut shutdown_tick = shutdown_rx.clone();
    tokio::spawn(async move {
        let mut tick_interval = interval(Duration::from_millis(100));
        loop {
            // Check if we should shutdown
            if *shutdown_tick.borrow() {
                break;
            }
            tick_interval.tick().await;
            if tick_tx.send(Message::Tick).is_err() {
                break;
            }
        }
    });

    // Main event loop
    loop {
        // Render UI
        terminal.draw(|f| ui::render(f, &app))?;

        // Handle events
        if let Some(message) = rx.recv().await {
            match message {
                Message::KeyPress(key) => {
                    // Handle text input in search mode
                    if app.input_mode == InputMode::Search {
                        match key.code {
                            KeyCode::Char(c) => {
                                app.search_input.push(c);
                            }
                            KeyCode::Backspace => {
                                app.search_input.pop();
                            }
                            _ => {
                                // Let the event handler process other keys
                                if let Some(msg) =
                                    handle_key_event(key, app.input_mode, &app.search_input)
                                {
                                    app.update(msg);
                                }
                            }
                        }
                    } else {
                        // Normal mode key handling
                        if let Some(msg) =
                            handle_key_event(key, app.input_mode, &app.search_input)
                        {
                            app.update(msg);
                        }
                    }
                }
                msg => {
                    app.update(msg);
                }
            }

            // Check if we should quit
            if app.should_quit {
                // Cleanup terminal immediately
                let _ = disable_raw_mode();
                let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                let _ = terminal.show_cursor();

                // Force exit - this ensures we don't wait for blocking tasks
                std::process::exit(0);
            }
        }
    }

    // Normal cleanup (shouldn't reach here if quit was pressed)
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
