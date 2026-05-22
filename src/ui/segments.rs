use crate::app::App;
use crate::ui::components::{layout_header_content, layout_two_columns, render_search_bar};
use crate::ui::{C_ACCENT, C_ACTIVE, C_DIM, C_WHITE};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub(crate) fn render_segments(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = layout_header_content(area);

    // Header
    f.render_widget(
        Paragraph::new("\n[ SEGMENT MANAGER ]")
            .alignment(Alignment::Center)
            .style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
        chunks[0],
    );

    let cols = layout_two_columns(chunks[1], 45, 55);

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
            let dot = if active { "●" } else { "○" };
            let style = if active {
                Style::default().fg(C_ACTIVE)
            } else {
                Style::default().fg(C_WHITE)
            };
            // ⚡ Bolt: Zero-allocation string concatenation via Spans instead of format!()
            let line = Line::from(vec![
                Span::raw("  "),
                Span::raw(dot),
                Span::raw(" "),
                Span::raw(s.name.as_str()),
            ]);
            ListItem::new(line).style(style)
        });

    let is_empty = app.filtered_segments_count() == 0;

    let empty_msg_iter = if is_empty {
        let msg = if app.segments_filter.is_empty() {
            "  No components available.".to_string()
        } else {
            format!(
                "  No components matching '{}' (Press Esc to clear search)",
                app.segments_filter
            )
        };
        Some(ListItem::new(msg).style(Style::default().fg(C_DIM)))
    } else {
        None
    }
    .into_iter();

    let title = if app.segments_filter.is_empty() {
        " Components "
    } else {
        " Components (Filtered) "
    };

    let items_iter = segments_iter.chain(empty_msg_iter);

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

    f.render_stateful_widget(list, area, &mut app.segments_list_state);
}

fn render_segment_detail(f: &mut Frame, area: Rect, app: &mut App) {
    let selected = app
        .segments_list_state
        .selected()
        .and_then(|i| app.filtered_segment_at(i));
    let is_empty = app.filtered_segments_count() == 0;

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_DIM))
        .title(" Information ");

    if let Some(ref seg) = selected {
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
            area,
        );
    } else {
        let msg = if is_empty && !app.segments_filter.is_empty() {
            "\n  No results. Press Esc to clear filter."
        } else {
            "\n  Select a component to toggle..."
        };
        f.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(C_DIM))
                .block(detail_block),
            area,
        );
    }
}
