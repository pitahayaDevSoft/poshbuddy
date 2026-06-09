use crate::app::{ActiveView, App, AppState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, Padding, Paragraph},
};

// Submodules declaration
pub(crate) mod components;
pub(crate) mod fonts;
pub(crate) mod segments;
pub(crate) mod themes;
pub(crate) mod welcome;

#[cfg(test)]
mod tests;

// ── Design Tokens ─────────────────────────────────────────────────────────────
// Rich RGB palette for a premium TUI look
pub(crate) const C_ACCENT: Color = Color::Rgb(99, 179, 237);   // Sky blue
pub(crate) const C_LOCAL: Color = Color::Rgb(104, 211, 145);   // Mint green
pub(crate) const C_REMOTE: Color = Color::Rgb(154, 117, 234);  // Soft purple
pub(crate) const C_ACTIVE: Color = Color::Rgb(252, 196, 25);   // Warm gold
pub(crate) const C_ERROR: Color = Color::Rgb(252, 90, 90);     // Coral red
pub(crate) const C_DIM: Color = Color::Rgb(100, 110, 125);     // Muted blue-grey
pub(crate) const C_WHITE: Color = Color::Rgb(230, 237, 243);   // Near-white
pub(crate) const C_BLACK: Color = Color::Rgb(13, 17, 23);      // Near-black

// Accent gradient palette (used for logo & highlights)
pub(crate) const C_GRAD_1: Color = Color::Rgb(66, 133, 244);
pub(crate) const C_GRAD_2: Color = Color::Rgb(120, 66, 250);
pub(crate) const C_GRAD_3: Color = Color::Rgb(207, 0, 191);
pub(crate) const C_GRAD_4: Color = Color::Rgb(255, 80, 80);

pub(crate) const SPINNER: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

// ── Root dispatcher ────────────────────────────────────────────────────────────
pub fn ui(f: &mut Frame, app: &mut App) {
    match app.state.clone() {
        AppState::Welcome => welcome::render_welcome(f, f.area(), app),
        AppState::DependencyMissing => welcome::render_dep_missing(f, f.area()),
        AppState::Loading => welcome::render_loading(f, f.area(), app),
        AppState::InstallingDependency {
            log,
            current_action,
        } => {
            welcome::render_installing_dep(f, f.area(), &log, &current_action);
        }
        _ => render_main(f, f.area(), app),
    }

    render_overlays(f, app);
}

// ── Main view layouter & router ───────────────────────────────────────────────
fn render_main(f: &mut Frame, area: Rect, app: &mut App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Length(3), // tab bar
            Constraint::Min(0),    // content
            Constraint::Length(1), // footer
        ])
        .split(area);

    components::render_title_bar(f, root[0], app);
    components::render_tab_bar(f, root[1], app);

    match app.active_view {
        ActiveView::Themes => themes::render_themes(f, root[2], app),
        ActiveView::Fonts => fonts::render_fonts(f, root[2], app),
        ActiveView::Segments => segments::render_segments(f, root[2], app),
    }

    components::render_main_footer(f, root[3], app);

    // Floating modals — rendered on top of everything
    match &app.state {
        AppState::Success(msg) => {
            components::render_modal(f, area, " ✓ Applied ", msg, C_LOCAL, Some("any key"));
        }
        AppState::FontSuccess(name) => {
            components::render_modal(
                f,
                area,
                " ✓ Font Installed ",
                &format!(" '{}' installed successfully.", name),
                C_LOCAL,
                Some("any key to continue"),
            );
        }
        AppState::SegmentSuccess(name) => {
            components::render_modal(
                f,
                area,
                " ✓ Segment Toggled ",
                &format!(" '{}' toggled in your active theme.", name),
                C_LOCAL,
                Some("any key to continue"),
            );
        }
        AppState::Installing(name) => {
            components::render_modal(
                f,
                area,
                " ⏳ Working ",
                &format!(" Processing: {}\n\n This may take a moment...", name),
                C_ACCENT,
                None,
            );
        }
        AppState::Error(msg) => {
            components::render_modal(f, area, " ✗ Error ", msg, C_ERROR, Some("any key"));
        }
        _ => {}
    }
}

// ── Overlay panels (confirm modals & progress gauges) ────────────────────────
fn render_overlays(f: &mut Frame, app: &App) {
    let area = f.area();

    // 1. Confirm Mass Font Installation
    if app.state == AppState::ConfirmMassFontInstallation {
        let modal_area = components::centered_rect(62, 28, area);
        f.render_widget(Clear, modal_area);

        let block = Block::default()
            .title(" ⚠  Confirm Mass Installation ")
            .title_style(Style::default().fg(C_ACTIVE).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(C_ACTIVE))
            .padding(Padding::uniform(1));

        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("You are about to install ALL Nerd Fonts.", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  This process may take significant time and bandwidth.", Style::default().fg(C_DIM)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Do you want to proceed? "),
                Span::styled(
                    " (y) Yes ",
                    Style::default().fg(C_BLACK).bg(C_LOCAL).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    " (n) No ",
                    Style::default().fg(C_BLACK).bg(C_ERROR).add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        f.render_widget(Paragraph::new(text).block(block), modal_area);
    }

    // 2. Installation Progress Gauge
    if let AppState::InstallingAllFonts {
        progress,
        current_font,
        index,
        total,
    } = &app.state
    {
        let modal_area = components::centered_rect(70, 22, area);
        f.render_widget(Clear, modal_area);

        let block = Block::default()
            .title(format!(" 󰌷 Installing Nerd Fonts ({}/{}) ", index, total))
            .title_style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(C_ACCENT));

        let gauge = Gauge::default()
            .block(Block::default().padding(Padding::new(2, 2, 1, 1)))
            .gauge_style(Style::default().fg(C_ACCENT).bg(Color::Rgb(30, 40, 55)))
            .percent(*progress as u16)
            .label(Span::styled(
                format!("{:.1}%", progress),
                Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
            ));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(block.inner(modal_area));

        f.render_widget(block, modal_area);
        f.render_widget(
            Paragraph::new(vec![Line::from(vec![
                Span::styled("  Current  ", Style::default().fg(C_DIM)),
                Span::styled(
                    current_font,
                    Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
                ),
            ])]),
            layout[0],
        );
        f.render_widget(gauge, layout[1]);
    }

    // 3. Theme Applying Progress Gauge
    if let AppState::ApplyingProgress {
        name,
        stage,
        progress,
    } = &app.state
    {
        let modal_area = components::centered_rect(62, 22, area);
        f.render_widget(Clear, modal_area);

        let (stage_icon, stage_label) = match stage {
            0 => ("⬇", "Downloading"),
            1 => ("🔍", "Verifying  "),
            2 => ("💾", "Backing up "),
            3 => ("⚡", "Applying   "),
            _ => ("⏳", "Working    "),
        };

        let block = Block::default()
            .title(format!(" {} {} ", stage_icon, stage_label))
            .title_style(Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(C_ACCENT));

        // Filled progress bar using block chars
        let bar_width = (modal_area.width as usize).saturating_sub(8).max(1);
        let filled = ((progress / 100.0) * bar_width as f32) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        let gauge = Gauge::default()
            .block(Block::default().padding(Padding::new(2, 2, 1, 1)))
            .gauge_style(Style::default().fg(C_GRAD_2).bg(Color::Rgb(30, 40, 55)))
            .percent(*progress as u16)
            .label(Span::styled(
                format!("{:.1}%", progress),
                Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
            ));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(block.inner(modal_area));

        f.render_widget(block, modal_area);

        let _ = bar; // silence unused warning; bar is illustrative

        f.render_widget(
            Paragraph::new(vec![Line::from(vec![
                Span::styled("  Theme  ", Style::default().fg(C_DIM)),
                Span::styled(
                    name,
                    Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD),
                ),
            ])]),
            layout[0],
        );
        f.render_widget(gauge, layout[1]);
    }
}
