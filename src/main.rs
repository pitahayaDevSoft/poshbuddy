mod api;
mod app;
mod assets;
mod backup;
mod cli;
mod diagnostic;
mod plugin_installer;
mod ui;

use crate::app::App;
use crate::cli::{Cli, Commands, InstallTarget, ListTarget, SetTarget, ToggleTarget};
use clap::Parser;
use crossterm::{
    cursor::Show,
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
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

/// Drains a dedicated catalog channel until remote themes are loaded or the channel closes.
async fn collect_remote_themes(
    app: &mut App,
    rx: &mut mpsc::Receiver<crate::app::AppMessage>,
) -> bool {
    while let Some(msg) = rx.recv().await {
        if let crate::app::AppMessage::RemoteThemesLoaded(themes) = msg {
            app.remote_themes = themes;
            return true;
        }
    }
    false
}

/// Drains a dedicated catalog channel until fonts are loaded or the channel closes.
async fn collect_remote_fonts(
    app: &mut App,
    rx: &mut mpsc::Receiver<crate::app::AppMessage>,
) -> bool {
    while let Some(msg) = rx.recv().await {
        match msg {
            crate::app::AppMessage::RemoteThemesLoaded(themes) => {
                app.remote_themes = themes;
            }
            crate::app::AppMessage::FontsLoaded(mut fonts) => {
                fonts.sort_by(|a, b| a.name.cmp(&b.name));
                app.fonts = fonts;
                return true;
            }
            _ => {}
        }
    }
    false
}

async fn fetch_remote_themes(app: &mut App) -> bool {
    let (tx, mut rx) = mpsc::channel(16);
    app.fetch_official_themes(tx);
    collect_remote_themes(app, &mut rx).await
}

async fn fetch_remote_fonts(app: &mut App) -> bool {
    let (tx, mut rx) = mpsc::channel(16);
    app.fetch_official_themes(tx);
    collect_remote_fonts(app, &mut rx).await
}

async fn handle_cli_command(command: Commands) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();

    match command {
        Commands::Set { target } => handle_set_command(target, &mut app).await?,
        Commands::Toggle { target } => handle_toggle_command(target, &mut app).await?,
        Commands::Install { target } => handle_install_command(target, &mut app).await?,
        Commands::List { target } => handle_list_command(target, &mut app).await?,
        Commands::Doctor => {
            let report = app.run_diagnostics();
            println!("{}", report);
        }
    }

    Ok(())
}

async fn handle_set_command(target: SetTarget, app: &mut App) -> Result<(), Box<dyn Error>> {
    match target {
        SetTarget::Theme { name } => {
            println!("🔍 Searching for theme: {}...", name);

            let local_match = app
                .themes
                .iter()
                .find(|t| t.name.eq_ignore_ascii_case(&name))
                .cloned();

            let theme = if let Some(t) = local_match {
                Some(t)
            } else {
                if !fetch_remote_themes(app).await {
                    eprintln!("⚠️  Could not fetch remote themes catalogue.");
                }
                app.filtered_themes()
                    .into_iter()
                    .find(|t| t.name.eq_ignore_ascii_case(&name))
            };

            if let Some(theme) = theme {
                println!("🚀 Applying theme: {}...", theme.name);
                let (apply_tx, mut apply_rx) = mpsc::channel(16);
                app.apply_theme_advanced(theme, apply_tx);

                while let Some(msg) = apply_rx.recv().await {
                    match msg {
                        crate::app::AppMessage::Success(m) => {
                            println!("✅ {}", m);
                            return Ok(());
                        }
                        crate::app::AppMessage::Error(e) => {
                            eprintln!("❌ Error: {}", e);
                            return Err(e.into());
                        }
                        _ => {}
                    }
                }
                return Err("Theme application task ended unexpectedly.".into());
            } else {
                eprintln!("❌ Error: Theme '{}' not found.", name);
                return Err(format!("Theme '{}' not found.", name).into());
            }
        }
        SetTarget::Segment { name } => {
            let seg = app
                .segments
                .iter()
                .find(|s| {
                    s.name.eq_ignore_ascii_case(&name) || s.segment_type.eq_ignore_ascii_case(&name)
                })
                .cloned();

            if let Some(s) = seg {
                match app.toggle_segment(&s) {
                    Ok(_) => println!("✅ Segment '{}' toggled successfully!", s.name),
                    Err(e) => {
                        eprintln!("❌ Error toggling segment: {}", e);
                        return Err(e.into());
                    }
                }
            } else {
                eprintln!("❌ Error: Segment '{}' not found.", name);
                return Err(format!("Segment '{}' not found.", name).into());
            }
        }
    }
    Ok(())
}

async fn handle_toggle_command(target: ToggleTarget, app: &mut App) -> Result<(), Box<dyn Error>> {
    match target {
        ToggleTarget::Segment { name } => {
            let seg = app
                .segments
                .iter()
                .find(|s| {
                    s.name.eq_ignore_ascii_case(&name) || s.segment_type.eq_ignore_ascii_case(&name)
                })
                .cloned();

            if let Some(s) = seg {
                match app.toggle_segment(&s) {
                    Ok(_) => println!("✅ Segment '{}' toggled successfully!", s.name),
                    Err(e) => {
                        eprintln!("❌ Error toggling segment: {}", e);
                        return Err(e.into());
                    }
                }
            } else {
                eprintln!("❌ Error: Segment '{}' not found.", name);
                return Err(format!("Segment '{}' not found.", name).into());
            }
        }
    }
    Ok(())
}

async fn handle_install_command(
    target: InstallTarget,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    match target {
        InstallTarget::Font { name } => {
            println!("🌐 Checking Nerd Fonts catalogue...");
            let _ = fetch_remote_fonts(app).await;

            println!("📥 Installing font: {}...", name);
            let (inst_tx, mut inst_rx) = mpsc::channel(16);
            app.install_font(name, inst_tx);

            while let Some(msg) = inst_rx.recv().await {
                match msg {
                    crate::app::AppMessage::FontInstalled(f) => {
                        println!("✅ Font '{}' installed successfully!", f);
                        return Ok(());
                    }
                    crate::app::AppMessage::Error(e) => {
                        eprintln!("❌ Error: {}", e);
                        return Err(e.into());
                    }
                    _ => {}
                }
            }
            Err("Font installation task ended unexpectedly.".into())
        }
    }
}

async fn handle_list_command(target: ListTarget, app: &mut App) -> Result<(), Box<dyn Error>> {
    match target {
        ListTarget::Themes { local, remote } => {
            let show_all = !local && !remote;

            if remote || show_all {
                println!("🌐 Fetching remote themes catalogue...");
                if !fetch_remote_themes(app).await {
                    eprintln!("⚠️  Could not fetch remote themes catalogue.");
                    if remote {
                        return Err("Failed to fetch remote themes catalogue.".into());
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
            if !fetch_remote_fonts(app).await {
                eprintln!("❌ Error: Could not fetch available Nerd Fonts.");
                return Err("Failed to fetch available Nerd Fonts.".into());
            }

            println!("\n{:<40}", "AVAILABLE NERD FONTS");
            println!("{:-<40}", "");
            for font in &app.fonts {
                println!("{}", font.name);
            }
            println!();
        }
        ListTarget::Segments => {
            println!(
                "\n{:<25} {:<15} {:<20} {:<10}",
                "SEGMENT NAME", "TYPE", "CATEGORY", "STATUS"
            );
            println!("{:-<75}", "");
            for seg in &app.segments {
                let active = app.is_segment_active(seg);
                let status = if active { "ENABLED" } else { "DISABLED" };
                println!(
                    "{:<25} {:<15} {:<20} {:<10}",
                    seg.name, seg.segment_type, seg.category, status
                );
            }
            println!();
        }
    }
    Ok(())
}

/// Restores terminal state unconditionally (raw mode, alternate screen, cursor visibility).
fn restore_terminal() {
    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen, Show);
}

/// RAII guard ensuring terminal restoration on normal exit, early error propagation, or unwind.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

async fn run_tui() -> Result<(), Box<dyn Error>> {
    // 0. Install panic hook so terminal is restored even on unexpected panics
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        prev_hook(panic_info);
    }));

    // 1. Setup terminal in raw mode and switch to alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if let Err(e) = execute!(stdout, EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(e.into());
    }
    let _terminal_guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Initialize application state
    let mut app = App::new();

    // Create an mpsc channel for async background tasks to communicate with the UI loop
    let (tx, mut rx) = mpsc::channel(64);

    // Initial fetch of themes, fonts, and official segment schema in the background
    app.fetch_official_themes(tx.clone());

    // 3. Main Application Loop
    loop {
        app.handle_messages(&mut rx, tx.clone());
        app.spinner_tick += 1;
        terminal.draw(|f| crate::ui::ui(f, &mut app))?;

        // Render Kitty Graphics font preview if applicable
        if let Some((x, y)) = app.kitty_preview_position
            && let Some(ref font_name) = app.selected_font_name
            && let Some(base64_image) = app.font_preview_cache.get(font_name)
        {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            let _ = write!(handle, "\x1b[{};{}H", y + 1, x + 1);
            let _ = write!(handle, "\x1b_Ga=T,f=32,m=0,c=45,r=1;{}\x1b\\", base64_image);
            let _ = handle.flush();
        }

        if event::poll(Duration::from_millis(30))?
            && let Event::Key(key) = event::read()?
            && app.handle_input(key, tx.clone())?
        {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use super::*;
    use crate::app::{AppMessage, FontAsset, RemoteTheme};

    #[tokio::test]
    async fn test_collect_remote_themes_populates_app_state() {
        let mut app = App::new();
        app.remote_themes.clear();
        let (tx, mut rx) = mpsc::channel(4);

        tx.send(AppMessage::RemoteThemesLoaded(vec![RemoteTheme {
            name: "bubbles".to_string(),
            download_url: "https://example.com/bubbles.omp.json".to_string(),
            sha: "abc".to_string(),
        }]))
        .await
        .unwrap();
        drop(tx);

        let loaded = collect_remote_themes(&mut app, &mut rx).await;
        assert!(loaded);
        assert_eq!(app.remote_themes.len(), 1);
        assert_eq!(app.remote_themes[0].name, "bubbles");
    }

    #[tokio::test]
    async fn test_collect_remote_themes_returns_false_on_closed_channel_without_deadlock() {
        let mut app = App::new();
        let (tx, mut rx) = mpsc::channel(4);
        drop(tx);

        let loaded = collect_remote_themes(&mut app, &mut rx).await;
        assert!(!loaded);
    }

    #[tokio::test]
    async fn test_collect_remote_fonts_populates_and_sorts_app_fonts() {
        let mut app = App::new();
        app.fonts.clear();
        let (tx, mut rx) = mpsc::channel(4);

        tx.send(AppMessage::FontsLoaded(vec![
            FontAsset {
                name: "JetBrainsMono".to_string(),
                download_url: "https://example.com/jb".to_string(),
            },
            FontAsset {
                name: "FiraCode".to_string(),
                download_url: "https://example.com/fc".to_string(),
            },
        ]))
        .await
        .unwrap();
        drop(tx);

        let loaded = collect_remote_fonts(&mut app, &mut rx).await;
        assert!(loaded);
        assert_eq!(app.fonts.len(), 2);
        assert_eq!(app.fonts[0].name, "FiraCode");
        assert_eq!(app.fonts[1].name, "JetBrainsMono");
    }

    #[tokio::test]
    async fn test_collect_remote_fonts_returns_false_on_closed_channel_without_deadlock() {
        let mut app = App::new();
        let (tx, mut rx) = mpsc::channel(4);
        drop(tx);

        let loaded = collect_remote_fonts(&mut app, &mut rx).await;
        assert!(!loaded);
    }
}
