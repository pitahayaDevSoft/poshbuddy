use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use crate::app::{App, ActiveView, AppState};

pub fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    // Header / Tabs
    let titles = vec![" [1] Temas ", " [2] Fuentes "];
    let mut header_text = String::new();
    for (i, title) in titles.iter().enumerate() {
        let is_selected = (i == 0 && app.active_view == ActiveView::Themes) || 
                          (i == 1 && app.active_view == ActiveView::Fonts);
        if is_selected {
            header_text.push_str(&format!(" >>{}<< ", title));
        } else {
            header_text.push_str(&format!("   {}   ", title));
        }
    }

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(format!(" PoshBuddy v{} ", app.version)));
    f.render_widget(header, main_layout[0]);

    match &app.state {
        AppState::Loading => {
            let area = main_layout[1];
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(40), Constraint::Length(3), Constraint::Percentage(40)])
                .split(area);

            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let char = spinner_chars[app.spinner_tick % spinner_chars.len()];
            app.spinner_tick += 1; // Note: We will move this to main.rs in Phase 3

            let loading_text = format!("{} Configurando PoshBuddy...", char);
            f.render_widget(Paragraph::new(loading_text).alignment(Alignment::Center).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)), chunks[1]);
        }
        AppState::Installing(name) => {
            let area = main_layout[1];
            let p = Paragraph::new(format!("\n\nInstalando fuente: {}\n\nEsto puede tardar un poco dependiendo de tu conexión...", name))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" INSTALACIÓN EN CURSO "))
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(p, area);
        }
        AppState::Error(msg) => {
            let area = main_layout[1];
            f.render_widget(Paragraph::new(format!("Error: {}\n\nPresiona 'q' para salir.", msg)).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL).title("ERROR")).style(Style::default().fg(Color::Red)), area);
        }
        AppState::Main => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(main_layout[1]);

            match app.active_view {
                ActiveView::Themes => {
                    let filtered = app.filtered_themes();
                    let items: Vec<ListItem> = filtered.iter().map(|t| ListItem::new(t.as_str())).collect();
                    let themes_list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title(" Temas "))
                        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> ");
                    f.render_stateful_widget(themes_list, chunks[0], &mut app.list_state);

                    // Panel derecho dividido
                    let right_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(6), // Espacio para el Prompt
                            Constraint::Min(0),    // Espacio para Info
                        ].as_ref())
                        .split(chunks[1]);

                    let selected_theme = app.list_state.selected().and_then(|i| filtered.get(i));
                    
                    // 1. Apartado del Prompt (Visual)
                    let display_preview = if app.theme_preview.is_empty() {
                        "\n    Generando prompt...".to_string()
                    } else {
                        format!("\n    {}", app.theme_preview.trim())
                    };

                    let prompt_box = Paragraph::new(display_preview)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .title(" Diseño del Prompt ")
                            .border_style(Style::default().fg(Color::Yellow)));
                    f.render_widget(prompt_box, right_chunks[0]);

                    // 2. Apartado de Información
                    let mut info_text = format!(
                        "\n  Nombre: {}\n  Ruta: ~/.poshthemes/{}\n\n  Controles:\n  [ENTER] Aplicar este tema\n  [TAB]   Explorar Fuentes\n  [Q/ESC] Salir",
                        selected_theme.unwrap_or(&"Ninguno".to_string()),
                        selected_theme.unwrap_or(&"".to_string())
                    );

                    if !app.has_nerd_font {
                        info_text.push_str("\n\n  ⚠️  Nerd Font no detectada. Instala una en la pestaña 'Fuentes'.");
                    }

                    let info_box = Paragraph::new(info_text)
                        .block(Block::default().borders(Borders::ALL).title(" Información del Tema "));
                    f.render_widget(info_box, right_chunks[1]);
                }
                ActiveView::Fonts => {
                    let filtered = app.filtered_fonts();
                    let items: Vec<ListItem> = filtered.iter().map(|f| ListItem::new(f.name.as_str())).collect();
                    let fonts_list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title(" Nerd Fonts "))
                        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> ");
                    f.render_stateful_widget(fonts_list, chunks[0], &mut app.fonts_list_state);

                    let selected_font = app.fonts_list_state.selected().and_then(|i| filtered.get(i));
                    let preview_text = match selected_font {
                        Some(font) => format!(
                            "\n  Fuente: {}\n\n  Estado: Disponible para instalar\n\n  Las Nerd Fonts son esenciales para ver los iconos\n  de Oh My Posh correctamente.\n\n  [ENTER] Descargar e Instalar automáticamente",
                            font.name
                        ),
                        None => "\n  No se encontraron fuentes".to_string(),
                    };
                    f.render_widget(Paragraph::new(preview_text).block(Block::default().borders(Borders::ALL).title(" Instalador de Fuentes ")), chunks[1]);
                }
            }
        }
    }
}
