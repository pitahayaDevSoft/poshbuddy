use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc;

mod app;
mod ui;
mod api;

use crate::app::{App, AppMessage, AppState, ActiveView};
use crate::ui::ui;
use crate::api::setup_app_task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let (tx, mut rx) = mpsc::channel(32); // Increased to 32 to prevent backpressure

    let themes_dir = app.themes_dir.clone();
    tokio::spawn(setup_app_task(tx.clone(), themes_dir));

    loop {
        if app.state == AppState::Loading {
            app.spinner_tick += 1;
        }
        terminal.draw(|f| ui(f, &mut app))?;

        while let Ok(msg) = rx.try_recv() {
            match msg {
                AppMessage::ThemesLoaded(mut names) => {
                    names.sort();
                    app.themes = names;
                    if app.state == AppState::Loading { app.state = AppState::Main; }
                    // Cargar primer preview
                    if let Some(t) = app.themes.first() {
                        app.load_theme_preview(t.clone(), tx.clone());
                    }
                }
                AppMessage::FontsLoaded(mut fonts) => {
                    fonts.sort_by(|a, b| a.name.cmp(&b.name));
                    app.fonts = fonts;
                }
                AppMessage::ThemePreviewLoaded { theme, preview } => {
                    let filtered = app.filtered_themes();
                    if let Some(selected_index) = app.list_state.selected() {
                        if let Some(current_theme) = filtered.get(selected_index) {
                            if current_theme == &theme {
                                app.theme_preview = preview;
                            }
                        }
                    }
                }
                AppMessage::FontInstalled(name) => {
                    app.state = AppState::FontSuccess(name);
                    app.has_nerd_font = true;
                }
                AppMessage::InstallProgress { line } => {
                    if let AppState::InstallingDependency { log, .. } = &mut app.state {
                        log.push(line.clone());
                        if log.len() > 100 { log.remove(0); }
                        app.state = AppState::InstallingDependency { 
                            current_action: line, 
                            log: log.clone() 
                        };
                    } else {
                        app.state = AppState::InstallingDependency { 
                            current_action: line.clone(), 
                            log: vec![line] 
                        };
                    }
                }
                AppMessage::InstallFinished => {
                    app.state = AppState::Loading;
                    let themes_dir = app.themes_dir.clone();
                    tokio::spawn(setup_app_task(tx.clone(), themes_dir));
                }
                AppMessage::Error(e) => { app.state = AppState::Error(e); }
            }
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != event::KeyEventKind::Press { continue; }
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc { break; }

                if app.state == AppState::DependencyMissing {
                    if key.code == KeyCode::Enter {
                        app.install_omp(tx.clone());
                    }
                }

                if let AppState::Onboarding(_) = app.state {
                    match key.code {
                        KeyCode::Enter => {
                            if app.themes.is_empty() {
                                app.state = AppState::Loading;
                            } else {
                                app.state = AppState::Main;
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        _ => {}
                    }
                    continue;
                }

                if let AppState::Success(_) = app.state {
                    break;
                }

                if let AppState::FontSuccess(_) = app.state {
                    app.state = AppState::Main;
                    continue;
                }

                if app.state == AppState::Main {
                    match key.code {
                        KeyCode::Tab => {
                            app.active_view = if app.active_view == ActiveView::Themes { ActiveView::Fonts } else { ActiveView::Themes };
                        }
                        KeyCode::Char('1') => { app.active_view = ActiveView::Themes; }
                        KeyCode::Char('2') => { app.active_view = ActiveView::Fonts; }
                        KeyCode::Down | KeyCode::Up => {
                            if app.active_view == ActiveView::Themes {
                                let total = app.filtered_themes().len();
                                if total > 0 {
                                    let i = match app.list_state.selected() {
                                        Some(i) => if key.code == KeyCode::Down { (i + 1) % total } else { (i + total - 1) % total },
                                        None => 0,
                                    };
                                    app.list_state.select(Some(i));
                                    if let Some(theme) = app.filtered_themes().get(i) {
                                        app.theme_preview = String::new(); // Clear while loading
                                        app.load_theme_preview(theme.clone(), tx.clone());
                                    }
                                }
                            } else {
                                let total = app.filtered_fonts().len();
                                if total > 0 {
                                    let i = match app.fonts_list_state.selected() {
                                        Some(i) => if key.code == KeyCode::Down { (i + 1) % total } else { (i + total - 1) % total },
                                        None => 0,
                                    };
                                    app.fonts_list_state.select(Some(i));
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if app.active_view == ActiveView::Themes {
                                let filtered = app.filtered_themes();
                                if let Some(selected) = app.list_state.selected() {
                                    if let Some(theme) = filtered.get(selected) {
                                        app.apply_theme(theme)?;
                                        app.state = AppState::Success(theme.clone());
                                    }
                                }
                            } else {
                                let filtered = app.filtered_fonts();
                                if let Some(selected) = app.fonts_list_state.selected() {
                                    if let Some(font) = filtered.get(selected) {
                                        app.state = AppState::Installing(font.name.clone());
                                        app.install_font(font.name.clone(), tx.clone());
                                    }
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            if app.active_view == ActiveView::Themes { 
                                app.filter.push(c); 
                                app.list_state.select(Some(0));
                                if let Some(theme) = app.filtered_themes().first() {
                                    app.theme_preview = String::new();
                                    app.load_theme_preview(theme.clone(), tx.clone());
                                }
                            }
                            else { app.fonts_filter.push(c); app.fonts_list_state.select(Some(0)); }
                        }
                        KeyCode::Backspace => {
                            if app.active_view == ActiveView::Themes { 
                                app.filter.pop(); 
                                app.list_state.select(Some(0));
                                if let Some(theme) = app.filtered_themes().first() {
                                    app.theme_preview = String::new();
                                    app.load_theme_preview(theme.clone(), tx.clone());
                                }
                            }
                            else { app.fonts_filter.pop(); app.fonts_list_state.select(Some(0)); }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
