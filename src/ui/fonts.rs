use crate::app::App;
use crate::ui::components::render_search_bar;
use crate::ui::{C_ACCENT, C_ACTIVE, C_BLACK, C_DIM, C_GRAD_2, C_GRAD_3, C_LOCAL, C_WHITE};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

pub(crate) fn render_fonts(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Header with gradient effect
    let header = Line::from(vec![
        Span::styled("  󰛖 ", Style::default().fg(C_GRAD_2).add_modifier(Modifier::BOLD)),
        Span::styled("FONT MANAGER", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
        Span::styled("  —  Nerd Fonts", Style::default().fg(C_DIM)),
    ]);
    f.render_widget(
        Paragraph::new(header).alignment(Alignment::Center),
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
            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled("󰛖 ", Style::default().fg(C_DIM)),
                Span::styled(font.name.as_str(), Style::default().fg(C_WHITE)),
            ]);
            ListItem::new(line)
        });

    let is_empty = app.filtered_fonts_count() == 0;

    let empty_msg_iter = if is_empty {
        let msg = if app.fonts_filter.is_empty() {
            "  No fonts available.".to_string()
        } else {
            format!(
                "  No fonts matching '{}'\n  Esc to clear",
                app.fonts_filter
            )
        };
        Some(ListItem::new(msg).style(Style::default().fg(C_DIM)))
    } else {
        None
    }
    .into_iter();

    let count = app.filtered_fonts_count();
    let title = if app.fonts_filter.is_empty() {
        format!(" 󰛖 Available Fonts  {} ", count)
    } else {
        format!(" 󰍉 Filtered  {} ", count)
    };

    let items_iter = font_iter.chain(empty_msg_iter);

    let mut list = List::new(items_iter).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(55, 70, 90)))
            .title(Span::styled(title, Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))),
    );

    if !is_empty {
        list = list
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(25, 40, 65))
                    .fg(C_WHITE)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▶ ");
    }

    f.render_stateful_widget(list, left[1], &mut app.fonts_list_state);

    // Right: rich detail panel
    let selected = app
        .fonts_list_state
        .selected()
        .and_then(|i| app.filtered_font_at(i));

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(55, 70, 90)))
        .title(Span::styled(
            " 󰋼 Font Details ",
            Style::default().fg(C_GRAD_2).add_modifier(Modifier::BOLD),
        ));

    if let Some(font) = selected {
        // Extract a short family name from the full font name
        let family = font
            .name
            .split_whitespace()
            .next()
            .unwrap_or(&font.name);

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  󰛖 ", Style::default().fg(C_GRAD_2).add_modifier(Modifier::BOLD)),
                Span::styled(
                    font.name.as_str(),
                    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            // Divider
            Line::from(Span::styled(
                "  ──────────────────────────────────────",
                Style::default().fg(Color::Rgb(40, 55, 75)),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Type       ", Style::default().fg(C_DIM)),
                Span::styled("Nerd Font (patched)", Style::default().fg(C_WHITE)),
            ]),
            Line::from(vec![
                Span::styled("  Family     ", Style::default().fg(C_DIM)),
                Span::styled(family, Style::default().fg(C_WHITE)),
            ]),
            Line::from(vec![
                Span::styled("  Glyphs     ", Style::default().fg(C_DIM)),
                Span::styled("3,600+ icons included", Style::default().fg(C_WHITE)),
            ]),
            Line::from(""),
            // Glyph preview row
            Line::from(vec![
                Span::styled("  Preview    ", Style::default().fg(C_DIM)),
                Span::styled(
                    "    󰊤    ",
                    Style::default().fg(C_GRAD_2),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  ──────────────────────────────────────",
                Style::default().fg(Color::Rgb(40, 55, 75)),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Action     ", Style::default().fg(C_DIM)),
                Span::styled(" Enter ", Style::default().fg(C_BLACK).bg(C_LOCAL).add_modifier(Modifier::BOLD)),
                Span::styled(" Install font", Style::default().fg(C_LOCAL)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  ℹ  After installing, update your terminal\n     font settings to activate.",
                Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC),
            )),
        ];
        f.render_widget(Paragraph::new(lines).block(detail_block), cols[1]);
    } else {
        let (msg, style) = if is_empty && !app.fonts_filter.is_empty() {
            ("\n  No results. Press Esc to clear.", Style::default().fg(C_DIM))
        } else {
            ("\n\n  󰛖  Select a font from the list\n  to see details and install it.", Style::default().fg(C_DIM))
        };
        f.render_widget(
            Paragraph::new(msg)
                .style(style)
                .block(detail_block),
            cols[1],
        );
    }

    // suppress unused
    let _ = (C_ACTIVE, C_GRAD_3);
}
