use crate::app::App;
use crate::ui::components::centered_rect;
use crate::ui::{C_ACCENT, C_ACTIVE, C_BLACK, C_DIM, C_ERROR, C_LOCAL, C_WHITE, SPINNER};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub(crate) fn render_welcome(f: &mut Frame, area: Rect, app: &App) {
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
    let has_space_for_logo = area.height > 25;

    let constraints = if has_space_for_logo {
        vec![
            Constraint::Length(14), // Logo
            Constraint::Length(3),  // Dashboard Title
            Constraint::Fill(1),    // Stats & Actions
            Constraint::Length(3),  // Next Step Hint
            Constraint::Length(1),  // Footer
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
        render_welcome_logo(f, chunks[next_chunk_idx]);
        next_chunk_idx += 1;
    }

    // Dashboard Header
    render_welcome_header(f, chunks[next_chunk_idx]);
    next_chunk_idx += 1;

    // 3. Stats & Actions (Dynamic Side-by-Side or Stacked)
    let is_narrow = area.width < 90;
    render_welcome_body(f, chunks[next_chunk_idx], app, is_narrow);
    next_chunk_idx += 1;

    // 4. Next Step Hint
    render_welcome_hint(f, chunks[next_chunk_idx], is_narrow);
    next_chunk_idx += 1;

    // 5. Footer
    render_welcome_footer(f, chunks[next_chunk_idx], app);
}

fn render_welcome_header(f: &mut Frame, area: Rect) {
    f.render_widget(
        Paragraph::new("[ DASHBOARD ]")
            .alignment(Alignment::Center)
            .style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
        area,
    );
}

fn render_welcome_body(f: &mut Frame, area: Rect, app: &App, is_narrow: bool) {
    let body_chunks = if is_narrow {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)])
            .split(area)
    } else {
        // Center the content on wide screens to avoid massive boxes
        let centered_body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(100),
                Constraint::Min(0),
            ])
            .split(area)[1];

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

    render_session_identity(f, left_column[0], app);
    render_environment_info(f, left_column[1], app);

    render_quick_steps(f, right_area, app);
}

fn render_welcome_hint(f: &mut Frame, area: Rect, is_narrow: bool) {
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
        area,
    );
}

fn render_welcome_footer(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(
        Paragraph::new(format!(
            "🐱 PoshBuddy v{} · crafted with ♥ by julesklord",
            app.version
        ))
        .alignment(Alignment::Center)
        .style(Style::default().fg(C_DIM)),
        area,
    );
}

fn render_welcome_logo(f: &mut Frame, area: Rect) {
    // Blocky Cat mascot head + stylized PoshBuddy title + vertical gradient
    let cat_and_text = [
        "                              ▄█▄       ▄█▄                              ",
        "                             ███████████████                             ",
        "                             ██ ▀██   ██▀ ██                             ",
        "                             ██    ▄▄▄    ██                             ",
        "                              ▀███████████▀                              ",
        "                                                                         ",
        "██████╗  ██████╗ ███████╗██╗  ██╗██████╗ ██╗   ██╗██████╗ ██████╗ ██╗   ██╗",
        "██╔══██╗██╔═══██╗██╔════╝██║  ██║██╔══██╗██║   ██║██╔══██╗██╔══██╗╚██╗ ██╔╝",
        "██████╔╝██║   ██║███████╗███████║██████╔╝██║   ██║██║  ██║██║  ██║ ╚████╔╝ ",
        "██╔═══╝ ██║   ██║╚════██║██╔══██║██╔══██╗██║   ██║██║  ██║██║  ██║  ╚██╔╝  ",
        "██║     ╚██████╔╝███████║██║  ██║██████╔╝╚██████╔╝██████╔╝██████╔╝   ██║   ",
        "╚═╝      ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═════╝  ╚═════╝ ╚═════╝ ╚═════╝    ╚═╝   ",
        "                             ~ posh posh posh !! ~                           ",
    ];

    let colors = [
        Color::Rgb(66, 133, 244), // Blue
        Color::Rgb(84, 110, 246),
        Color::Rgb(102, 88, 248),
        Color::Rgb(120, 66, 250),
        Color::Rgb(138, 44, 252),
        Color::Rgb(156, 22, 254),
        Color::Rgb(175, 0, 255), // Purple
        Color::Rgb(191, 0, 223),
        Color::Rgb(207, 0, 191),
        Color::Rgb(223, 0, 159),
        Color::Rgb(239, 0, 127),
        Color::Rgb(255, 0, 95),  // Pinkish red
        Color::Rgb(255, 80, 80), // Tagline
    ];

    let mut lines = Vec::new();
    for (i, line) in cat_and_text.iter().enumerate() {
        lines.push(Line::from(Span::styled(
            *line,
            Style::default()
                .fg(colors[i % colors.len()])
                .add_modifier(Modifier::BOLD),
        )));
    }

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn render_session_identity(f: &mut Frame, area: Rect, app: &App) {
    let username = whoami::username().unwrap_or_else(|_| "User".to_string());
    let hostname = whoami::hostname().unwrap_or_else(|_| "Host".to_string());
    let os = std::env::consts::OS;
    let sys_info = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Account:     ", Style::default().fg(C_DIM)),
            Span::styled(
                username,
                Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" @ {}", hostname), Style::default().fg(C_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  System:      ", Style::default().fg(C_DIM)),
            Span::styled(os.to_uppercase(), Style::default().fg(C_WHITE)),
            Span::styled(
                format!(" ({})", std::env::consts::ARCH),
                Style::default().fg(C_DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Status:      ", Style::default().fg(C_DIM)),
            Span::styled("󱐋 ", Style::default().fg(C_LOCAL)),
            Span::styled(
                "ACTIVE",
                Style::default().fg(C_LOCAL).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Backups:     ", Style::default().fg(C_DIM)),
            Span::styled(
                format!("{} archived", app.total_backups),
                Style::default().fg(C_WHITE),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(sys_info).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_DIM))
                .title(" Session Identity "),
        ),
        area,
    );
}

fn render_environment_info(f: &mut Frame, area: Rect, app: &App) {
    if area.height > 5 {
        let is_pwsh_7 = app
            .system_specs
            .as_ref()
            .map(|s| s.is_pwsh_7)
            .unwrap_or(false);
        let is_wt = app
            .system_specs
            .as_ref()
            .map(|s| s.is_windows_terminal)
            .unwrap_or(false);
        let has_nf = app
            .system_specs
            .as_ref()
            .map(|s| s.has_nerd_font)
            .unwrap_or(false);

        let info = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  SHELL     ",
                    Style::default().fg(Color::Rgb(138, 180, 248)),
                ),
                Span::styled("󱆃 ", Style::default().fg(Color::Rgb(138, 180, 248))),
                Span::raw(if is_pwsh_7 { "pwsh 7+" } else { "powershell" }),
            ]),
            Line::from(vec![
                Span::styled(
                    "  TERM      ",
                    Style::default().fg(Color::Rgb(197, 138, 249)),
                ),
                Span::styled("󰆍 ", Style::default().fg(Color::Rgb(197, 138, 249))),
                Span::raw(if is_wt { "Windows Terminal" } else { "Console" }),
            ]),
            Line::from(vec![
                Span::styled(
                    "  FONTS     ",
                    Style::default().fg(Color::Rgb(247, 137, 215)),
                ),
                Span::styled(
                    if has_nf { "󰄬 " } else { "󰅖 " },
                    Style::default().fg(if has_nf { C_LOCAL } else { C_ERROR }),
                ),
                Span::raw(if has_nf {
                    "Nerd Font Active"
                } else {
                    "Nerd Font Missing"
                }),
            ]),
            Line::from(vec![
                Span::styled(
                    "  VERSION   ",
                    Style::default().fg(Color::Rgb(138, 180, 248)),
                ),
                Span::styled("󰚀 ", Style::default().fg(Color::Rgb(138, 180, 248))),
                Span::raw(format!("v{}", app.version)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("● ", Style::default().fg(Color::Rgb(138, 180, 248))),
                Span::styled("● ", Style::default().fg(Color::Rgb(197, 138, 249))),
                Span::styled("● ", Style::default().fg(Color::Rgb(247, 137, 215))),
                Span::styled("● ", Style::default().fg(C_ACCENT)),
                Span::styled("● ", Style::default().fg(C_LOCAL)),
                Span::styled("● ", Style::default().fg(C_DIM)),
            ]),
        ];

        f.render_widget(
            Paragraph::new(info).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(C_DIM))
                    .title(" Environment "),
            ),
            area,
        );
    }
}

fn render_quick_steps(f: &mut Frame, area: Rect, app: &App) {
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
                Style::default()
                    .fg(C_DIM)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(C_DIM).bg(Color::DarkGray),
            )
        } else if is_disabled {
            (Style::default().fg(C_DIM), Style::default().fg(C_DIM))
        } else if is_selected {
            (
                Style::default()
                    .fg(C_BLACK)
                    .bg(C_ACCENT)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(C_WHITE)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                Style::default().fg(C_WHITE),
            )
        };

        let final_label = if is_disabled {
            format!("{} [Coming Soon]", label)
        } else {
            label.to_string()
        };

        actions.push(Line::from(vec![
            Span::styled(format!("  [{}]", key), key_style),
            Span::styled(format!(" {} ({})", final_label, mnemonic), label_style),
        ]));
    }

    f.render_widget(
        Paragraph::new(actions).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_DIM))
                .title(" Quick Steps "),
        ),
        area,
    );
}

pub(crate) fn render_loading(f: &mut Frame, area: Rect, app: &App) {
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

pub(crate) fn render_dep_missing(f: &mut Frame, area: Rect) {
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

pub(crate) fn render_installing_dep(f: &mut Frame, area: Rect, log: &[String], current: &str) {
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
