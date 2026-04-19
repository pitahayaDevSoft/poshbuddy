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
            ActiveView::Themes => app.filtered_themes().len(),
            ActiveView::Fonts => app.filtered_fonts().len(),
            ActiveView::Segments => app.filtered_segments().len(),
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
    let hint = match app.active_view {
        ActiveView::Themes =>
            "  ↑↓ Navigate  │  Enter Apply  │  Type Search  │  Esc/H Dashboard  │  Tab Next Tab  │  Ctrl+R Restore  │  Q Quit",
        ActiveView::Fonts =>
            "  ↑↓ Navigate  │  Enter Install  │  Type Search  │  Esc/H Dashboard  │  Tab Next Tab  │  Ctrl+R Restore  │  Q Quit",
        ActiveView::Segments =>
            "  ↑↓ Navigate  │  Enter Toggle  │  Type Search  │  Esc/H Dashboard  │  Tab Next Tab  │  Ctrl+R Restore  │  Q Quit",
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
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left column: search + list
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(cols[0]);

    render_search_bar(f, left[0], &app.filter, "Themes");

    let themes = app.filtered_themes();
    let n_local = app.themes.len();
    let n_remote = app.remote_themes.len();

    let mut items: Vec<ListItem> = themes
        .iter()
        .map(|t| {
            if t.is_local {
                ListItem::new(format!("  L  {}", t.name)).style(Style::default().fg(C_LOCAL))
            } else {
                ListItem::new(format!("  R  {}", t.name)).style(Style::default().fg(C_REMOTE))
            }
        })
        .collect();

    if items.is_empty() {
        let msg = if app.filter.is_empty() {
            "  No themes available.".to_string()
        } else {
            format!("  No themes matching '{}'", app.filter)
        };
        items.push(
            ListItem::new(msg).style(Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC)),
        );
    }

    let title = if app.filter.is_empty() {
        format!(" Themes  L:{}  R:{} ", n_local, n_remote)
    } else {
        format!(
            " Themes  L:{}  R:{} [Filter: {}] ",
            n_local, n_remote, app.filter
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_ACCENT))
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(C_WHITE)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▶ ");

    f.render_stateful_widget(list, left[1], &mut app.list_state);

    // Right column: legend + preview
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(cols[1]);

    // Badge legend
    let legend = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            " L ",
            Style::default()
                .fg(C_BLACK)
                .bg(C_LOCAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Local  ", Style::default().fg(C_DIM)),
        Span::styled(
            " R ",
            Style::default()
                .fg(C_BLACK)
                .bg(C_REMOTE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " Remote — Enter to download & apply  ",
            Style::default().fg(C_DIM),
        ),
        Span::styled(
            "Enter",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" = Apply", Style::default().fg(C_DIM)),
    ]);
    f.render_widget(
        Paragraph::new(legend).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_DIM)),
        ),
        right[0],
    );

    // Preview pane
    if app.theme_preview.is_empty() {
        f.render_widget(
            Paragraph::new("\n  Loading preview...")
                .style(Style::default().fg(C_DIM))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(C_ACCENT))
                        .title(" Preview "),
                ),
            right[1],
        );
    } else {
        let preview_text = app.theme_preview.as_str().into_text().unwrap_or_default();
        f.render_widget(
            Paragraph::new(preview_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(C_ACCENT))
                        .title(" Preview "),
                )
                .wrap(Wrap { trim: false }),
            right[1],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  FONTS VIEW
// ═══════════════════════════════════════════════════════════════════════════════

fn render_fonts(f: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left: search + list
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(cols[0]);

    render_search_bar(f, left[0], &app.fonts_filter, "Fonts");

    let fonts = app.filtered_fonts();
    let mut items: Vec<ListItem> = fonts
        .iter()
        .map(|font| ListItem::new(format!("   {}", font.name)).style(Style::default().fg(C_WHITE)))
        .collect();

    if items.is_empty() {
        let msg = if app.fonts_filter.is_empty() {
            "  No fonts available.".to_string()
        } else {
            format!("  No fonts matching '{}'", app.fonts_filter)
        };
        items.push(
            ListItem::new(msg).style(Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC)),
        );
    }

    let title = if app.fonts_filter.is_empty() {
        format!(" Nerd Fonts ({}) ", fonts.len())
    } else {
        format!(
            " Nerd Fonts ({}) [Filter: {}] ",
            fonts.len(),
            app.fonts_filter
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_ACCENT))
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(C_WHITE)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▶ ");

    f.render_stateful_widget(list, left[1], &mut app.fonts_list_state);

    // Right: detail panel
    let selected = app.fonts_list_state.selected().and_then(|i| fonts.get(i));

    let detail: Vec<Line> = if let Some(font) = selected {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {}", font.name),
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Type    ", Style::default().fg(C_DIM)),
                Span::raw("Nerd Font (icon-patched)"),
            ]),
            Line::from(vec![
                Span::styled("  Source  ", Style::default().fg(C_DIM)),
                Span::styled("nerdfonts.com", Style::default().fg(C_ACCENT)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press ", Style::default().fg(C_DIM)),
                Span::styled(
                    "Enter",
                    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " to install via oh-my-posh font install",
                    Style::default().fg(C_DIM),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  After installing, restart your terminal.",
                Style::default().fg(C_DIM),
            )]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Select a font to see details.",
                Style::default().fg(C_DIM),
            )]),
        ]
    };

    f.render_widget(
        Paragraph::new(detail).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_DIM))
                .title(" Font Detail "),
        ),
        cols[1],
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
//  SEGMENTS VIEW
// ═══════════════════════════════════════════════════════════════════════════════

fn render_segments(f: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

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
            let cat_col = match s.category.as_str() {
                "Development" => Color::Blue,
                "Cloud" => C_ACCENT,
                _ => C_DIM,
            };
            let name_style = if active {
                Style::default().fg(C_ACTIVE)
            } else {
                Style::default().fg(C_WHITE)
            };
            ListItem::new(Line::from(vec![
                Span::raw(format!("  {} ", dot)),
                Span::styled(
                    format!("[{}] ", &s.category[..3.min(s.category.len())]),
                    Style::default().fg(cat_col),
                ),
                Span::styled(s.name.clone(), name_style),
            ]))
        })
        .collect();

    if items.is_empty() {
        let msg = if app.segments_filter.is_empty() {
            "  No segments available.".to_string()
        } else {
            format!("  No segments matching '{}'", app.segments_filter)
        };
        items.push(
            ListItem::new(msg).style(Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC)),
        );
    }

    let title = if app.segments_filter.is_empty() {
        format!(" Segments ({}) ", segments.len())
    } else {
        format!(
            " Segments ({}) [Filter: {}] ",
            segments.len(),
            app.segments_filter
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_ACCENT))
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(C_WHITE)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("  ");

    f.render_stateful_widget(list, left[1], &mut app.plugins_list_state);

    // Right: detail + action
    let selected = app
        .plugins_list_state
        .selected()
        .and_then(|i| segments.get(i));

    if let Some(seg) = selected {
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(5)])
            .split(cols[1]);

        let active = app.is_segment_active(seg);
        let status_style = if active {
            Style::default().fg(C_ACTIVE)
        } else {
            Style::default().fg(C_DIM)
        };

        let detail = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {}", seg.name),
                Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status     ", Style::default().fg(C_DIM)),
                Span::styled(if active { "● ACTIVE" } else { "○ INACTIVE" }, status_style),
            ]),
            Line::from(vec![
                Span::styled("  Type       ", Style::default().fg(C_DIM)),
                Span::styled(seg.segment_type.clone(), Style::default().fg(C_ACCENT)),
            ]),
            Line::from(vec![
                Span::styled("  Category   ", Style::default().fg(C_DIM)),
                Span::raw(seg.category.clone()),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Description",
                Style::default().fg(C_DIM),
            )]),
            Line::from(format!("  {}", seg.description)),
            Line::from(""),
            Line::from(vec![Span::styled("  Notes", Style::default().fg(C_DIM))]),
            Line::from(format!("  {}", seg.documentation)),
        ];

        f.render_widget(
            Paragraph::new(detail)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(C_ACCENT))
                        .title(" Segment Detail "),
                )
                .wrap(Wrap { trim: true }),
            right[0],
        );

        // Action hint
        let (action_label, action_color, verb) = if active {
            ("Enter", C_ERROR, "REMOVE from theme")
        } else {
            ("Enter", C_LOCAL, "ADD to theme")
        };

        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::raw("\n  Press "),
                Span::styled(
                    action_label,
                    Style::default()
                        .fg(action_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" to {} · Ctrl+R to undo last change", verb),
                    Style::default().fg(C_DIM),
                ),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(C_DIM))
                    .title(" Action "),
            ),
            right[1],
        );
    } else {
        f.render_widget(
            Paragraph::new("\n  Select a segment to see details and toggle it in your theme.")
                .style(Style::default().fg(C_DIM))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(C_DIM))
                        .title(" Segment Detail "),
                ),
            cols[1],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  WELCOME SCREEN
// ═══════════════════════════════════════════════════════════════════════════════

fn render_welcome(f: &mut Frame, area: Rect, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Min(0),    // content
            Constraint::Length(1), // footer
        ])
        .split(area);

    // Title
    f.render_widget(
        Paragraph::new(format!(
            "  PoshBuddy v{} — Terminal Management Engine",
            app.version
        ))
        .style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
        root[0],
    );

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(root[1]);

    // ── Left: Quick Actions ─────────────────────────────────────────────────
    let action_defs: &[(&str, &str, usize)] = &[
        ("R", "Random Theme", 0),
        ("N", "Install Nerd Fonts", 1),
        ("I", "Toggle Terminal-Icons", 2),
        ("D", "Diagnostics (Soon)", 3),
        ("V", "View Backups Info", 4),
        ("B", "Create Manual Backup", 8),
        ("1", "Go to Themes", 5),
        ("2", "Go to Fonts", 6),
        ("3", "Go to Segments", 7),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (display_i, (key, label, action_idx)) in action_defs.iter().enumerate() {
        if display_i == 6 {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "  ─────────────────────── ",
                Style::default().fg(C_DIM),
            )])));
        }
        let is_selected = *action_idx == app.welcome_selected_action;
        let is_disabled = *action_idx == 3; // Diagnostics soon
        let key_style = if is_disabled && is_selected {
            Style::default().fg(C_DIM).bg(Color::DarkGray)
        } else if is_disabled {
            Style::default().fg(C_DIM)
        } else if is_selected {
            Style::default()
                .fg(C_BLACK)
                .bg(C_ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)
        };
        let label_style = if is_disabled && is_selected {
            Style::default().fg(C_DIM).bg(Color::DarkGray)
        } else if is_disabled {
            Style::default().fg(C_DIM)
        } else if is_selected {
            Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!(" {} ", key), key_style),
            Span::raw("  "),
            Span::styled(label.to_string(), label_style),
        ])));
    }

    f.render_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_ACCENT))
                .title(" Quick Actions "),
        ),
        cols[0],
    );

    // ── Right: System status + Resources ───────────────────────────────────
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(cols[1]);

    // System status panel
    let sys_lines = if let Some(s) = &app.system_specs {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Nerd Font    ", Style::default().fg(C_DIM)),
                if s.has_nerd_font {
                    Span::styled("● Detected", Style::default().fg(C_LOCAL))
                } else {
                    Span::styled(
                        "○ Not found  (icons may be broken)",
                        Style::default().fg(C_ERROR),
                    )
                },
            ]),
            Line::from(vec![
                Span::styled("  PowerShell   ", Style::default().fg(C_DIM)),
                if s.is_pwsh_7 {
                    Span::styled("● v7 (pwsh)", Style::default().fg(C_LOCAL))
                } else {
                    Span::styled(
                        "○ v5.1  (PowerShell 7 recommended)",
                        Style::default().fg(C_ACTIVE),
                    )
                },
            ]),
            Line::from(vec![
                Span::styled("  Terminal     ", Style::default().fg(C_DIM)),
                if s.is_windows_terminal {
                    Span::styled("● Windows Terminal", Style::default().fg(C_LOCAL))
                } else {
                    Span::styled(
                        "○ Classic Console  (upgrade recommended)",
                        Style::default().fg(C_ACTIVE),
                    )
                },
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Backups      ", Style::default().fg(C_DIM)),
                Span::styled(
                    format!("{} available", app.total_backups),
                    Style::default().fg(C_WHITE),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Profiles     ", Style::default().fg(C_DIM)),
                Span::styled(
                    format!("{} detected", app.detected_profiles.len()),
                    Style::default().fg(C_WHITE),
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Scanning system...",
                Style::default().fg(C_DIM),
            )]),
        ]
    };

    f.render_widget(
        Paragraph::new(sys_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_DIM))
                .title(" System Status "),
        ),
        right[0],
    );

    // Latest Changes panel (v0.3.3)
    let changelog_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  v0.3.3 ",
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Mass Font Installer", Style::default().fg(C_WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(C_ACCENT)),
            Span::styled(
                "Install all Nerd Fonts with one click.",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(C_ACCENT)),
            Span::styled(
                "Professional progress bar & safety checks.",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(C_ACCENT)),
            Span::styled(
                "New version dashboard panel.",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  v0.3.2 ",
                Style::default().fg(C_ACTIVE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Navigation & Localization", Style::default().fg(C_WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(C_ACTIVE)),
            Span::styled(
                "100% English & Global Nav (Esc/H).",
                Style::default().fg(Color::Gray),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(changelog_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_DIM))
                .title(" Latest Changes "),
        ),
        right[1],
    );

    // ── Overlays ───────────────────────────────────────────────────────────

    // 1. Confirm Mass Font Installation
    if app.state == AppState::ConfirmMassFontInstallation {
        let area = centered_rect(60, 25, f.size());
        f.render_widget(Clear, area);
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
        f.render_widget(Paragraph::new(text).block(block), area);
    }

    // 2. Installation Progress Gauge
    if let AppState::InstallingAllFonts {
        progress,
        current_font,
        index,
        total,
    } = &app.state
    {
        let area = centered_rect(70, 20, f.size());
        f.render_widget(Clear, area);
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
            .split(block.inner(area));

        f.render_widget(block, area);
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

    // Footer
    f.render_widget(
        Paragraph::new(
            "  ↑↓ Navigate  │  Enter Execute  │  1-5/B Action  │  T/F/S Go to View  │  Q Quit",
        )
        .style(Style::default().fg(C_DIM)),
        root[2],
    );
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
