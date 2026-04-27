use crate::app::{ActiveView, App, AppState};
use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

// ── Design tokens ─────────────────────────────────────────────────────────────
const C_ACCENT: Color = Color::Cyan;
const C_LOCAL: Color = Color::Green;
const C_REMOTE: Color = Color::Blue;
const C_ACTIVE: Color = Color::Yellow;
const C_ERROR: Color = Color::Red;
const C_DIM: Color = Color::DarkGray;
const C_WHITE: Color = Color::White;
const C_BLACK: Color = Color::Black;

const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

// ── Root dispatcher ────────────────────────────────────────────────────────────
pub fn ui(f: &mut Frame, app: &mut App) {
    match app.state.clone() {
        AppState::Welcome => render_welcome(f, f.size(), app),
        AppState::Onboarding(specs) => render_onboarding(f, f.size(), &specs),
        AppState::DependencyMissing => render_dep_missing(f, f.size()),
        AppState::Loading => render_loading(f, f.size(), app),
        AppState::InstallingDependency {
            log,
            current_action,
        } => {
            render_installing_dep(f, f.size(), &log, &current_action);
        }
        _ => render_main(f, f.size(), app),
    }

    render_overlays(f, app);
}

// ═══════════════════════════════════════════════════════════════════════════════
//  MAIN VIEW
// ═══════════════════════════════════════════════════════════════════════════════

fn render_main(f: &mut Frame, area: Rect, app: &mut App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Length(3), // tab bar
            Constraint::Min(0),    // content
            Constraint::Length(1), // footer
        ])
        .split(area);

    render_title_bar(f, root[0], app);
    render_tab_bar(f, root[1], app);

    match app.active_view {
        ActiveView::Themes => render_themes(f, root[2], app),
        ActiveView::Fonts => render_fonts(f, root[2], app),
        ActiveView::Segments => render_segments(f, root[2], app),
    }

    render_main_footer(f, root[3], app);

    // Floating modals — rendered on top of everything
    match &app.state {
        AppState::Success(msg) => {
            render_modal(f, area, " ✓ Applied ", msg, C_ACTIVE, "any key");
        }
        AppState::FontSuccess(name) => {
            render_modal(
                f,
                area,
                " ✓ Font Installed ",
                &format!("'{}' installed successfully.", name),
                C_LOCAL,
                "any key to continue",
            );
        }
        AppState::PluginSuccess(name) => {
            render_modal(
                f,
                area,
                " ✓ Segment Toggled ",
                &format!("'{}' toggled in your active theme.", name),
                C_LOCAL,
                "any key to continue",
            );
        }
        AppState::Installing(name) => {
            render_modal(
                f,
                area,
                " ⏳ Working ",
                &format!("Processing: {}\n\nThis may take a moment...", name),
                C_ACCENT,
                "please wait",
            );
        }
        AppState::Error(msg) => {
            render_modal(f, area, " ✗ Error ", msg, C_ERROR, "any key");
        }
        AppState::ApplyingProgress {
            name,
            stage,
            progress,
        } => {
            let title = match stage {
                0 => " ⬇ Downloading ",
                1 => " 🔍 Verifying ",
                2 => " 💾 Backing up ",
                3 => " ⚡ Applying ",
                _ => " ⏳ Working ",
            };
            let msg = format!("Theme: {}\n\nProgress: {}%", name, progress);
            render_modal(f, area, title, &msg, C_ACCENT, "please wait");
        }
        _ => {}
    }
}

// ── Title bar (1 line, no border) ─────────────────────────────────────────────
fn render_title_bar(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    // Left: brand
    f.render_widget(
        Paragraph::new(format!("  PoshBuddy v{}", app.version))
            .style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
        cols[0],
    );

    // Centre: active theme
    let theme = app
        .active_config_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("no theme");
    f.render_widget(
        Paragraph::new(format!(" 🎨 {}", theme))
            .alignment(Alignment::Center)
            .style(Style::default().fg(C_WHITE)),
        cols[1],
    );

    // Right: user + clock
    let time = chrono::Local::now().format("%H:%M").to_string();
    let user = whoami::username();
    f.render_widget(
        Paragraph::new(format!("{}  {}  ", user, time))
            .alignment(Alignment::Right)
            .style(Style::default().fg(C_DIM)),
        cols[2],
    );
}

// ── Tab bar (3 lines with border bottom) ──────────────────────────────────────
fn render_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    let tabs: &[(&str, ActiveView)] = &[
        ("  [1] Themes ", ActiveView::Themes),
        ("  [2] Fonts  ", ActiveView::Fonts),
        ("  [3] Segments", ActiveView::Segments),
    ];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    for (i, (label, view)) in tabs.iter().enumerate() {
        let is_active = app.active_view == *view;
        let count = match view {
            ActiveView::Themes => app.filtered_themes_count(),
            ActiveView::Fonts => app.filtered_fonts_count(),
            ActiveView::Segments => app.filtered_segments_count(),
        };

        // Text composition
        let count_text = format!("({}) ", count);
        let text = Line::from(vec![
            Span::styled(
                label.to_string(),
                Style::default().add_modifier(if is_active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            ),
            Span::raw(" "),
            Span::styled(
                count_text,
                Style::default().fg(if is_active { C_BLACK } else { C_DIM }),
            ),
        ]);

        let (fg, bg, border_fg) = if is_active {
            (C_BLACK, C_ACCENT, C_ACCENT)
        } else {
            (C_WHITE, Color::Reset, C_DIM)
        };

        f.render_widget(
            Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(fg).bg(bg))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_fg)),
                ),
            chunks[i],
        );
    }
}

// ── Context-sensitive footer (1 line, no border) ──────────────────────────────
fn render_main_footer(f: &mut Frame, area: Rect, app: &App) {
    let is_filtering = match app.active_view {
        ActiveView::Themes => !app.filter.is_empty(),
        ActiveView::Fonts => !app.fonts_filter.is_empty(),
        ActiveView::Segments => !app.segments_filter.is_empty(),
    };

    let esc_action = if is_filtering { "Esc Clear Search" } else { "Esc/H Dashboard" };

    let hint = match app.active_view {
        ActiveView::Themes =>
            format!("  ↑↓ Navigate  │  Enter Apply  │  Type Search  │  {}  │  Tab Next Tab  │  Ctrl+R Restore  │  Q Quit", esc_action),
        ActiveView::Fonts =>
            format!("  ↑↓ Navigate  │  Enter Install  │  Type Search  │  {}  │  Tab Next Tab  │  Ctrl+R Restore  │  Q Quit", esc_action),
        ActiveView::Segments =>
            format!("  ↑↓ Navigate  │  Enter Toggle  │  Type Search  │  {}  │  Tab Next Tab  │  Ctrl+R Restore  │  Q Quit", esc_action),
    };
    f.render_widget(Paragraph::new(hint).style(Style::default().fg(C_DIM)), area);
}

// ── Floating modal ────────────────────────────────────────────────────────────
fn render_modal(f: &mut Frame, area: Rect, title: &str, msg: &str, color: Color, dismiss: &str) {
    let w = area.width.min(58);
    let h = 7u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let modal = Rect::new(x, y, w, h);

    f.render_widget(Clear, modal);
    f.render_widget(
        Paragraph::new(format!("\n  {}\n\n  Press {} to dismiss.", msg, dismiss))
            .style(Style::default().fg(color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))
                    .title(title),
            )
            .wrap(Wrap { trim: true }),
        modal,
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
//  THEMES VIEW
// ═══════════════════════════════════════════════════════════════════════════════

fn render_themes(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // 1. Clean Header
    f.render_widget(
        Paragraph::new("\n[ THEMES EXPLORER ]")
            .alignment(Alignment::Center)
            .style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
        chunks[0],
    );

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // 2. Left column: search + list
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(cols[0]);

    render_search_bar(f, left[0], &app.filter, "Themes");

    let themes = app.filtered_themes();
    let mut items: Vec<ListItem> = themes
        .iter()
        .map(|t| {
            let label = if t.is_local { "[Local]" } else { "[Remote]" };
            let style = if t.is_local {
                Style::default().fg(C_LOCAL)
            } else {
                Style::default().fg(C_REMOTE)
            };
            ListItem::new(format!("  {} {}", t.name, label)).style(style)
        })
        .collect();

    let is_empty = items.is_empty();
    let title = if app.filter.is_empty() {
        " Themes List "
    } else {
        " Themes List (Filtered) "
    };

    if is_empty {
        let msg = if app.filter.is_empty() {
            "  No themes available.".to_string()
        } else {
            format!("  No themes matching '{}'", app.filter)
        };
        items.push(ListItem::new(msg).style(Style::default().fg(C_DIM)));
    }

    let mut list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_ACCENT))
            .title(title),
    );

    if !is_empty {
        list = list
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(C_WHITE)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▶ ");
    }

    f.render_stateful_widget(list, left[1], &mut app.list_state);

    // 3. Right column: preview
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_ACCENT))
        .title(" ANSI Preview ");

    if app.theme_preview.is_empty() {
        f.render_widget(
            Paragraph::new("\n  Select a theme to see preview...")
                .style(Style::default().fg(C_DIM))
                .block(preview_block),
            cols[1],
        );
    } else {
        let preview_text = app.theme_preview.as_str().into_text().unwrap_or_default();
        f.render_widget(
            Paragraph::new(preview_text)
                .block(preview_block)
                .wrap(Wrap { trim: false }),
            cols[1],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  FONTS VIEW
// ═══════════════════════════════════════════════════════════════════════════════

fn render_fonts(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Header
    f.render_widget(
        Paragraph::new("\n[ FONT MANAGER ]")
            .alignment(Alignment::Center)
            .style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
        chunks[0],
    );

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Left: search + list
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(cols[0]);

    render_search_bar(f, left[0], &app.fonts_filter, "Fonts");

    let fonts = app.filtered_fonts();
    let mut items: Vec<ListItem> = fonts
        .iter()
        .map(|font| ListItem::new(format!("  {}", font.name)).style(Style::default().fg(C_WHITE)))
        .collect();

    let is_empty = items.is_empty();
    let title = if app.fonts_filter.is_empty() {
        " Available Fonts "
    } else {
        " Available Fonts (Filtered) "
    };

    if is_empty {
        let msg = if app.fonts_filter.is_empty() {
            "  No fonts available.".to_string()
        } else {
            format!("  No fonts matching '{}'", app.fonts_filter)
        };
        items.push(ListItem::new(msg).style(Style::default().fg(C_DIM)));
    }

    let mut list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_ACCENT))
            .title(title),
    );

    if !is_empty {
        list = list
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(C_WHITE)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▶ ");
    }

    f.render_stateful_widget(list, left[1], &mut app.fonts_list_state);

    // Right: detail panel
    let selected = app.fonts_list_state.selected().and_then(|i| fonts.get(i));
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_DIM))
        .title(" Font Details ");

    if let Some(font) = selected {
        let lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {}", font.name),
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Type:     ", Style::default().fg(C_DIM)),
                Span::raw("Nerd Font"),
            ]),
            Line::from(vec![
                Span::styled("  Action:   ", Style::default().fg(C_DIM)),
                Span::styled("Press [Enter] to install", Style::default().fg(C_LOCAL)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Note: After installing, you must update your terminal settings.",
                Style::default().fg(C_DIM),
            )]),
        ];
        f.render_widget(Paragraph::new(lines).block(detail_block), cols[1]);
    } else {
        f.render_widget(
            Paragraph::new("\n  Select a font to continue...")
                .style(Style::default().fg(C_DIM))
                .block(detail_block),
            cols[1],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  SEGMENTS VIEW
// ═══════════════════════════════════════════════════════════════════════════════

fn render_segments(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Header
    f.render_widget(
        Paragraph::new("\n[ SEGMENT MANAGER ]")
            .alignment(Alignment::Center)
            .style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
        chunks[0],
    );

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    // Left: search + list
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(cols[0]);

    render_search_bar(f, left[0], &app.segments_filter, "Segments");

    let segments = app.filtered_segments();
    let mut items: Vec<ListItem> = segments
        .iter()
        .map(|s| {
            let active = app.is_segment_active(s);
            let dot = if active { "●" } else { "○" };
            let style = if active {
                Style::default().fg(C_ACTIVE)
            } else {
                Style::default().fg(C_WHITE)
            };
            ListItem::new(format!("  {} {}", dot, s.name)).style(style)
        })
        .collect();

    let is_empty = items.is_empty();
    let title = if app.segments_filter.is_empty() {
        " Components "
    } else {
        " Components (Filtered) "
    };

    if is_empty {
        let msg = if app.segments_filter.is_empty() {
            "  No components available.".to_string()
        } else {
            format!("  No components matching '{}'", app.segments_filter)
        };
        items.push(ListItem::new(msg).style(Style::default().fg(C_DIM)));
    }

    let mut list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_ACCENT))
            .title(title),
    );

    if !is_empty {
        list = list
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(C_WHITE)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▶ ");
    }

    f.render_stateful_widget(list, left[1], &mut app.plugins_list_state);

    // Right: detail
    let selected = app
        .plugins_list_state
        .selected()
        .and_then(|i| segments.get(i));
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_DIM))
        .title(" Information ");

    if let Some(seg) = selected {
        let active = app.is_segment_active(seg);
        let lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {}", seg.name),
                Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status:   ", Style::default().fg(C_DIM)),
                Span::styled(
                    if active { "ENABLED" } else { "DISABLED" },
                    if active {
                        Style::default().fg(C_ACTIVE)
                    } else {
                        Style::default().fg(C_DIM)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("  Category: ", Style::default().fg(C_DIM)),
                Span::raw(seg.category.clone()),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Description:",
                Style::default().fg(C_DIM),
            )]),
            Line::from(format!("  {}", seg.description)),
        ];
        f.render_widget(
            Paragraph::new(lines)
                .block(detail_block)
                .wrap(Wrap { trim: true }),
            cols[1],
        );
    } else {
        f.render_widget(
            Paragraph::new("\n  Select a component to toggle...")
                .style(Style::default().fg(C_DIM))
                .block(detail_block),
            cols[1],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  WELCOME SCREEN
// ═══════════════════════════════════════════════════════════════════════════════

fn render_welcome(f: &mut Frame, area: Rect, app: &App) {
    // 1. Responsive Guard: If terminal is extremely small, show a simplified message
    if area.width < 40 || area.height < 10 {
        f.render_widget(
            Paragraph::new("Terminal too small to display Dashboard.\nPlease resize your window.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(C_ERROR)),
            area,
        );
        return;
    }

    // 2. Dynamic Constraints based on available height
    let has_space_for_logo = area.height > 20;

    let constraints = if has_space_for_logo {
        vec![
            Constraint::Length(9), // Logo
            Constraint::Length(3), // Dashboard Title
            Constraint::Fill(1),   // Stats & Actions
            Constraint::Length(3), // Next Step Hint
            Constraint::Length(1), // Footer
        ]
    } else {
        vec![
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Dashboard Title (shorter)
            Constraint::Fill(1),   // Stats & Actions
            Constraint::Length(2), // Next Step Hint (shorter)
            Constraint::Length(1), // Footer
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut next_chunk_idx = 0;

    // Render Logo if space permits
    if has_space_for_logo {
        // All lines must have the exact same width for perfect centering
        let logo = r#"  _____           _     ____            _     _        
 |  __ \         | |   |  _ \          | |   | |       
 | |__) |__  ___ | |__ | |_) |_   _  __| | __| |_   _  
 |  ___/ _ \/ __|| '_ \|  _ <| | | |/ _` |/ _` | | | | 
 | |  | (_) \__ \| | | | |_) | |_| | (_| | (_| | |_| | 
 |_|   \___/|___/|_| |_|____/ \__,_|\__,_|\__,_|\__, | 
                                                 __/ | 
                                                |___/  "#;
        f.render_widget(
            Paragraph::new(logo)
                .alignment(Alignment::Center)
                .style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
            chunks[next_chunk_idx],
        );
        next_chunk_idx += 1;
    }

    // Dashboard Header
    f.render_widget(
        Paragraph::new("[ DASHBOARD ]")
            .alignment(Alignment::Center)
            .style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
        chunks[next_chunk_idx],
    );
    next_chunk_idx += 1;

    // 3. Stats & Actions (Dynamic Side-by-Side or Stacked)
    let is_narrow = area.width < 90;
    let body_area = chunks[next_chunk_idx];
    next_chunk_idx += 1;

    let body_chunks = if is_narrow {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)])
            .split(body_area)
    } else {
        // Center the content on wide screens to avoid massive boxes
        let centered_body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(100),
                Constraint::Min(0),
            ])
            .split(body_area)[1];

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Length(2), // Spacer
                Constraint::Percentage(50),
            ])
            .split(centered_body)
    };

    let left_area = body_chunks[0];
    let right_area = if is_narrow {
        body_chunks[1]
    } else {
        body_chunks[2]
    };

    // Left Column: Stats + Changelog
    let left_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(left_area);

    // System Info
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "User".to_string());
    let os = std::env::consts::OS;
    let mut sys_info = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  User:        ", Style::default().fg(C_DIM)),
            Span::styled(
                username,
                Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" @ {}", os), Style::default().fg(C_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  Status:      ", Style::default().fg(C_DIM)),
            Span::styled("󱐋 ", Style::default().fg(C_LOCAL)),
            Span::styled(
                "READY",
                Style::default().fg(C_LOCAL).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Backups:     ", Style::default().fg(C_DIM)),
            Span::styled(
                format!("{} profiles", app.total_backups),
                Style::default().fg(C_WHITE),
            ),
        ]),
    ];

    if let Some(s) = &app.system_specs {
        sys_info.push(Line::from(vec![
            Span::styled("  Nerd Font:   ", Style::default().fg(C_DIM)),
            if s.has_nerd_font {
                Span::styled("󰄬 ", Style::default().fg(C_LOCAL))
            } else {
                Span::styled("󰅖 ", Style::default().fg(C_ERROR))
            },
            Span::styled(
                if s.has_nerd_font {
                    "Detected"
                } else {
                    "Missing"
                },
                Style::default().fg(if s.has_nerd_font { C_LOCAL } else { C_ERROR }),
            ),
        ]));
    }

    f.render_widget(
        Paragraph::new(sys_info).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_DIM))
                .title(" System Identity "),
        ),
        left_column[0],
    );

    // Changelog
    if left_column[1].height > 3 {
        let changelog = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  v0.4.1 ",
                    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::styled("- The 'Rice' Update 󰓅", Style::default().fg(C_WHITE)),
            ]),
            Line::from(vec![
                Span::styled("  󰄬 ", Style::default().fg(C_ACCENT)),
                Span::raw("Modernized TUI with responsive 50/50 layout"),
            ]),
            Line::from(vec![
                Span::styled("  󰄬 ", Style::default().fg(C_ACCENT)),
                Span::raw("Strict Nerd Font integration (No Emojis)"),
            ]),
            Line::from(vec![
                Span::styled("  󰄬 ", Style::default().fg(C_ACCENT)),
                Span::raw("Pixel-perfect ASCII logo alignment"),
            ]),
        ];

        f.render_widget(
            Paragraph::new(changelog).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(C_DIM))
                    .title(" Latest Changes "),
            ),
            left_column[1],
        );
    }

    // Right Column: Quick Steps
    let action_labels = [
        (" Explore Themes ", "T", "1"),
        (" Install Fonts  ", "F", "2"),
        (" Manage Segments ", "S", "3"),
        (" Randomize Style ", "R", "4"),
        (" Install ALL Fonts ", "N", "5"),
        (" Terminal Icons  ", "I", "6"),
        (" Diagnostics     ", "D", "7"),
        (" Manual Backup   ", "B", "8"),
    ];

    let mut actions = vec![Line::from("")];
    for (i, (label, mnemonic, key)) in action_labels.iter().enumerate() {
        let is_selected = i == app.welcome_selected_action;
        let is_disabled = i == 6; // Diagnostics soon

        let (key_style, label_style) = if is_disabled && is_selected {
            (
                Style::default().fg(C_DIM).bg(Color::DarkGray).add_modifier(Modifier::BOLD),
                Style::default().fg(C_DIM).bg(Color::DarkGray),
            )
        } else if is_disabled {
            (
                Style::default().fg(C_DIM),
                Style::default().fg(C_DIM),
            )
        } else if is_selected {
            (
                Style::default().fg(C_BLACK).bg(C_ACCENT).add_modifier(Modifier::BOLD),
                Style::default().fg(C_WHITE).bg(Color::DarkGray).add_modifier(Modifier::BOLD),
            )
        } else {
            (
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                Style::default().fg(C_WHITE),
            )
        };

        actions.push(Line::from(vec![
            Span::styled(format!("  [{}]", key), key_style),
            Span::styled(format!(" {} ({})", label, mnemonic), label_style),
        ]));
    }

    f.render_widget(
        Paragraph::new(actions).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_DIM))
                .title(" Quick Steps "),
        ),
        right_area,
    );

    // 4. Next Step Hint
    f.render_widget(
        Paragraph::new(if is_narrow {
            "Use [1-8, T, F, S, R, N, I, D, B] to navigate"
        } else {
            "\nUse keys [1-8] or mnemonics [T, F, S, R, N, I, D, B] to navigate..."
        })
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ),
        chunks[next_chunk_idx],
    );
    next_chunk_idx += 1;

    // 5. Footer
    f.render_widget(
        Paragraph::new(format!("PoshBuddy v{} │ julesklord", app.version))
            .alignment(Alignment::Center)
            .style(Style::default().fg(C_DIM)),
        chunks[next_chunk_idx],
    );
}

fn render_overlays(f: &mut Frame, app: &App) {
    let area = f.size();

    // 1. Confirm Mass Font Installation
    if app.state == AppState::ConfirmMassFontInstallation {
        let modal_area = centered_rect(60, 25, area);
        f.render_widget(Clear, modal_area);
        let block = Block::default()
            .title(" Confirm Mass Installation ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_ACCENT));

        let text = vec![
            Line::from(""),
            Line::from("  You are about to install ALL Nerd Fonts available."),
            Line::from("  This process may take significant time and bandwidth."),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Do you want to proceed? "),
                Span::styled(
                    "(y) Yes",
                    Style::default().fg(C_LOCAL).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" / "),
                Span::styled(
                    "(n) No",
                    Style::default().fg(C_ERROR).add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        f.render_widget(Paragraph::new(text).block(block), modal_area);
    }

    // 2. Installation Progress Gauge
    if let AppState::InstallingAllFonts {
        progress,
        current_font,
        index,
        total,
    } = &app.state
    {
        let modal_area = centered_rect(70, 20, area);
        f.render_widget(Clear, modal_area);
        let block = Block::default()
            .title(format!(" Installing Nerd Fonts ({}/{}) ", index, total))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_ACCENT));

        let gauge = ratatui::widgets::Gauge::default()
            .block(Block::default().padding(Padding::new(2, 2, 1, 1)))
            .gauge_style(Style::default().fg(C_ACCENT).bg(C_DIM))
            .percent(*progress as u16)
            .label(format!("{:.1}%", progress));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(block.inner(modal_area));

        f.render_widget(block, modal_area);
        f.render_widget(
            Paragraph::new(vec![Line::from(vec![
                Span::raw("  Current: "),
                Span::styled(
                    current_font,
                    Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
                ),
            ])]),
            layout[0],
        );
        f.render_widget(gauge, layout[1]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  ONBOARDING / LOADING / DEPENDENCY MISSING
// ═══════════════════════════════════════════════════════════════════════════════

fn render_onboarding(f: &mut Frame, area: Rect, specs: &crate::app::SystemSpecs) {
    let center = centered_rect(58, 52, area);
    f.render_widget(Clear, center);

    let rows: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  System Diagnostics",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        status_line(
            "  Nerd Font   ",
            specs.has_nerd_font,
            "Detected",
            "Not found — icons may break",
        ),
        status_line(
            "  PowerShell  ",
            specs.is_pwsh_7,
            "v7 (pwsh)",
            "v5.1 — PowerShell 7 recommended",
        ),
        status_line(
            "  Terminal    ",
            specs.is_windows_terminal,
            "Windows Terminal",
            "Classic Console — upgrade recommended",
        ),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Press Enter to continue  ·  Q to quit",
            Style::default().fg(C_DIM),
        )]),
    ];

    f.render_widget(
        Paragraph::new(rows).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_ACCENT))
                .title(" Welcome to PoshBuddy "),
        ),
        center,
    );
}

fn render_loading(f: &mut Frame, area: Rect, app: &App) {
    let spin = SPINNER[app.spinner_tick % SPINNER.len()];
    let center = centered_rect(50, 30, area);
    f.render_widget(Clear, center);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} Loading PoshBuddy...", spin),
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Scanning themes, fonts, and shell profiles.",
                Style::default().fg(C_DIM),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_ACCENT)),
        ),
        center,
    );
}

fn render_dep_missing(f: &mut Frame, area: Rect) {
    let center = centered_rect(62, 44, area);
    f.render_widget(Clear, center);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Oh My Posh not found",
                Style::default().fg(C_ERROR).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  PoshBuddy requires Oh My Posh to work.",
                Style::default().fg(C_DIM),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Enter  ",
                    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::styled("Auto-install via WinGet", Style::default().fg(C_WHITE)),
            ]),
            Line::from(vec![
                Span::styled("  Q      ", Style::default().fg(C_DIM)),
                Span::styled("Quit and install manually", Style::default().fg(C_DIM)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Manual: ", Style::default().fg(C_DIM)),
                Span::styled(
                    "winget install JanDeDobbeleer.OhMyPosh",
                    Style::default().fg(C_ACTIVE),
                ),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_ERROR))
                .title(" Missing Dependency "),
        )
        .wrap(Wrap { trim: true }),
        center,
    );
}

fn render_installing_dep(f: &mut Frame, area: Rect, log: &[String], current: &str) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    f.render_widget(
        Paragraph::new(format!("  {}", current))
            .style(Style::default().fg(C_ACTIVE))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Installing Oh My Posh "),
            ),
        root[0],
    );

    let log_lines: Vec<Line> = log.iter().map(|l| Line::from(format!("  {}", l))).collect();
    f.render_widget(
        Paragraph::new(log_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_DIM))
                .title(" Installation Log "),
        ),
        root[1],
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
//  SHARED HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Renders a search bar with visible cursor when filter is active
fn render_search_bar(f: &mut Frame, area: Rect, filter: &str, context: &str) {
    let (text, style) = if filter.is_empty() {
        (
            format!("  Search {}...", context.to_lowercase()),
            Style::default().fg(C_DIM),
        )
    } else {
        (format!("  {}_", filter), Style::default().fg(C_WHITE))
    };

    let title = if filter.is_empty() {
        " / Search ".to_string()
    } else {
        " / Search (Esc to clear) ".to_string()
    };

    f.render_widget(
        Paragraph::new(text).style(style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if filter.is_empty() {
                    Style::default().fg(C_DIM)
                } else {
                    Style::default().fg(C_ACCENT)
                })
                .title(title),
        ),
        area,
    );
}

/// Returns a status line with ●/○ indicator and color coding
fn status_line<'a>(label: &'a str, ok: bool, ok_msg: &'a str, warn_msg: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(C_DIM)),
        if ok {
            Span::styled(format!("● {}", ok_msg), Style::default().fg(C_LOCAL))
        } else {
            Span::styled(format!("○ {}", warn_msg), Style::default().fg(C_ACTIVE))
        },
    ])
}

/// Centers a rect of given percentage within parent
fn centered_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(vert[1])[1]
}
