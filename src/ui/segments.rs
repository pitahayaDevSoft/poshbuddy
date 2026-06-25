use crate::app::App;
use crate::ui::components::{
    layout_header_content, layout_two_columns, render_search_bar, status_dot,
};
use crate::ui::{C_ACCENT, C_ACTIVE, C_BLACK, C_DIM, C_ERROR, C_GRAD_2, C_LOCAL, C_WHITE};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
};

pub(crate) fn render_segments(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = layout_header_content(area);

    // Header
    let header = Line::from(vec![
        Span::styled(
            "  󰓣 ",
            Style::default().fg(C_GRAD_2).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "SEGMENT MANAGER",
            Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  —  Customize your prompt", Style::default().fg(C_DIM)),
    ]);
    f.render_widget(
        Paragraph::new(header).alignment(Alignment::Center),
        chunks[0],
    );

    let cols = layout_two_columns(chunks[1], 42, 58);

    // Left: search + list
    let left = layout_header_content(cols[0]);
    render_search_bar(f, left[0], &app.segments_filter, "Segments");
    render_segment_list(f, left[1], app);

    // Right: detail
    render_segment_detail(f, cols[1], app);
}

fn render_segment_list(f: &mut Frame, area: Rect, app: &mut App) {
    let segments_iter = app
        .segments
        .iter()
        .filter(|p| {
            crate::app::contains_ignore_ascii_case(&p.name, &app.segments_filter)
                || crate::app::contains_ignore_ascii_case(&p.description, &app.segments_filter)
                || crate::app::contains_ignore_ascii_case(&p.category, &app.segments_filter)
        })
        .map(|s| {
            let active = app.is_segment_active(s);
            let dot = status_dot(active);
            let name_style = if active {
                Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(170, 180, 195))
            };
            let line = Line::from(vec![
                Span::raw("  "),
                dot,
                Span::styled(s.name.as_str(), name_style),
            ]);
            ListItem::new(line)
        });

    let is_empty = app.filtered_segments_count() == 0;

    let empty_msg_iter = if is_empty {
        let msg = if app.segments_filter.is_empty() {
            "  No components available.".to_string()
        } else {
            format!(
                "  No components matching '{}'\n  Esc to clear",
                app.segments_filter
            )
        };
        Some(ListItem::new(msg).style(Style::default().fg(C_DIM)))
    } else {
        None
    }
    .into_iter();

    let count = app.filtered_segments_count();
    let active_count = app
        .segments
        .iter()
        .filter(|s| app.is_segment_active(s))
        .count();

    let title = if app.segments_filter.is_empty() {
        format!(" 󰓣 Components  {} ", count)
    } else {
        format!(" 󰍉 Filtered  {} ", count)
    };

    let items_iter = segments_iter.chain(empty_msg_iter);

    let mut list = List::new(items_iter).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(55, 70, 90)))
            .title(Span::styled(
                title,
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            )),
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

    f.render_stateful_widget(list, area, &mut app.segments_list_state);

    // Active count strip at very bottom — show overall stats
    let _ = active_count; // used for context
}

fn render_segment_detail(f: &mut Frame, area: Rect, app: &mut App) {
    // Split detail into top content + bottom stats strip
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let selected = app
        .segments_list_state
        .selected()
        .and_then(|i| app.filtered_segment_at(i));
    let is_empty = app.filtered_segments_count() == 0;

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(55, 70, 90)))
        .title(Span::styled(
            " 󰋼 Component Info ",
            Style::default().fg(C_GRAD_2).add_modifier(Modifier::BOLD),
        ));

    if let Some(ref seg) = selected {
        let active = app.is_segment_active(seg);

        // Category color mapping
        let cat_color = match seg.category.as_str() {
            "Development" => Color::Rgb(104, 211, 145),
            "System" => Color::Rgb(99, 179, 237),
            "Cloud" => Color::Rgb(154, 117, 234),
            "Version Control" => Color::Rgb(252, 196, 25),
            "Time" => Color::Rgb(247, 137, 215),
            _ => C_DIM,
        };

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  󰓣 ",
                    Style::default().fg(C_GRAD_2).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    seg.name.as_str(),
                    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  ──────────────────────────────────────",
                Style::default().fg(Color::Rgb(40, 55, 75)),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status     ", Style::default().fg(C_DIM)),
                if active {
                    Span::styled(
                        " ENABLED ",
                        Style::default()
                            .fg(C_BLACK)
                            .bg(C_LOCAL)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        " DISABLED ",
                        Style::default()
                            .fg(C_WHITE)
                            .bg(Color::Rgb(40, 50, 65))
                            .add_modifier(Modifier::BOLD),
                    )
                },
            ]),
            Line::from(vec![
                Span::styled("  Category   ", Style::default().fg(C_DIM)),
                Span::styled(
                    seg.category.as_str(),
                    Style::default().fg(cat_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Type       ", Style::default().fg(C_DIM)),
                Span::styled(seg.segment_type.as_str(), Style::default().fg(C_WHITE)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  ──────────────────────────────────────",
                Style::default().fg(Color::Rgb(40, 55, 75)),
            )),
            Line::from(""),
            Line::from(Span::styled("  Description", Style::default().fg(C_DIM))),
            Line::from(Span::styled(
                format!("  {}", seg.description),
                Style::default().fg(Color::Rgb(200, 210, 225)),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  ──────────────────────────────────────",
                Style::default().fg(Color::Rgb(40, 55, 75)),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Action     ", Style::default().fg(C_DIM)),
                Span::styled(
                    " Enter ",
                    Style::default()
                        .fg(C_BLACK)
                        .bg(C_ACTIVE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if active {
                        " Disable segment"
                    } else {
                        " Enable segment"
                    },
                    Style::default().fg(C_ACTIVE),
                ),
            ]),
        ];
        f.render_widget(
            Paragraph::new(lines)
                .block(detail_block)
                .wrap(Wrap { trim: true }),
            split[0],
        );
    } else {
        let (msg, style) = if is_empty && !app.segments_filter.is_empty() {
            (
                "\n  No results. Press Esc to clear.",
                Style::default().fg(C_DIM),
            )
        } else {
            (
                "\n\n  󰓣  Select a component\n  to view details and toggle it.",
                Style::default().fg(C_DIM),
            )
        };
        f.render_widget(
            Paragraph::new(msg).style(style).block(detail_block),
            split[0],
        );
    }

    // Bottom stats strip
    let total = app.segments.len();
    let active_total = app
        .segments
        .iter()
        .filter(|s| app.is_segment_active(s))
        .count();
    let inactive_total = total.saturating_sub(active_total);

    let stats = Line::from(vec![
        Span::styled("  ● ", Style::default().fg(C_LOCAL)),
        Span::styled(
            format!("{} enabled  ", active_total),
            Style::default().fg(C_DIM),
        ),
        Span::styled("○ ", Style::default().fg(C_DIM)),
        Span::styled(
            format!("{} disabled  ", inactive_total),
            Style::default().fg(C_DIM),
        ),
        Span::styled("Total: ", Style::default().fg(C_DIM)),
        Span::styled(
            format!("{}", total),
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        ),
    ]);

    f.render_widget(
        Paragraph::new(stats).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(35, 45, 60))),
        ),
        split[1],
    );

    // suppress unused
    let _ = (C_ERROR,);
}
