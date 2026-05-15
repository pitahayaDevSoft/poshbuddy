mod api;
mod app;
mod assets;
mod backup;
mod cli;
mod plugin_installer;
mod ui;

use crate::app::App;
use crate::cli::{Cli, Commands, InstallTarget, ListTarget, SetTarget};
use clap::Parser;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return handle_cli_command(command).await;
    }

    // Launch TUI
    run_tui().await
}

async fn handle_cli_command(command: Commands) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    let (tx, mut rx) = mpsc::channel(64);

    match command {
        Commands::Set { target } => match target {
            SetTarget::Theme { name } => {
                println!("🔍 Searching for theme: {}...", name);
                app.fetch_official_themes(tx.clone());

                while let Some(msg) = rx.recv().await {
                    app.handle_messages(&mut rx, tx.clone());
                    if let crate::app::AppMessage::RemoteThemesLoaded(_) = msg {
                        break;
                    }
                }

                let theme = app.filtered_themes().into_iter().find(|t| t.name == name);

                if let Some(theme) = theme {
                    println!("🚀 Applying theme: {}...", theme.name);
                    app.apply_theme_advanced(theme, tx.clone());

                    while let Some(msg) = rx.recv().await {
                        match msg {
                            crate::app::AppMessage::Success(m) => {
                                println!("✅ {}", m);
                                return Ok(());
                            }
                            crate::app::AppMessage::Error(e) => {
                                println!("❌ Error: {}", e);
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                } else {
                    println!("❌ Error: Theme '{}' not found.", name);
                }
            }
        },
        Commands::Install { target } => match target {
            InstallTarget::Font { name } => {
                println!("📥 Installing font: {}...", name);
                app.install_font(name, tx.clone());
                while let Some(msg) = rx.recv().await {
                    match msg {
                        crate::app::AppMessage::FontInstalled(f) => {
                            println!("✅ Font '{}' installed successfully!", f);
                            return Ok(());
                        }
                        crate::app::AppMessage::Error(e) => {
                            println!("❌ Error: {}", e);
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        },
        Commands::List { target } => match target {
            ListTarget::Themes { local, remote } => {
                let show_all = !local && !remote;
                
                if remote || show_all {
                    println!("🌐 Fetching remote themes catalogue...");
                    app.fetch_official_themes(tx.clone());
                    while let Some(msg) = rx.recv().await {
                        app.handle_messages(&mut rx, tx.clone());
                        if let crate::app::AppMessage::RemoteThemesLoaded(_) = msg {
                            break;
                        }
                    }
                }

                println!("\n{:<30} {:<10}", "THEME NAME", "STATUS");
                println!("{:-<45}", "");
                
                let mut themes = app.filtered_themes();
                themes.sort_by(|a, b| a.name.cmp(&b.name));

                for theme in themes {
                    let is_local = theme.is_local;
                    let status = if is_local { "Local" } else { "Remote" };
                    
                    if (local && is_local) || (remote && !is_local) || show_all {
                        println!("{:<30} {:<10}", theme.name, status);
                    }
                }
                println!();
            }
            ListTarget::Fonts => {
                println!("🌐 Fetching available Nerd Fonts...");
                let themes_dir = app.themes_dir.clone();
                tokio::spawn(crate::api::setup_app_task(tx.clone(), themes_dir));

                while let Some(msg) = rx.recv().await {
                    app.handle_messages(&mut rx, tx.clone());
                    if let crate::app::AppMessage::FontsLoaded(fonts) = msg {
                        println!("\n{:<40}", "AVAILABLE NERD FONTS");
                        println!("{:-<40}", "");
                        for font in fonts {
                            println!("{}", font.name);
                        }
                        println!();
                        break;
                    }
                }
            }
        },
    }

    Ok(())
}

// I realized I need to handle the nested List commands too.
// I'll refine handle_cli_command to be more complete.

async fn run_tui() -> Result<(), Box<dyn Error>> {
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
    tokio::spawn(crate::api::setup_app_task(tx.clone(), themes_dir));
    app.fetch_official_themes(tx.clone());

    // 3. Main Application Loop
    loop {
        app.handle_messages(&mut rx, tx.clone());
        app.spinner_tick += 1;
        terminal.draw(|f| crate::ui::ui(f, &mut app))?;

        if event::poll(Duration::from_millis(30))?
            && let Event::Key(key) = event::read()?
                && app.handle_input(key, tx.clone())? {
                    break;
                }
    }

    // 6. Cleanup terminal state on exit
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
