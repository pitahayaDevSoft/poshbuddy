use crate::app::{ActiveView, App};
use crate::ui::{C_ACCENT, C_ACTIVE, C_BLACK, C_DIM, C_ERROR, C_GRAD_1, C_GRAD_2, C_GRAD_3, C_LOCAL, C_WHITE};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

// ── Title bar (1 line, no border) ─────────────────────────────────────────────
pub(crate) fn render_title_bar(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Left: brand with gradient-style text
    let brand_line = Line::from(vec![
        Span::styled("  🐱 ", Style::default().fg(C_GRAD_2)),
        Span::styled("Posh", Style::default().fg(C_GRAD_1).add_modifier(Modifier::BOLD)),
        Span::styled("Buddy", Style::default().fg(C_GRAD_2).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" v{}", app.version), Style::default().fg(C_DIM)),
    ]);
    f.render_widget(Paragraph::new(brand_line), cols[0]);

    // Centre: active theme with icon
    let theme = app
        .active_config_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("no theme");
    let center_line = Line::from(vec![
        Span::styled(" 🎨 ", Style::default().fg(C_ACTIVE)),
        Span::styled(theme, Style::default().fg(C_WHITE).add_modifier(Modifier::ITALIC)),
    ]);
    f.render_widget(
        Paragraph::new(center_line).alignment(Alignment::Center),
        cols[1],
    );

    // Right: user + clock with subtle separator
    let time = chrono::Local::now().format("%H:%M").to_string();
    let user = whoami::username().unwrap_or_else(|_| "unknown".to_string());
    let right_line = Line::from(vec![
        Span::styled(" 󰀄 ", Style::default().fg(C_DIM)),
        Span::styled(&user, Style::default().fg(C_WHITE)),
        Span::styled("  󱑎 ", Style::default().fg(C_DIM)),
        Span::styled(time, Style::default().fg(C_ACCENT)),
        Span::raw("  "),
    ]);
    f.render_widget(
        Paragraph::new(right_line).alignment(Alignment::Right),
        cols[2],
    );
}

// ── Tab bar (3 lines with border bottom) ──────────────────────────────────────
pub(crate) fn render_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    let tabs: &[(&str, &str, ActiveView)] = &[
        (" 󰔰 ", "Themes", ActiveView::Themes),
        (" 󰛖 ", "Fonts", ActiveView::Fonts),
        (" 󰓣 ", "Segments", ActiveView::Segments),
    ];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    for (i, (icon, label, view)) in tabs.iter().enumerate() {
        let is_active = app.active_view == *view;
        let count = match view {
            ActiveView::Themes => app.filtered_themes_count(),
            ActiveView::Fonts => app.filtered_fonts_count(),
            ActiveView::Segments => app.filtered_segments_count(),
        };

        let key = i + 1;

        let text = if is_active {
            Line::from(vec![
                Span::styled(*icon, Style::default().fg(C_BLACK).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{} ", label),
                    Style::default().fg(C_BLACK).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("({}) ", count),
                    Style::default().fg(C_BLACK),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled(*icon, Style::default().fg(C_DIM)),
                Span::styled(format!("{} ", label), Style::default().fg(C_DIM)),
                Span::styled(format!("[{}] ", key), Style::default().fg(C_DIM)),
            ])
        };

        let (fg, bg, border_color, border_type) = if is_active {
            (C_BLACK, C_ACCENT, C_ACCENT, BorderType::Thick)
        } else {
            (C_WHITE, Color::Reset, Color::Rgb(40, 50, 65), BorderType::Rounded)
        };

        f.render_widget(
            Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(fg).bg(bg))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(border_type)
                        .border_style(Style::default().fg(border_color)),
                ),
            chunks[i],
        );
    }
}

// ── Context-sensitive footer (1 line, no border) ──────────────────────────────
pub(crate) fn render_main_footer(f: &mut Frame, area: Rect, app: &App) {
    let is_filtering = match app.active_view {
        ActiveView::Themes => !app.filter.is_empty(),
        ActiveView::Fonts => !app.fonts_filter.is_empty(),
        ActiveView::Segments => !app.segments_filter.is_empty(),
    };

    // Build a rich, icon-based footer
    let action_label = match app.active_view {
        ActiveView::Themes => "Apply",
        ActiveView::Fonts => "Install",
        ActiveView::Segments => "Toggle",
    };

    let esc_text = if is_filtering {
        "Clear Search"
    } else {
        "Dashboard"
    };

    let spans = vec![
        Span::styled("  ↑↓ ", Style::default().fg(C_ACCENT)),
        Span::styled("Navigate", Style::default().fg(C_DIM)),
        Span::styled("  ·  Enter ", Style::default().fg(C_ACCENT)),
        Span::styled(action_label, Style::default().fg(C_DIM)),
        Span::styled("  ·  Type ", Style::default().fg(C_ACCENT)),
        Span::styled("Search", Style::default().fg(C_DIM)),
        Span::styled("  ·  Esc ", Style::default().fg(C_ACTIVE)),
        Span::styled(esc_text, Style::default().fg(C_DIM)),
        Span::styled("  ·  Tab ", Style::default().fg(C_ACCENT)),
        Span::styled("Switch Tab", Style::default().fg(C_DIM)),
        Span::styled("  ·  Ctrl+R ", Style::default().fg(C_GRAD_3)),
        Span::styled("Restore", Style::default().fg(C_DIM)),
        Span::styled("  ·  Q ", Style::default().fg(C_ERROR)),
        Span::styled("Quit", Style::default().fg(C_DIM)),
        Span::raw("  "),
    ];

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Floating modal ────────────────────────────────────────────────────────────
pub(crate) fn render_modal(
    f: &mut Frame,
    area: Rect,
    title: &str,
    msg: &str,
    color: Color,
    dismiss: Option<&str>,
) {
    let w = area.width.min(60);
    let h = 8u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let modal = Rect::new(x, y, w, h);

    let content = if let Some(d) = dismiss {
        format!("\n{}\n\n  Press {} to dismiss.", msg, d)
    } else {
        format!("\n{}", msg)
    };

    f.render_widget(Clear, modal);
    f.render_widget(
        Paragraph::new(content)
            .style(Style::default().fg(color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(color))
                    .title(Span::styled(title, Style::default().fg(color).add_modifier(Modifier::BOLD))),
            )
            .wrap(Wrap { trim: true }),
        modal,
    );
}

pub(crate) fn layout_header_content(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area)
}

pub(crate) fn layout_two_columns(area: Rect, left_pct: u16, right_pct: u16) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_pct),
            Constraint::Percentage(right_pct),
        ])
        .split(area)
}

/// Renders a styled search bar with visible cursor when filter is active
pub(crate) fn render_search_bar(f: &mut Frame, area: Rect, filter: &str, context: &str) {
    let (text, style) = if filter.is_empty() {
        (
            Line::from(vec![
                Span::styled("  󰍉 ", Style::default().fg(C_DIM)),
                Span::styled(format!("Search {}...", context.to_lowercase()), Style::default().fg(C_DIM)),
            ]),
            Style::default(),
        )
    } else {
        (
            Line::from(vec![
                Span::styled("  󰍉 ", Style::default().fg(C_ACCENT)),
                Span::styled(filter, Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(C_ACCENT)),
            ]),
            Style::default(),
        )
    };

    let title = if filter.is_empty() {
        Span::styled(" 󰍉 Search ", Style::default().fg(C_DIM))
    } else {
        Span::styled(" 󰍉 Search  Esc to clear ", Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))
    };

    let border_style = if filter.is_empty() {
        Style::default().fg(Color::Rgb(40, 50, 65))
    } else {
        Style::default().fg(C_ACCENT)
    };

    f.render_widget(
        Paragraph::new(text).style(style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(title),
        ),
        area,
    );
}

/// Centers a rect of given percentage within parent
pub(crate) fn centered_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
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

/// Render a section header with a decorative horizontal line
pub(crate) fn render_section_header(f: &mut Frame, area: Rect, title: &str, color: Color) {
    let line = Line::from(vec![
        Span::styled("  ▌ ", Style::default().fg(color)),
        Span::styled(title, Style::default().fg(color).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

/// Render a badge/pill span  
pub(crate) fn badge(text: &str, fg: Color, bg: Color) -> Span<'static> {
    Span::styled(
        format!(" {} ", text),
        Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
    )
}

/// Render a status dot
pub(crate) fn status_dot(active: bool) -> Span<'static> {
    if active {
        Span::styled("● ", Style::default().fg(C_LOCAL))
    } else {
        Span::styled("○ ", Style::default().fg(C_DIM))
    }
}
