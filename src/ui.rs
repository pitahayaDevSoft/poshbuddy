use crate::app::{ActiveView, App, AppState};
use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Main UI rendering function called for each frame
pub fn ui(f: &mut Frame, app: &mut App) {
    // 1. Root container (Total area)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main Content
            Constraint::Length(3), // Expanded Global Footer
        ].as_ref())
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
    match &app.state {
        AppState::Onboarding(specs) => {
            render_onboarding(f, chunks[1], specs);
        }
        AppState::Welcome => {
            render_welcome(f, chunks[1], app);
        }
        AppState::Main | AppState::Success(_) | AppState::Error(_) | AppState::Installing(_) | AppState::FontSuccess(_) | AppState::PluginSuccess(_) => {
            render_main_dashboard(f, chunks[1], app);
        }
        AppState::Loading => {
            let msg = Paragraph::new("\n\n  🚀 Loading PoshBuddy Core...\n  Analyzing themes, fonts, and shell profiles.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(msg, chunks[1]);
        }
        _ => {}
    }

    // 4. Global Footer (High-Density Status Bar)
    render_footer(f, chunks[2], app);
}

fn render_onboarding(f: &mut Frame, area: Rect, specs: &crate::app::SystemSpecs) {
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Length(15),
            Constraint::Percentage(20),
        ])
        .split(area);

    let font_status = if specs.has_nerd_font { "[ √ ] Nerd Font Detected" } else { "[ ! ] Missing Nerd Font (Icons might be broken)" };
    let ps_status = if specs.is_pwsh_7 { "[ √ ] PowerShell 7 Detected" } else { "[ ! ] Windows PowerShell 5.1 (PowerShell 7 recommended)" };
    let term_status = if specs.is_windows_terminal { "[ √ ] Modern Terminal Detected" } else { "[ ! ] Classic Console (Windows Terminal recommended)" };

    let msg = format!(
        "\n  🔍 SYSTEM DIAGNOSTICS\n\n  {}\n  {}\n  {}\n\n  Press [ENTER] to continue\n  Press [Q] to quit",
        font_status, ps_status, term_status
    );

    let color = if specs.has_nerd_font { Color::Cyan } else { Color::Yellow };

    f.render_widget(
        Paragraph::new(msg)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" WELCOME "))
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
        inner_chunks[1],
    );
}

fn render_welcome(f: &mut Frame, area: Rect, app: &App) {
    let welcome_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Sidebar menu
            Constraint::Percentage(70), // Resource display
        ])
        .split(area);

    // Left Sidebar: Quick Actions
    let actions = [(" 1 ", "Tema Aleatorio", "Enter"),
        (" 2 ", "Instalar Nerd Font", "f"),
        (" 3 ", "Terminal-Icons", "i"),
        (" 4 ", "Diagnóstico", "d"),
        (" 5 ", "Ver Backups", "b"),
        (" 6 ", "Ir a Temas", "t"),
        (" 7 ", "Ir a Fuentes", "F"),
        (" 8 ", "Ir a Segmentos", "p")];

    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, (id, label, _))| {
            let style = if i == app.welcome_selected_action {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(format!("{} {}", id, label)).style(style)
        })
        .collect();

    let sidebar = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Quick Actions "))
        .highlight_symbol(">> ");
    f.render_widget(sidebar, welcome_chunks[0]);

    // Right Content: Resources & Info
    let info = vec![
        Line::from(vec![Span::styled(" Useful Resources ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(vec![Span::raw(" - "), Span::styled("Oh My Posh Docs: ", Style::default().fg(Color::Blue)), Span::raw("https://ohmyposh.dev")]),
        Line::from(vec![Span::raw(" - "), Span::styled("Nerd Fonts: ", Style::default().fg(Color::Blue)), Span::raw("https://nerdfonts.com")]),
        Line::from(vec![Span::raw(" - "), Span::styled("PowerShell Profile: ", Style::default().fg(Color::Blue)), Span::raw("$PROFILE")]),
        Line::from(""),
        Line::from(vec![Span::styled(" Tips: ", Style::default().fg(Color::Green))]),
        Line::from(" Use [Tab] to navigate between sections efficiently."),
        Line::from(" PoshBuddy automatically backups your profile before any edit."),
    ];

    let content = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" Information "))
        .wrap(Wrap { trim: true });
    f.render_widget(content, welcome_chunks[1]);
}

fn render_main_dashboard(f: &mut Frame, area: Rect, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Navigation Sidebar
            Constraint::Min(0),         // Dynamic Content Area
        ])
        .split(area);

    // Sidebar Navigation
    let tabs = [" [T] Temas ", " [F] Fuentes ", " [S] Segmentos "];
    let items: Vec<ListItem> = tabs
        .iter()
        .enumerate()
        .map(|(i, &t)| {
            let view = match i {
                0 => ActiveView::Themes,
                1 => ActiveView::Fonts,
                _ => ActiveView::Segments,
            };

            if app.active_view == view {
                ListItem::new(t).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::REVERSED))
            } else {
                ListItem::new(t).style(Style::default().fg(Color::Gray))
            }
        })
        .collect();

    let sidebar = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Navigation "));
    f.render_widget(sidebar, main_chunks[0]);

    // Right Content based on ActiveView
    match app.active_view {
        ActiveView::Themes => render_themes_view(f, main_chunks[1], app),
        ActiveView::Fonts => render_fonts_view(f, main_chunks[1], app),
        ActiveView::Segments => render_segments_view(f, main_chunks[1], app),
    }
}

fn render_themes_view(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(8), // Preview
        ])
        .split(area);

    let themes = app.filtered_themes();
    let mut theme_items = Vec::new();

    for t in themes {
        if t.is_local {
            theme_items.push(ListItem::new(format!(" [L] {}", t.name)).style(Style::default().fg(Color::Cyan)));
        } else {
            theme_items.push(ListItem::new(format!(" [G] {}", t.name)).style(Style::default().fg(Color::Gray)));
        }
    }

    let items_count = theme_items.len();
    let title = if app.filter.is_empty() {
        format!(" Themes Catalog ({}) ", items_count)
    } else {
        format!(" Themes Catalog ({}) [Filter: {}] ", items_count, app.filter)
    };
    let list = List::new(theme_items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.list_state);

    let preview_text = if app.theme_preview.is_empty() { " Loading preview..." } else { &app.theme_preview };
    let preview = Paragraph::new(preview_text.into_text().unwrap_or_default())
        .block(Block::default().borders(Borders::ALL).title(" Active Theme Preview "));
    f.render_widget(preview, chunks[1]);
}

fn render_fonts_view(f: &mut Frame, area: Rect, app: &mut App) {
    let fonts = app.filtered_fonts();
    let items: Vec<ListItem> = fonts
        .iter()
        .map(|f| ListItem::new(format!("  • {}", f.name)))
        .collect();

    let title = if app.fonts_filter.is_empty() {
        format!(" Nerd Fonts Available ({}) ", fonts.len())
    } else {
        format!(" Nerd Fonts Available ({}) [Filter: {}] ", fonts.len(), app.fonts_filter)
    };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.fonts_list_state);
}

fn render_segments_view(f: &mut Frame, area: Rect, app: &mut App) {
    let segments = app.filtered_segments();
    let items: Vec<ListItem> = segments
        .iter()
        .map(|s| {
            let active = if app.is_segment_active(s) { " [X] " } else { " [ ] " };
            ListItem::new(format!("{}{}", active, s.name))
        })
        .collect();

    let title = if app.segments_filter.is_empty() {
        format!(" OMP Segments ({}) ", segments.len())
    } else {
        format!(" OMP Segments ({}) [Filter: {}] ", segments.len(), app.segments_filter)
    };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.plugins_list_state);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let current_time = chrono::Local::now().format("%H:%M:%S").to_string();
    let theme_name = app.active_config_path.as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("Default");

    let footer_text = format!(
        " [1] Dashboard | [2] Temas | [3] Segmentos | [Tab] Switch | [Q] Quit  ────  🎨 Theme: {}  |  🕒 {}  |  👤 {} ",
        theme_name,
        current_time,
        whoami::username()
    );

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Black)),
        );
    f.render_widget(footer, area);
}
