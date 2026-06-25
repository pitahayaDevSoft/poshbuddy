use crate::app::App;
use crate::ui::components::centered_rect;
use crate::ui::{
    C_ACCENT, C_ACTIVE, C_BLACK, C_DIM, C_ERROR, C_GRAD_1, C_GRAD_2, C_GRAD_3, C_GRAD_4, C_LOCAL,
    C_WHITE, SPINNER,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

pub(crate) fn render_welcome(f: &mut Frame, area: Rect, app: &App) {
    // Responsive Guard
    if area.width < 40 || area.height < 10 {
        f.render_widget(
            Paragraph::new("Terminal too small.\nPlease resize your window.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(C_ERROR)),
            area,
        );
        return;
    }

    let has_space_for_logo = area.height > 23;

    let constraints = if has_space_for_logo {
        vec![
            Constraint::Length(8), // Logo
            Constraint::Length(3), // Dashboard Title
            Constraint::Fill(1),   // Stats & Actions
            Constraint::Length(3), // Next Step Hint
            Constraint::Length(1), // Footer
        ]
    } else {
        vec![
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Dashboard Title
            Constraint::Fill(1),   // Stats & Actions
            Constraint::Length(2), // Hint
            Constraint::Length(1), // Footer
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;

    if has_space_for_logo {
        render_welcome_logo(f, chunks[idx]);
        idx += 1;
    }

    render_welcome_header(f, chunks[idx]);
    idx += 1;

    let is_narrow = area.width < 90;
    render_welcome_body(f, chunks[idx], app, is_narrow);
    idx += 1;

    render_welcome_hint(f, chunks[idx], is_narrow);
    idx += 1;

    render_welcome_footer(f, chunks[idx], app);
}

fn render_welcome_header(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled("━━━━  ", Style::default().fg(Color::Rgb(40, 55, 75))),
        Span::styled(
            "D",
            Style::default().fg(C_GRAD_1).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "A",
            Style::default()
                .fg(Color::Rgb(90, 100, 248))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "S",
            Style::default().fg(C_GRAD_2).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "H",
            Style::default()
                .fg(Color::Rgb(170, 50, 240))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "B",
            Style::default().fg(C_GRAD_3).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "O",
            Style::default()
                .fg(Color::Rgb(240, 30, 160))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "A",
            Style::default()
                .fg(Color::Rgb(252, 60, 120))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "R",
            Style::default().fg(C_GRAD_4).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "D",
            Style::default()
                .fg(Color::Rgb(255, 80, 80))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ━━━━", Style::default().fg(Color::Rgb(40, 55, 75))),
    ]);
    f.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_welcome_body(f: &mut Frame, area: Rect, app: &App, is_narrow: bool) {
    let body_chunks = if is_narrow {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)])
            .split(area)
    } else {
        let centered_body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(110),
                Constraint::Min(0),
            ])
            .split(area)[1];

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Length(2),
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

    // Left Column: Session Identity + Environment
    let left_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(0)])
        .split(left_area);

    render_session_identity(f, left_column[0], app);
    render_environment_info(f, left_column[1], app);

    render_quick_steps(f, right_area, app);
}

fn render_welcome_hint(f: &mut Frame, area: Rect, is_narrow: bool) {
    let line = Line::from(vec![
        Span::styled("  ↑↓ ", Style::default().fg(C_ACCENT)),
        Span::styled("Navigate  ", Style::default().fg(C_DIM)),
        Span::styled("Enter ", Style::default().fg(C_ACCENT)),
        Span::styled("Select  ", Style::default().fg(C_DIM)),
        Span::styled("[1-8] ", Style::default().fg(C_ACTIVE)),
        Span::styled(
            if is_narrow {
                "Quick Keys"
            } else {
                "Quick Actions  [T F S R N I D B] Mnemonics"
            },
            Style::default().fg(C_DIM),
        ),
    ]);
    f.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_welcome_footer(f: &mut Frame, area: Rect, app: &App) {
    let line = Line::from(vec![
        Span::styled("  🐱 PoshBuddy ", Style::default().fg(C_GRAD_2)),
        Span::styled(format!("v{}", app.version), Style::default().fg(C_DIM)),
        Span::styled("  ·  crafted with ♥ by ", Style::default().fg(C_DIM)),
        Span::styled(
            "julesklord",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::ITALIC),
        ),
        Span::styled("  ·  ", Style::default().fg(Color::Rgb(40, 55, 75))),
        Span::styled(
            "github.com/julesklord/poshbuddy",
            Style::default().fg(Color::Rgb(60, 80, 110)),
        ),
    ]);
    f.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_welcome_logo(f: &mut Frame, area: Rect) {
    let logo_left = [
        "  ▄█▄       ▄█▄  ",
        " ███████████████ ",
        " ██ ▀██   ██▀ ██ ",
        " ██    ▄▄▄    ██ ",
        "  ▀███████████▀  ",
        "                 ",
    ];

    let logo_right = [
        "██████╗  ██████╗ ███████╗██╗  ██╗██████╗ ██╗   ██╗██████╗ ██████╗ ██╗   ██╗",
        "██╔══██╗██╔═══██╗██╔════╝██║  ██║██╔══██╗██║   ██║██╔══██╗██╔══██╗╚██╗ ██╔╝",
        "██████╔╝██║   ██║███████╗███████║██████╔╝██║   ██║██║  ██║██║  ██║ ╚████╔╝ ",
        "██╔═══╝ ██║   ██║╚════██║██╔══██║██╔══██╗██║   ██║██║  ██║██║  ██║  ╚██╔╝  ",
        "██║     ╚██████╔╝███████║██║  ██║██████╔╝╚██████╔╝██████╔╝██████╔╝   ██║   ",
        "╚═╝      ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═════╝  ╚═════╝ ╚═════╝ ╚═════╝    ╚═╝   ",
    ];

    let colors = [
        C_GRAD_1,
        Color::Rgb(84, 110, 246),
        Color::Rgb(102, 88, 248),
        C_GRAD_2,
        Color::Rgb(138, 44, 252),
        Color::Rgb(156, 22, 254),
        Color::Rgb(175, 0, 255),
        Color::Rgb(191, 0, 223),
        C_GRAD_3,
        Color::Rgb(223, 0, 159),
        Color::Rgb(239, 0, 127),
        C_GRAD_4,
        Color::Rgb(255, 120, 80),
    ];

    let mut lines = Vec::with_capacity(7);
    for i in 0..6 {
        let color = colors[(i * 2) % colors.len()];
        lines.push(Line::from(vec![
            Span::styled(
                logo_left[i],
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                logo_right[i],
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Add subtitle centered under the right side title
    // "logo_left[i] + spacer" is 17 + 4 = 21 chars wide.
    // The title is 75 chars wide.
    // We want to center "~ posh posh posh !! ~" (25 chars) under the 75-character title.
    // Offset inside the 75 chars is (75 - 25) / 2 = 25 chars.
    // So total spaces before subtitle is 21 + 25 = 46.
    // We pad the end with 25 spaces to make the line width exactly 96 chars (46 + 25 + 25 = 96).
    let padding_left = " ".repeat(46);
    let padding_right = " ".repeat(25);
    let subtitle_color = colors[12 % colors.len()];
    lines.push(Line::from(vec![
        Span::raw(padding_left),
        Span::styled(
            "~ posh posh posh !! ~",
            Style::default()
                .fg(subtitle_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(padding_right),
    ]));

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn render_session_identity(f: &mut Frame, area: Rect, app: &App) {
    let username = whoami::username().unwrap_or_else(|_| "User".to_string());
    let hostname = whoami::hostname().unwrap_or_else(|_| "Host".to_string());
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let sys_info = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  󰀄  Account  ", Style::default().fg(C_DIM)),
            Span::styled(
                &username,
                Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  @  ", Style::default().fg(C_DIM)),
            Span::styled(&hostname, Style::default().fg(C_ACCENT)),
        ]),
        Line::from(vec![
            Span::styled("  󰻀  System   ", Style::default().fg(C_DIM)),
            Span::styled(os.to_uppercase(), Style::default().fg(C_WHITE)),
            Span::styled(format!(" / {}", arch), Style::default().fg(C_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  󱐋  Status   ", Style::default().fg(C_DIM)),
            Span::styled("● ", Style::default().fg(C_LOCAL)),
            Span::styled(
                "ACTIVE",
                Style::default().fg(C_LOCAL).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  󰁯  Backups  ", Style::default().fg(C_DIM)),
            Span::styled(
                format!("{} archived", app.total_backups),
                Style::default().fg(C_WHITE),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
                Style::default().fg(Color::Rgb(35, 45, 60)),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(sys_info).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(45, 60, 80)))
                .title(Span::styled(
                    " 󰋼 Session ",
                    Style::default().fg(C_GRAD_1).add_modifier(Modifier::BOLD),
                )),
        ),
        area,
    );
}

fn render_environment_info(f: &mut Frame, area: Rect, app: &App) {
    if area.height <= 5 {
        return;
    }

    let shell_name = app
        .system_specs
        .as_ref()
        .map(|s| s.shell_name.as_str())
        .unwrap_or("Unknown");
    let terminal_name = app
        .system_specs
        .as_ref()
        .map(|s| s.terminal_name.as_str())
        .unwrap_or("Unknown");
    let has_nf = app
        .system_specs
        .as_ref()
        .map(|s| s.has_nerd_font)
        .unwrap_or(false);

    let shell_color = C_LOCAL;
    let term_color = C_LOCAL;
    let font_color = if has_nf { C_LOCAL } else { C_ERROR };

    let info = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  󱆃  Shell    ", Style::default().fg(C_DIM)),
            Span::styled("● ", Style::default().fg(shell_color)),
            Span::styled(shell_name, Style::default().fg(C_WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  󰆍  Terminal ", Style::default().fg(C_DIM)),
            Span::styled("● ", Style::default().fg(term_color)),
            Span::styled(terminal_name, Style::default().fg(C_WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  󰛖  Fonts    ", Style::default().fg(C_DIM)),
            Span::styled(
                if has_nf { "✓ " } else { "✗ " },
                Style::default().fg(font_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if has_nf {
                    "Nerd Font active"
                } else {
                    "Nerd Font missing!"
                },
                Style::default().fg(if has_nf { C_WHITE } else { C_ERROR }),
            ),
        ]),
        Line::from(vec![
            Span::styled("  󰚀  Version  ", Style::default().fg(C_DIM)),
            Span::styled("● ", Style::default().fg(C_ACCENT)),
            Span::styled(format!("v{}", app.version), Style::default().fg(C_WHITE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("● ", Style::default().fg(C_GRAD_1)),
            Span::styled("● ", Style::default().fg(C_GRAD_2)),
            Span::styled("● ", Style::default().fg(C_GRAD_3)),
            Span::styled("● ", Style::default().fg(C_ACCENT)),
            Span::styled("● ", Style::default().fg(C_LOCAL)),
            Span::styled("● ", Style::default().fg(C_DIM)),
        ]),
    ];

    f.render_widget(
        Paragraph::new(info).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(45, 60, 80)))
                .title(Span::styled(
                    " 󰢹 Environment ",
                    Style::default().fg(C_GRAD_2).add_modifier(Modifier::BOLD),
                )),
        ),
        area,
    );
}

fn render_quick_steps(f: &mut Frame, area: Rect, app: &App) {
    // Icons + labels + mnemonics + key
    let action_entries: &[(&str, &str, &str, &str, bool)] = &[
        ("󰔰", " Explore Themes   ", "T", "1", false),
        ("󰛖", " Install Fonts    ", "F", "2", false),
        ("󰓣", " Manage Segments  ", "S", "3", false),
        ("󰔉", " Randomize Style  ", "R", "4", false),
        ("󰏔", " Install ALL Fonts", "N", "5", false),
        ("󰙲", " Terminal Icons   ", "I", "6", false),
        ("󰙔", " Diagnostics      ", "D", "7", true), // Coming soon
        ("󰁯", " Manual Backup    ", "B", "8", false),
    ];

    let mut actions = vec![Line::from("")];
    for (i, (icon, label, mnemonic, key, is_disabled)) in action_entries.iter().enumerate() {
        let is_selected = i == app.welcome_selected_action;

        let (key_bg, key_fg, row_fg, row_bg) = if *is_disabled && is_selected {
            (Color::DarkGray, C_DIM, C_DIM, Color::Rgb(25, 30, 40))
        } else if *is_disabled {
            (Color::Reset, C_DIM, C_DIM, Color::Reset)
        } else if is_selected {
            (C_ACCENT, C_BLACK, C_WHITE, Color::Rgb(20, 35, 55))
        } else {
            (
                Color::Reset,
                C_ACCENT,
                Color::Rgb(170, 185, 205),
                Color::Reset,
            )
        };

        let suffix = if *is_disabled { " [Soon]" } else { "" };

        actions.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("[{}]", key),
                Style::default().fg(key_fg).bg(key_bg).add_modifier(
                    if is_selected && !is_disabled {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    },
                ),
            ),
            Span::styled(
                format!(" {} ", icon),
                Style::default().fg(if is_selected && !is_disabled {
                    C_ACCENT
                } else {
                    C_DIM
                }),
            ),
            Span::styled(
                format!("{}{}", label, suffix),
                Style::default().fg(row_fg).bg(row_bg).add_modifier(
                    if is_selected && !is_disabled {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    },
                ),
            ),
            Span::styled(format!(" ({})", mnemonic), Style::default().fg(C_DIM)),
        ]));
    }

    f.render_widget(
        Paragraph::new(actions).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(45, 60, 80)))
                .title(Span::styled(
                    " ⚡ Quick Actions ",
                    Style::default().fg(C_ACTIVE).add_modifier(Modifier::BOLD),
                )),
        ),
        area,
    );
}

pub(crate) fn render_loading(f: &mut Frame, area: Rect, app: &App) {
    let spin = SPINNER[app.spinner_tick % SPINNER.len()];
    let center = centered_rect(52, 32, area);
    f.render_widget(Clear, center);

    // Animated spinner color cycling based on tick
    let spin_color = match app.spinner_tick % 4 {
        0 => C_GRAD_1,
        1 => C_GRAD_2,
        2 => C_GRAD_3,
        _ => C_ACCENT,
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    spin,
                    Style::default().fg(spin_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "  Loading PoshBuddy...",
                    Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Scanning themes, fonts, and shell profiles",
                Style::default().fg(C_DIM),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("● ", Style::default().fg(C_GRAD_1)),
                Span::styled("● ", Style::default().fg(C_GRAD_2)),
                Span::styled("● ", Style::default().fg(C_GRAD_3)),
                Span::styled("● ", Style::default().fg(C_ACCENT)),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(spin_color))
                .title(Span::styled(
                    " PoshBuddy ",
                    Style::default().fg(spin_color).add_modifier(Modifier::BOLD),
                )),
        ),
        center,
    );
}

fn has_brew() -> bool {
    if let Some(paths) = std::env::var_os("PATH") {
        for path in std::env::split_paths(&paths) {
            if path.join("brew").is_file() {
                return true;
            }
        }
    }
    let common_brew_paths = [
        "/opt/homebrew/bin/brew",
        "/usr/local/bin/brew",
        "/home/linuxbrew/.linuxbrew/bin/brew",
    ];
    for path_str in &common_brew_paths {
        if std::path::Path::new(path_str).is_file() {
            return true;
        }
    }
    false
}

pub(crate) fn render_dep_missing(f: &mut Frame, area: Rect) {
    let center = centered_rect(64, 46, area);
    f.render_widget(Clear, center);

    let (installer_desc, manual_cmd) = if cfg!(windows) {
        (
            "Auto-install via WinGet",
            "winget install JanDeDobbeleer.OhMyPosh",
        )
    } else if has_brew() {
        ("Auto-install via Homebrew", "brew install oh-my-posh")
    } else {
        (
            "Auto-install via Official Script",
            "curl -s https://ohmyposh.dev/install.sh | bash -s",
        )
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  ✗  ",
                    Style::default().fg(C_ERROR).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Oh My Posh not found",
                    Style::default().fg(C_ERROR).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  PoshBuddy requires Oh My Posh to work properly.",
                Style::default().fg(C_DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  ──────────────────────────────────────────",
                Style::default().fg(Color::Rgb(40, 55, 75)),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Enter  ",
                    Style::default()
                        .fg(C_BLACK)
                        .bg(C_ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(installer_desc, Style::default().fg(C_WHITE)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Q      ",
                    Style::default().fg(C_DIM).bg(Color::Rgb(40, 50, 65)),
                ),
                Span::raw("  "),
                Span::styled("Quit and install manually", Style::default().fg(C_DIM)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  ──────────────────────────────────────────",
                Style::default().fg(Color::Rgb(40, 55, 75)),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Manual  ", Style::default().fg(C_DIM)),
                Span::styled(
                    manual_cmd,
                    Style::default().fg(C_ACTIVE).add_modifier(Modifier::ITALIC),
                ),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(C_ERROR))
                .title(Span::styled(
                    " ✗ Missing Dependency ",
                    Style::default().fg(C_ERROR).add_modifier(Modifier::BOLD),
                )),
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
        Paragraph::new(format!("  ⚡ {}", current))
            .style(Style::default().fg(C_ACTIVE).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(C_ACTIVE))
                    .title(Span::styled(
                        " ⬇ Installing Oh My Posh ",
                        Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                    )),
            ),
        root[0],
    );

    let log_lines: Vec<Line> = log
        .iter()
        .map(|l| {
            Line::from(vec![
                Span::styled("  │ ", Style::default().fg(Color::Rgb(55, 70, 90))),
                Span::styled(l.as_str(), Style::default().fg(C_DIM)),
            ])
        })
        .collect();

    f.render_widget(
        Paragraph::new(log_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(45, 60, 80)))
                .title(Span::styled(
                    " 📋 Installation Log ",
                    Style::default().fg(C_DIM),
                )),
        ),
        root[1],
    );
}
