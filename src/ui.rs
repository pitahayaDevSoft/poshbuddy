use crate::app::{ActiveView, App, AppState};
use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Main UI rendering function called for each frame
pub fn ui(f: &mut Frame, app: &mut App) {
    // 1. Root container (Total area)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    // 2. Application Header
    let header = Paragraph::new(" PoshBuddy — Premium Oh My Posh Theme Manager ")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" PoshBuddy v{} ", app.version)),
        );
    f.render_widget(header, chunks[0]);

    // 3. Conditional view rendering based on AppState
    let main_layout = chunks;
    match &app.state {
        // Initial system diagnostic view
        AppState::Onboarding(specs) => {
            let area = main_layout[1];
            let inner_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Length(15),
                    Constraint::Percentage(20),
                ])
                .split(area);

            let font_status = if specs.has_nerd_font {
                "[ √ ] Nerd Font Detected"
            } else {
                "[ ! ] Missing Nerd Font (Icons might be broken)"
            };
            let ps_status = if specs.is_pwsh_7 {
                "[ √ ] PowerShell 7 Detected"
            } else {
                "[ ! ] Windows PowerShell 5.1 (PowerShell 7 recommended)"
            };
            let term_status = if specs.is_windows_terminal {
                "[ √ ] Modern Terminal Detected (Windows Terminal / VS Code)"
            } else {
                "[ ! ] Classic Console (Windows Terminal recommended)"
            };

            let msg = format!(
                "\n  🔍 SYSTEM DIAGNOSTICS\n\n  {}\n  {}\n  {}\n\n  For Oh My Posh themes to render correctly, you need a Nerd Font\n  compatible with your terminal emulator.\n\n  Press [ENTER] to continue to PoshBuddy\n  Press [Q] to quit",
                font_status, ps_status, term_status
            );

            let color = if specs.has_nerd_font {
                Color::Cyan
            } else {
                Color::Yellow
            };

            f.render_widget(
                Paragraph::new(msg)
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" WELCOME TO POSHBUDDY "),
                    )
                    .style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                inner_chunks[1],
            );
        }

        // Generic loading spinner view
        AppState::Loading => {
            let area = main_layout[1];
            let loading_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(3),
                    Constraint::Percentage(40),
                ])
                .split(area);

            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let symbol = spinner_chars[app.spinner_tick % spinner_chars.len()];

            let loading_text = format!("{} Configuring PoshBuddy...", symbol);
            f.render_widget(
                Paragraph::new(loading_text)
                    .alignment(Alignment::Center)
                    .style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                loading_chunks[1],
            );
        }

        // Active installation view for fonts
        AppState::Installing(name) => {
            let area = main_layout[1];
            let p = Paragraph::new(format!("\n\nInstalling font: {}\n\nDownloading assets using Oh My Posh CLI. This may take a moment...", name))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" INSTALACIÓN EN CURSO "))
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(p, area);
        }

        // Global error view
        AppState::Error(msg) => {
            let area = main_layout[1];
            f.render_widget(
                Paragraph::new(format!("Error: {}\n\nPress 'q' to quit.", msg))
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" FAILURE REPORT "),
                    )
                    .style(Style::default().fg(Color::Red)),
                area,
            );
        }

        // Dependency missing view (Oh My Posh not in PATH)
        AppState::DependencyMissing => {
            let area = main_layout[1];
            let msg = "\n   ⚠️  Oh My Posh is not installed or not found in PATH.\n\n   This binary is required to render and apply themes.\n\n   Would you like to install it automatically now?\n\n   [ENTER] Install via Winget (Recommended)\n   [Q/ESC] Quit";
            f.render_widget(
                Paragraph::new(msg)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" MISSING DEPENDENCY "),
                    )
                    .style(Style::default().fg(Color::Yellow)),
                area,
            );
        }

        // Real-time installation log view
        AppState::InstallingDependency {
            current_action,
            log,
        } => {
            let area = main_layout[1];
            let log_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area);

            // 1. Current step status
            f.render_widget(
                Paragraph::new(format!(" >> {}", current_action))
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(Block::default().borders(Borders::BOTTOM)),
                log_chunks[0],
            );

            // 2. Detailed log scroll (Debug mode)
            let log_summary = log.join("\n");
            f.render_widget(
                Paragraph::new(log_summary)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Installation Log "),
                    )
                    .style(Style::default().fg(Color::Gray)),
                log_chunks[1],
            );
        }

        // Final feedback view after theme application
        AppState::Success(theme) => {
            let area = main_layout[1];
            let success_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Length(10),
                    Constraint::Percentage(30),
                ])
                .split(area);

            let msg = format!(
                "\n   🎉 THEME ACTIVATED SUCCESSFULLY!\n\n   The theme '{}' has been configured in your profiles.\n\n   To see the changes, please:\n   1. Close this terminal.\n   2. Open a new PowerShell window.\n\n   (Or run: '. $PROFILE' in your current session)\n\n   [Press any key to exit]",
                theme
            );

            f.render_widget(
                Paragraph::new(msg)
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" POSHBUDDY FEEDBACK "),
                    )
                    .style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                success_chunks[1],
            );
        }

        // Final feedback view after font installation
        AppState::FontSuccess(font) => {
            let area = main_layout[1];
            let font_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Length(10),
                    Constraint::Percentage(30),
                ])
                .split(area);

            let msg = format!(
                "\n   🎉 FONT INSTALLED!\n\n   The font '{}' was successfully added to your system.\n\n   Reload your terminal to start using it!\n   (Remember to set it as the primary font in terminal settings)\n\n   [Press any key to return]",
                font
            );

            f.render_widget(
                Paragraph::new(msg)
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" POSHBUDDY FEEDBACK "),
                    )
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                font_chunks[1],
            );
        }

        // Final feedback view after plugin installation/activation
        AppState::PluginSuccess(plugin) => {
            let area = main_layout[1];
            let plugin_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Length(10),
                    Constraint::Percentage(30),
                ])
                .split(area);

            let msg = format!(
                "\n   🎉 MODULE UPDATED!\n\n   '{}' has been processed.\n\n   Reload your terminal to see the changes!\n   (Run '. $PROFILE' to activate in this session)\n\n   [Press any key to return]",
                plugin
            );

            f.render_widget(
                Paragraph::new(msg)
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" POSHBUDDY FEEDBACK "),
                    )
                    .style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                plugin_chunks[1],
            );
        }

        // Primary application interface (Themes and Fonts explorer)
        AppState::Main => {
            let explorer_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(main_layout[1]);

            // Left Sidebar: Discovery List
            match app.active_view {
                ActiveView::Themes => {
                    let filtered = app.filtered_themes();
                    let themes: Vec<ListItem> =
                        filtered.iter().map(|t| ListItem::new(t.as_str())).collect();
                    let themes_list = List::new(themes)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" Themes Explorer "),
                        )
                        .highlight_style(
                            Style::default()
                                .bg(Color::Blue)
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol(">> ");
                    f.render_stateful_widget(themes_list, explorer_chunks[0], &mut app.list_state);

                    // Right Panel: Split into Preview and Info
                    let panel_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(8), Constraint::Min(0)].as_ref())
                        .split(explorer_chunks[1]);

                    // Top: Rendered Prompt Preview (via ansi-to-tui)
                    let display_preview = if app.theme_preview.is_empty() {
                        "\n    Rendering prompt..."
                            .into_text()
                            .unwrap_or_else(|_| "Loading...".into())
                    } else {
                        app.theme_preview
                            .as_bytes()
                            .into_text()
                            .unwrap_or_else(|_| app.theme_preview.clone().into())
                    };

                    let prompt_box = Paragraph::new(display_preview).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Prompt Visual Preview ")
                            .border_style(Style::default().fg(Color::Yellow)),
                    );
                    f.render_widget(prompt_box, panel_chunks[0]);

                    // Bottom: Theme Documentation and Controls
                    let selected_theme = filtered.get(app.list_state.selected().unwrap_or(0));
                    let mut info_text = format!(
                        "\n  Name: {}\n  Path: ~/.poshthemes/{}\n\n  Profile Sync: {} shells detected\n\n  Controls:\n  [ENTER] Apply theme\n  [TAB]   Browse Fonts\n  [Q/ESC] Quit",
                        selected_theme.unwrap_or(&"None".to_string()),
                        selected_theme.unwrap_or(&"".to_string()),
                        app.detected_profiles.len()
                    );

                    if !app.has_nerd_font {
                        info_text.push_str(
                            "\n\n  ⚠️  Nerd Font not detected. Install one in 'Fonts' tab.",
                        );
                    }

                    let info_panel = Paragraph::new(info_text).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Theme Context "),
                    );
                    f.render_widget(info_panel, panel_chunks[1]);
                }
                ActiveView::Fonts => {
                    // Fonts Explorer view (Secondary tab)
                    let filtered = app.filtered_fonts();
                    let font_items: Vec<ListItem> = filtered
                        .iter()
                        .map(|f| ListItem::new(f.name.as_str()))
                        .collect();
                    let font_list = List::new(font_items)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" Recommended Fonts "),
                        )
                        .highlight_style(
                            Style::default()
                                .bg(Color::Cyan)
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol(">> ");
                    f.render_stateful_widget(
                        font_list,
                        explorer_chunks[0],
                        &mut app.fonts_list_state,
                    );

                    let info_box = Paragraph::new("\n  Select a font to install via Oh My Posh CLI.\n\n  Controls:\n  [ENTER] Download & Install\n  [TAB]   Browse Plugins\n  [Q/ESC] Quit")
                        .block(Block::default().borders(Borders::ALL).title(" Font Manager "));
                    f.render_widget(info_box, explorer_chunks[1]);
                }
                ActiveView::Plugins => {
                    // Plugins (PowerShell Modules) Explorer view
                    let filtered = app.filtered_plugins();
                    let plugin_items: Vec<ListItem> = filtered
                        .iter()
                        .map(|p| {
                            let indicator = if app.is_plugin_active(p) {
                                "[X] "
                            } else {
                                "[ ] "
                            };
                            ListItem::new(format!("{}{}", indicator, p.name))
                        })
                        .collect();

                    let plugin_list = List::new(plugin_items)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" Modules & Extensions "),
                        )
                        .highlight_style(
                            Style::default()
                                .bg(Color::Green)
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol(">> ");
                    f.render_stateful_widget(
                        plugin_list,
                        explorer_chunks[0],
                        &mut app.plugins_list_state,
                    );

                    let selected_idx = app.plugins_list_state.selected().unwrap_or(0);
                    let info_text = if let Some(p) = filtered.get(selected_idx) {
                        format!(
                            "\n  Module: {}\n  Status: {}\n\n  Description:\n  {}\n\n  Usage / Docs:\n  {}\n\n  Controls:\n  [ENTER] Install / Toggle Activation\n  [TAB]   Back to Themes\n  [Q/ESC] Quit",
                            p.name,
                            if app.is_plugin_active(p) { "ACTIVE" } else { "INACTIVE / NOT INSTALLED" },
                            p.description,
                            p.documentation
                        )
                    } else {
                        " No plugins found.".to_string()
                    };

                    let info_box = Paragraph::new(info_text).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Module Documentation "),
                    );
                    f.render_widget(info_box, explorer_chunks[1]);
                }
            }
        }
    }
}
