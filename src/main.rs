use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
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
mod ui;

use crate::api::setup_app_task;
use crate::app::{ActiveView, App, AppMessage, AppState};
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
    let (tx, mut rx) = mpsc::channel(32);

    // Initial fetch of themes and fonts in the background
    let themes_dir = app.themes_dir.clone();
    tokio::spawn(setup_app_task(tx.clone(), themes_dir));

    // 3. Main Application Loop
    loop {
        // Handle tick-based state updates (like spinner animations)
        if app.state == AppState::Loading {
            app.spinner_tick += 1;
        }

        // Draw the TUI frame
        terminal.draw(|f| ui(f, &mut app))?;

        // 4. Handle Incoming Messages from Background Tasks
        while let Ok(msg) = rx.try_recv() {
            match msg {
                AppMessage::ThemesLoaded(mut names) => {
                    names.sort();
                    app.themes = names;
                    // Transition to Main view if we were loading
                    if app.state == AppState::Loading {
                        app.state = AppState::Main;
                    }
                    // Pre-load the first theme preview
                    if let Some(t) = app.themes.first() {
                        app.load_theme_preview(t.clone(), tx.clone());
                    }
                }
                AppMessage::FontsLoaded(mut fonts) => {
                    fonts.sort_by(|a, b| a.name.cmp(&b.name));
                    app.fonts = fonts;
                }
                AppMessage::ThemePreviewLoaded { theme, preview } => {
                    // Only update preview if it corresponds to the currently selected theme
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
                    // Transition to FontSuccess screen
                    app.state = AppState::FontSuccess(name);
                    app.has_nerd_font = true;
                }
                AppMessage::PluginInstalled(name) => {
                    // Transition to PluginSuccess screen
                    app.state = AppState::PluginSuccess(name);
                }
                AppMessage::InstallProgress { line } => {
                    // Update the debug log and current action for the installation view
                    if let AppState::InstallingDependency { log, .. } = &mut app.state {
                        log.push(line.clone());
                        if log.len() > 100 {
                            log.remove(0);
                        }
                        app.state = AppState::InstallingDependency {
                            current_action: line,
                            log: log.clone(),
                        };
                    } else {
                        app.state = AppState::InstallingDependency {
                            current_action: line.clone(),
                            log: vec![line],
                        };
                    }
                }
                AppMessage::InstallFinished => {
                    // After OMP install, reload themes
                    app.state = AppState::Loading;
                    let themes_dir = app.themes_dir.clone();
                    tokio::spawn(setup_app_task(tx.clone(), themes_dir));
                }
                AppMessage::Error(e) => {
                    app.state = AppState::Error(e);
                }
            }
        }

        // 5. User Input Handling (keyboard events)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Filter for key press events to avoid double-triggering on Windows
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Dependency checking state: allow starting install or exit
                if app.state == AppState::DependencyMissing {
                    if key.code == KeyCode::Enter {
                        app.install_omp(tx.clone());
                    }
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        break;
                    }
                    continue;
                }

                // Initial diagnostic screen: allow continuing to Main or exit
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

                // Final success screen: Exit on any key
                if let AppState::Success(_) = app.state {
                    break;
                }

                // Font success screen: Return to main menu on any key
                if let AppState::FontSuccess(_) = app.state {
                    app.state = AppState::Main;
                    continue;
                }

                // Plugin success screen: Return to main menu on any key
                if let AppState::PluginSuccess(_) = app.state {
                    app.state = AppState::Main;
                    app.active_view = ActiveView::Plugins;
                    continue;
                }

                // Main navigation and controls
                if app.state == AppState::Main {
                    match key.code {
                        KeyCode::Tab => {
                            // Cycle between Themes, Fonts and Plugins views
                            app.active_view = match app.active_view {
                                ActiveView::Themes => ActiveView::Fonts,
                                ActiveView::Fonts => ActiveView::Plugins,
                                ActiveView::Plugins => ActiveView::Themes,
                            };
                        }
                        KeyCode::Char('1') => {
                            app.active_view = ActiveView::Themes;
                        }
                        KeyCode::Char('2') => {
                            app.active_view = ActiveView::Fonts;
                        }
                        KeyCode::Char('3') => {
                            app.active_view = ActiveView::Plugins;
                        }
                        KeyCode::Down | KeyCode::Up => {
                            // Horizontal focus logic for selection changes
                            if app.active_view == ActiveView::Themes {
                                let filtered = app.filtered_themes();
                                let i = match app.list_state.selected() {
                                    Some(i) => {
                                        if key.code == KeyCode::Down {
                                            if i >= filtered.len() - 1 {
                                                0
                                            } else {
                                                i + 1
                                            }
                                        } else {
                                            if i == 0 {
                                                filtered.len() - 1
                                            } else {
                                                i - 1
                                            }
                                        }
                                    }
                                    None => 0,
                                };
                                app.list_state.select(Some(i));
                                // Request a preview update for the newly selected theme
                                app.theme_preview = String::new();
                                if let Some(t) = filtered.get(i) {
                                    app.load_theme_preview(t.clone(), tx.clone());
                                }
                            } else if app.active_view == ActiveView::Fonts {
                                // Font selection navigation
                                let filtered = app.filtered_fonts();
                                let i = match app.fonts_list_state.selected() {
                                    Some(i) => {
                                        if key.code == KeyCode::Down {
                                            if i >= filtered.len() - 1 {
                                                0
                                            } else {
                                                i + 1
                                            }
                                        } else {
                                            if i == 0 {
                                                filtered.len() - 1
                                            } else {
                                                i - 1
                                            }
                                        }
                                    }
                                    None => 0,
                                };
                                app.fonts_list_state.select(Some(i));
                            } else if app.active_view == ActiveView::Plugins {
                                // Plugin selection navigation
                                let filtered = app.filtered_plugins();
                                let i = match app.plugins_list_state.selected() {
                                    Some(i) => {
                                        if key.code == KeyCode::Down {
                                            if i >= filtered.len() - 1 {
                                                0
                                            } else {
                                                i + 1
                                            }
                                        } else {
                                            if i == 0 {
                                                filtered.len() - 1
                                            } else {
                                                i - 1
                                            }
                                        }
                                    }
                                    None => 0,
                                };
                                app.plugins_list_state.select(Some(i));
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char(c) => {
                            // Filtering logic for the list
                            if app.active_view == ActiveView::Themes {
                                app.filter.push(c);
                                app.list_state.select(Some(0));
                            } else if app.active_view == ActiveView::Fonts {
                                app.fonts_filter.push(c);
                                app.fonts_list_state.select(Some(0));
                            } else {
                                app.plugins_filter.push(c);
                                app.plugins_list_state.select(Some(0));
                            }
                        }
                        KeyCode::Backspace => {
                            if app.active_view == ActiveView::Themes {
                                app.filter.pop();
                            } else if app.active_view == ActiveView::Fonts {
                                app.fonts_filter.pop();
                            } else {
                                app.plugins_filter.pop();
                            }
                        }
                        KeyCode::Enter => {
                            if app.active_view == ActiveView::Themes {
                                let filtered = app.filtered_themes();
                                if let Some(selected) = app.list_state.selected() {
                                    if let Some(theme) = filtered.get(selected) {
                                        // Update profiles and show success screen
                                        app.apply_theme(theme)?;
                                        app.state = AppState::Success(theme.clone());
                                    }
                                }
                            } else if app.active_view == ActiveView::Fonts {
                                let filtered = app.filtered_fonts();
                                if let Some(selected) = app.fonts_list_state.selected() {
                                    if let Some(font) = filtered.get(selected) {
                                        // Start font installation
                                        app.state = AppState::Installing(font.name.clone());
                                        app.install_font(font.name.clone(), tx.clone());
                                    }
                                }
                            } else {
                                let filtered = app.filtered_plugins();
                                if let Some(selected) = app.plugins_list_state.selected() {
                                    if let Some(plugin) = filtered.get(selected) {
                                        // Toggle plugin activation
                                        if let Err(e) = app.toggle_plugin(plugin) {
                                            app.state = AppState::Error(format!(
                                                "Failed to update profile: {}",
                                                e
                                            ));
                                        } else {
                                            app.state =
                                                AppState::PluginSuccess(plugin.name.clone());
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
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
