use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::error::Error;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

mod api;
mod app;
mod backup;
mod diagnostic;
mod plugin_installer;
mod ui;

use crate::api::setup_app_task;
use crate::app::App;
use crate::ui::ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Setup terminal in raw mode and switch to alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Initialize application state
    let mut app = App::new();

    // Create an mpsc channel for async background tasks to communicate with the UI loop
    let (tx, mut rx) = mpsc::channel(64);

    // Initial fetch of themes and fonts in the background
    let themes_dir = app.themes_dir.clone();
    tokio::spawn(setup_app_task(tx.clone(), themes_dir));
    app.fetch_official_themes(tx.clone());

    // 3. Main Application Loop
    loop {
        // A. Handle Incoming Messages (Prioritize state updates before drawing)
        app.handle_messages(&mut rx, tx.clone());

        // B. Handle tick-based state updates (animations)
        // Usamos un contador global para animaciones más fluidas
        app.spinner_tick += 1;

        // C. Draw the TUI frame
        terminal.draw(|f| ui(f, &mut app))?;

        // D. User Input Handling (keyboard events)
        // Reducimos el poll a 30ms para una respuesta mucho más ágil (aprox 33 fps)
        if event::poll(Duration::from_millis(30))? {
            if let Event::Key(key) = event::read()? {
                if app.handle_input(key, tx.clone())? {
                    break;
                }
            }
        }
    }

    // 6. Cleanup terminal state on exit
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
