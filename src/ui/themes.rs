use crate::app::App;
use crate::ui::components::{render_search_bar, status_dot};
use crate::ui::{
    C_ACCENT, C_DIM, C_ERROR, C_GRAD_1, C_GRAD_2, C_GRAD_3, C_LOCAL, C_REMOTE, C_WHITE,
};
use ansi_to_tui::IntoText as _;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
};

pub(crate) fn render_themes(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Header with decorative gradient underline feel
    let header = Line::from(vec![
        Span::styled(
            "  󰔰 ",
            Style::default().fg(C_GRAD_1).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "T",
            Style::default().fg(C_GRAD_1).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "H",
            Style::default().fg(C_GRAD_2).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "E",
            Style::default()
                .fg(Color::Rgb(150, 100, 245))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "M",
            Style::default().fg(C_GRAD_3).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "E",
            Style::default()
                .fg(Color::Rgb(235, 60, 140))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "S",
            Style::default()
                .fg(Color::Rgb(255, 80, 80))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  EXPLORER",
            Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(
        Paragraph::new(header).alignment(Alignment::Center),
        chunks[0],
    );

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Left column: search + list
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(cols[0]);

    render_search_bar(f, left[0], &app.filter, "Themes");

    let is_empty = app.filtered_themes_count() == 0;
    render_themes_list(f, left[1], app, is_empty);

    // Right column: preview
    render_themes_preview(f, cols[1], app, is_empty);
}

fn render_themes_list(f: &mut Frame, area: Rect, app: &mut App, is_empty: bool) {
    let filter = &app.filter;

    let local_iter = app
        .themes
        .iter()
        .filter(|t| crate::app::contains_ignore_ascii_case(&t.name, filter))
        .map(|t| {
            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled("● ", Style::default().fg(C_LOCAL)),
                Span::styled(t.name.as_str(), Style::default().fg(C_WHITE)),
                Span::styled(
                    " local",
                    Style::default().fg(C_LOCAL).add_modifier(Modifier::DIM),
                ),
            ]);
            ListItem::new(line)
        });

    let remote_iter = app
        .remote_themes
        .iter()
        .filter(|rt| {
            crate::app::contains_ignore_ascii_case(&rt.name, filter)
                && app
                    .themes
                    .binary_search_by(|t| t.name.cmp(&rt.name))
                    .is_err()
        })
        .map(|rt| {
            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled("◈ ", Style::default().fg(C_REMOTE)),
                Span::styled(
                    rt.name.as_str(),
                    Style::default().fg(Color::Rgb(200, 185, 250)),
                ),
                Span::styled(
                    " remote",
                    Style::default().fg(C_REMOTE).add_modifier(Modifier::DIM),
                ),
            ]);
            ListItem::new(line)
        });

    let empty_msg_iter = if is_empty {
        let msg = if app.filter.is_empty() {
            "  No themes available.".to_string()
        } else {
            format!(
                "  No themes matching '{}'\n  Press Esc to clear",
                app.filter
            )
        };
        Some(ListItem::new(msg).style(Style::default().fg(C_DIM)))
    } else {
        None
    }
    .into_iter();

    let count = app.filtered_themes_count();
    let title = if app.filter.is_empty() {
        format!(" 󰔰 Themes  {} ", count)
    } else {
        format!(" 󰍉 Filtered  {} ", count)
    };

    let items_iter = local_iter.chain(remote_iter).chain(empty_msg_iter);

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

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_themes_preview(f: &mut Frame, area: Rect, app: &App, is_empty: bool) {
    // Split into preview + info strip
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    if app.theme_preview.is_empty() || app.theme_preview == " Loading preview..." {
        let preview_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(55, 70, 90)))
            .title(Span::styled(
                " 󰸉 ANSI Preview ",
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            ));

        let msg = if app.theme_preview == " Loading preview..." {
            "\n  Loading preview..."
        } else if is_empty && !app.filter.is_empty() {
            "\n  No results. Press Esc to clear filter."
        } else {
            "\n  Select a theme to see a live preview..."
        };
        f.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(C_DIM))
                .block(preview_block),
            split[0],
        );
    } else {
        // Split horizontally into left and right preview panels
        let preview_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(split[0]);

        let (left_preview, right_preview) = split_preview(&app.theme_preview);

        let left_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(55, 70, 90)))
            .title(Span::styled(
                " 󰸉 Left Prompt ",
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            ));

        let right_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(55, 70, 90)))
            .title(Span::styled(
                " 󰸉 Right Prompt ",
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            ));

        let left_text = left_preview.as_bytes().into_text().unwrap_or_default();
        f.render_widget(
            Paragraph::new(left_text)
                .block(left_block)
                .wrap(Wrap { trim: false }),
            preview_columns[0],
        );

        let right_text = right_preview.as_bytes().into_text().unwrap_or_default();
        f.render_widget(
            Paragraph::new(right_text)
                .block(right_block)
                .alignment(Alignment::Right)
                .wrap(Wrap { trim: false }),
            preview_columns[1],
        );
    }

    // Info strip below preview
    let legend = Line::from(vec![
        Span::styled("  ● ", Style::default().fg(C_LOCAL)),
        Span::styled("Local  ", Style::default().fg(C_DIM)),
        Span::styled("◈ ", Style::default().fg(C_REMOTE)),
        Span::styled("Remote  ", Style::default().fg(C_DIM)),
        Span::styled("▶ ", Style::default().fg(C_ACCENT)),
        Span::styled("Selected  ", Style::default().fg(C_DIM)),
        Span::styled("Enter ", Style::default().fg(C_ACCENT)),
        Span::styled("Apply theme", Style::default().fg(C_DIM)),
    ]);
    f.render_widget(
        Paragraph::new(legend).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(35, 45, 60))),
        ),
        split[1],
    );

    // suppress unused
    let _ = (C_ERROR, C_GRAD_1, C_GRAD_2, status_dot);
}

fn split_line_by_gap(line: &str) -> (String, String) {
    let mut longest_space_start = 0;
    let mut longest_space_len = 0;

    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' {
            let start = i;
            while i < bytes.len() && bytes[i] == b' ' {
                i += 1;
            }
            let len = i - start;
            if start > 0 && i < bytes.len() && len > longest_space_len {
                longest_space_len = len;
                longest_space_start = start;
            }
        } else {
            i += 1;
        }
    }

    if longest_space_len >= 4 {
        let left = line[..longest_space_start].to_string();
        let right = line[longest_space_start + longest_space_len..].to_string();
        (left, right)
    } else {
        (line.to_string(), String::new())
    }
}

fn split_preview(preview: &str) -> (String, String) {
    let mut left_lines = Vec::new();
    let mut right_lines = Vec::new();

    for line in preview.lines() {
        let (left, right) = split_line_by_gap(line);
        left_lines.push(left);
        right_lines.push(right);
    }

    (left_lines.join("\n"), right_lines.join("\n"))
}
