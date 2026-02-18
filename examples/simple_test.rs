// Simple terminal test to verify Ghostty compatibility
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use std::time::Duration;

fn main() -> io::Result<()> {
    println!("Starting terminal test...");
    println!("Enabling raw mode...");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    println!("Terminal initialized. Press any key (q to quit)...\r");
    stdout.flush()?;

    loop {
        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                println!("Key pressed: {:?}\r", key);
                stdout.flush()?;

                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    println!("Test complete!");

    Ok(())
}
