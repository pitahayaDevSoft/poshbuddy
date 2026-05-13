with open('src/ui.rs', 'r') as file:
    content = file.read()

old_func = """fn render_themes(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // 1. Clean Header
    f.render_widget(
        Paragraph::new("\\n[ THEMES EXPLORER ]")
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

    let filter = &app.filter;

    let local_iter = app.themes
        .iter()
        .filter(|t| crate::app::contains_ignore_ascii_case(&t.name, filter))
        .map(|t| {
            let style = Style::default().fg(C_LOCAL);
            let line = Line::from(vec![
                Span::raw("  "),
                Span::raw(t.name.as_str()),
                Span::raw(" [Local]"),
            ]);
            ListItem::new(line).style(style)
        });

    let remote_iter = app.remote_themes
        .iter()
        .filter(|rt| crate::app::contains_ignore_ascii_case(&rt.name, filter)
            && app.themes.binary_search_by(|t| t.name.cmp(&rt.name)).is_err())
        .map(|rt| {
            let style = Style::default().fg(C_REMOTE);
            let line = Line::from(vec![
                Span::raw("  "),
                Span::raw(rt.name.as_str()),
                Span::raw(" [Remote]"),
            ]);
            ListItem::new(line).style(style)
        });

    let is_empty = app.filtered_themes_count() == 0;

    let empty_msg_iter = if is_empty {
        let msg = if app.filter.is_empty() {
            "  No themes available.".to_string()
        } else {
            format!("  No themes matching '{}' (Press Esc to clear search)", app.filter)
        };
        Some(ListItem::new(msg).style(Style::default().fg(C_DIM)))
    } else {
        None
    }.into_iter();

    let title = if app.filter.is_empty() {
        " Themes List "
    } else {
        " Themes List (Filtered) "
    };

    let items_iter = local_iter.chain(remote_iter).chain(empty_msg_iter);

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

    f.render_stateful_widget(list, left[1], &mut app.list_state);

    // 3. Right column: preview
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_ACCENT))
        .title(" ANSI Preview ");

    if app.theme_preview.is_empty() {
        let msg = if is_empty && !app.filter.is_empty() {
            "\\n  No results. Press Esc to clear filter."
        } else {
            "\\n  Select a theme to see preview..."
        };
        f.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(C_DIM))
                .block(preview_block),
            cols[1],
        );
    } else {
        let preview_text = app.theme_preview.as_bytes().into_text().unwrap_or_default();
        f.render_widget(
            Paragraph::new(preview_text)
                .block(preview_block)
                .wrap(Wrap { trim: false }),
            cols[1],
        );
    }
}"""

new_func = """fn render_themes(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // 1. Clean Header
    f.render_widget(
        Paragraph::new("\\n[ THEMES EXPLORER ]")
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

    let is_empty = app.filtered_themes_count() == 0;
    render_themes_list(f, left[1], app, is_empty);

    // 3. Right column: preview
    render_themes_preview(f, cols[1], app, is_empty);
}

fn render_themes_list(f: &mut Frame, area: Rect, app: &mut App, is_empty: bool) {
    let filter = &app.filter;

    let local_iter = app.themes
        .iter()
        .filter(|t| crate::app::contains_ignore_ascii_case(&t.name, filter))
        .map(|t| {
            let style = Style::default().fg(C_LOCAL);
            let line = Line::from(vec![
                Span::raw("  "),
                Span::raw(t.name.as_str()),
                Span::raw(" [Local]"),
            ]);
            ListItem::new(line).style(style)
        });

    let remote_iter = app.remote_themes
        .iter()
        .filter(|rt| crate::app::contains_ignore_ascii_case(&rt.name, filter)
            && app.themes.binary_search_by(|t| t.name.cmp(&rt.name)).is_err())
        .map(|rt| {
            let style = Style::default().fg(C_REMOTE);
            let line = Line::from(vec![
                Span::raw("  "),
                Span::raw(rt.name.as_str()),
                Span::raw(" [Remote]"),
            ]);
            ListItem::new(line).style(style)
        });

    let empty_msg_iter = if is_empty {
        let msg = if app.filter.is_empty() {
            "  No themes available.".to_string()
        } else {
            format!("  No themes matching '{}' (Press Esc to clear search)", app.filter)
        };
        Some(ListItem::new(msg).style(Style::default().fg(C_DIM)))
    } else {
        None
    }.into_iter();

    let title = if app.filter.is_empty() {
        " Themes List "
    } else {
        " Themes List (Filtered) "
    };

    let items_iter = local_iter.chain(remote_iter).chain(empty_msg_iter);

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

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_themes_preview(f: &mut Frame, area: Rect, app: &App, is_empty: bool) {
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_ACCENT))
        .title(" ANSI Preview ");

    if app.theme_preview.is_empty() {
        let msg = if is_empty && !app.filter.is_empty() {
            "\\n  No results. Press Esc to clear filter."
        } else {
            "\\n  Select a theme to see preview..."
        };
        f.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(C_DIM))
                .block(preview_block),
            area,
        );
    } else {
        let preview_text = app.theme_preview.as_bytes().into_text().unwrap_or_default();
        f.render_widget(
            Paragraph::new(preview_text)
                .block(preview_block)
                .wrap(Wrap { trim: false }),
            area,
        );
    }
}"""

if old_func in content:
    content = content.replace(old_func, new_func)
    with open('src/ui.rs', 'w') as file:
        file.write(content)
    print("Success")
else:
    print("Could not find exact block")
