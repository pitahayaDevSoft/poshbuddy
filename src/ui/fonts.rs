use crate::app::App;
use crate::ui::components::render_search_bar;
use crate::ui::{C_ACCENT, C_DIM, C_LOCAL, C_WHITE};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub(crate) fn render_fonts(f: &mut Frame, area: Rect, app: &mut App) {
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

    let font_iter = app
        .fonts
        .iter()
        .filter(|f| crate::app::contains_ignore_ascii_case(&f.name, &app.fonts_filter))
        .map(|font| {
            // ⚡ Bolt: Zero-allocation string concatenation via Spans instead of format!()
            let line = Line::from(vec![Span::raw("  "), Span::raw(font.name.as_str())]);
            ListItem::new(line).style(Style::default().fg(C_WHITE))
        });

    let is_empty = app.filtered_fonts_count() == 0;

    let empty_msg_iter = if is_empty {
        let msg = if app.fonts_filter.is_empty() {
            "  No fonts available.".to_string()
        } else {
            format!(
                "  No fonts matching '{}' (Press Esc to clear search)",
                app.fonts_filter
            )
        };
        Some(ListItem::new(msg).style(Style::default().fg(C_DIM)))
    } else {
        None
    }
    .into_iter();

    let title = if app.fonts_filter.is_empty() {
        " Available Fonts "
    } else {
        " Available Fonts (Filtered) "
    };

    let items_iter = font_iter.chain(empty_msg_iter);

    let mut list = List::new(items_iter).block(
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
    let selected = app
        .fonts_list_state
        .selected()
        .and_then(|i| app.filtered_font_at(i));
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
        let msg = if is_empty && !app.fonts_filter.is_empty() {
            "\n  No results. Press Esc to clear filter."
        } else {
            "\n  Select a font to continue..."
        };
        f.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(C_DIM))
                .block(detail_block),
            cols[1],
        );
    }
}
