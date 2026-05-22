use crate::app::{ActiveView, App};
use crate::ui::{C_ACCENT, C_BLACK, C_DIM, C_WHITE};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

// ── Title bar (1 line, no border) ─────────────────────────────────────────────
pub(crate) fn render_title_bar(f: &mut Frame, area: Rect, app: &App) {
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
        Paragraph::new(format!("  🐱 PoshBuddy v{}", app.version))
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
    let user = whoami::username().unwrap_or_else(|_| "unknown".to_string());
    f.render_widget(
        Paragraph::new(format!("{}  {}  ", user, time))
            .alignment(Alignment::Right)
            .style(Style::default().fg(C_DIM)),
        cols[2],
    );
}

// ── Tab bar (3 lines with border bottom) ──────────────────────────────────────
pub(crate) fn render_tab_bar(f: &mut Frame, area: Rect, app: &App) {
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
pub(crate) fn render_main_footer(f: &mut Frame, area: Rect, app: &App) {
    let is_filtering = match app.active_view {
        ActiveView::Themes => !app.filter.is_empty(),
        ActiveView::Fonts => !app.fonts_filter.is_empty(),
        ActiveView::Segments => !app.segments_filter.is_empty(),
    };

    let esc_action = if is_filtering {
        "Esc Clear Search"
    } else {
        "Esc/H Dashboard"
    };

    let hint = match app.active_view {
        ActiveView::Themes => format!(
            "  ↑↓ Navigate  │  Enter Apply  │  Type Search  │  {}  │  Tab Next Tab  │  Ctrl+R Restore  │  Q Quit",
            esc_action
        ),
        ActiveView::Fonts => format!(
            "  ↑↓ Navigate  │  Enter Install  │  Type Search  │  {}  │  Tab Next Tab  │  Ctrl+R Restore  │  Q Quit",
            esc_action
        ),
        ActiveView::Segments => format!(
            "  ↑↓ Navigate  │  Enter Toggle  │  Type Search  │  {}  │  Tab Next Tab  │  Ctrl+R Restore  │  Q Quit",
            esc_action
        ),
    };
    f.render_widget(Paragraph::new(hint).style(Style::default().fg(C_DIM)), area);
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
    let w = area.width.min(58);
    let h = 7u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let modal = Rect::new(x, y, w, h);

    let content = if let Some(d) = dismiss {
        format!("\n  {}\n\n  Press {} to dismiss.", msg, d)
    } else {
        format!("\n  {}", msg)
    };

    f.render_widget(Clear, modal);
    f.render_widget(
        Paragraph::new(content)
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

/// Renders a search bar with visible cursor when filter is active
pub(crate) fn render_search_bar(f: &mut Frame, area: Rect, filter: &str, context: &str) {
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
