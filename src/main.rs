use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::future::join_all;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::fs;
use std::io;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(PartialEq)]
enum AppState {
    Loading,
    Main,
    Error(String),
}

enum AppMessage {
    Loaded(Vec<String>),
    Error(String),
}

struct App {
    state: AppState,
    themes: Vec<String>,
    filter: String,
    themes_dir: PathBuf,
    profile_path: PathBuf,
    version: String,
    list_state: ListState,
    spinner_tick: usize,
}

impl App {
    fn new() -> Self {
        let home = dirs::home_dir().expect("No se pudo encontrar el directorio home");
        let themes_dir = home.join(".poshthemes");
        
        // Determinar ruta del perfil según el OS
        let profile_path = if cfg!(windows) {
            home.join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")
        } else {
            // En WSL/Linux, buscamos el perfil de pwsh
            home.join(".config/powershell/Microsoft.PowerShell_profile.ps1")
        };

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        App {
            state: AppState::Loading,
            themes: Vec::new(),
            filter: String::new(),
            themes_dir,
            profile_path,
            version: "0.2.0-rust".to_string(),
            list_state,
            spinner_tick: 0,
        }
    }

    fn filtered_themes(&self) -> Vec<String> {
        self.themes
            .iter()
            .filter(|t| t.to_lowercase().contains(&self.filter.to_lowercase()))
            .cloned()
            .collect()
    }

    fn apply_theme(&self, theme_name: &str) -> io::Result<()> {
        let theme_path = self.themes_dir.join(theme_name);
        let config_line = format!(
            "oh-my-posh init pwsh --config '{}' | Invoke-Expression",
            theme_path.display()
        );

        let content = if self.profile_path.exists() {
            fs::read_to_string(&self.profile_path)?
        } else {
            String::new()
        };

        let mut new_content = Vec::new();
        let mut found = false;

        for line in content.lines() {
            if line.to_lowercase().contains("oh-my-posh init") {
                new_content.push(config_line.clone());
                found = true;
            } else {
                new_content.push(line.to_string());
            }
        }

        if !found {
            if !content.is_empty() {
                new_content.push("".to_string());
            }
            new_content.push(config_line);
        }

        if let Some(parent) = self.profile_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.profile_path, new_content.join("\n"))?;
        Ok(())
    }
}

// Función para obtener la lista de nombres de temas desde GitHub
async fn fetch_theme_names() -> Result<Vec<String>, String> {
    let url = "https://api.github.com/repos/JanDeDobbeleer/oh-my-posh/contents/themes";
    let client = reqwest::Client::builder()
        .user_agent("PoshBuddy-Rust")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<serde_json::Value>>()
        .await
        .map_err(|e| e.to_string())?;

    let themes = resp
        .into_iter()
        .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
        .filter(|s| s.ends_with(".omp.json"))
        .collect();

    Ok(themes)
}

// Tarea para descargar todos los temas
async fn download_all_themes_task(tx: mpsc::Sender<AppMessage>, themes_dir: PathBuf) {
    let theme_names = match fetch_theme_names().await {
        Ok(names) => names,
        Err(e) => {
            let _ = tx.send(AppMessage::Error(e)).await;
            return;
        }
    };

    if !themes_dir.exists() {
        if let Err(e) = fs::create_dir_all(&themes_dir) {
            let _ = tx.send(AppMessage::Error(format!("Error creando carpeta: {}", e))).await;
            return;
        }
    }

    let client = reqwest::Client::builder()
        .user_agent("PoshBuddy-Rust")
        .build()
        .unwrap_or_default();

    let download_futures = theme_names.iter().map(|name| {
        let client = client.clone();
        let name = name.clone();
        let path = themes_dir.join(&name);
        async move {
            if path.exists() {
                return Ok(()); // Ya descargado
            }
            let url = format!(
                "https://raw.githubusercontent.com/JanDeDobbeleer/oh-my-posh/main/themes/{}",
                name
            );
            let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
            let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
            fs::write(path, bytes).map_err(|e| e.to_string())
        }
    });

    // Ejecutar en paralelo (limitando un poco si fuera necesario, pero join_all está bien aquí)
    let _results = join_all(download_futures).await;
    
    // Contar errores opcionalmente, pero enviar el mensaje de éxito
    let _ = tx.send(AppMessage::Loaded(theme_names)).await;
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    match &app.state {
        AppState::Loading => {
            let area = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(3),
                    Constraint::Percentage(40),
                ])
                .split(area);

            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let char = spinner_chars[app.spinner_tick % spinner_chars.len()];
            app.spinner_tick += 1;

            let loading_text = format!("{} Configurando PoshBuddy por primera vez...", char);
            let p = Paragraph::new(loading_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
            
            f.render_widget(p, chunks[1]);
        }
        AppState::Error(msg) => {
            let area = f.size();
            let p = Paragraph::new(format!("Error fatal: {}\n\nPresiona 'q' para salir.", msg))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("ERROR"))
                .style(Style::default().fg(Color::Red));
            f.render_widget(p, area);
        }
        AppState::Main => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(f.size());

            let filtered = app.filtered_themes();
            
            // Sincronizar selección si el filtro cambia
            if let Some(selected) = app.list_state.selected() {
                if selected >= filtered.len() && !filtered.is_empty() {
                    app.list_state.select(Some(0));
                }
            }

            let items: Vec<ListItem> = filtered
                .iter()
                .map(|t| ListItem::new(t.as_str()))
                .collect();

            let themes_list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(format!(" Temas v{} ", app.version)))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");

            f.render_stateful_widget(themes_list, chunks[0], &mut app.list_state);

            let selected_theme = app.list_state.selected()
                .and_then(|i| filtered.get(i));

            let preview_text = match selected_theme {
                Some(theme) => format!(
                    "\n  Tema seleccionado: {}\n\n  Filtro actual: {}\n\n  [ENTER] Aplicar Tema\n  [ESC/Q] Salir\n\n  Usa la rueda del ratón para navegar",
                    theme, app.filter
                ),
                None => format!("\n  No se encontraron temas con el filtro: {}\n\n  [ESC/Q] Salir", app.filter),
            };

            let preview = Paragraph::new(preview_text)
                .block(Block::default().borders(Borders::ALL).title(" Vista Previa / Info "));
            f.render_widget(preview, chunks[1]);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let (tx, mut rx) = mpsc::channel(1);

    // Lógica de carga: si ya hay temas, entrar directo. Si no, descargar.
    let themes_already_present = if app.themes_dir.exists() {
        let entries = fs::read_dir(&app.themes_dir)?;
        let count = entries.filter_map(|e| e.ok()).count();
        count > 5 // Un umbral razonable
    } else {
        false
    };

    if themes_already_present {
        let mut themes: Vec<String> = fs::read_dir(&app.themes_dir)?
            .filter_map(|res| res.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|s| s.ends_with(".omp.json"))
            .collect();
        themes.sort();
        app.themes = themes;
        app.state = AppState::Main;
    } else {
        let themes_dir = app.themes_dir.clone();
        tokio::spawn(download_all_themes_task(tx, themes_dir));
    }

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Revisar mensajes de la tarea de fondo
        while let Ok(msg) = rx.try_recv() {
            match msg {
                AppMessage::Loaded(mut names) => {
                    names.sort();
                    app.themes = names;
                    app.state = AppState::Main;
                }
                AppMessage::Error(e) => {
                    app.state = AppState::Error(e);
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }

                if app.state == AppState::Main {
                    let filtered_count = app.filtered_themes().len();
                    
                    match key.code {
                        KeyCode::Down => {
                            let i = match app.list_state.selected() {
                                Some(i) if filtered_count > 0 => (i + 1) % filtered_count,
                                _ => 0,
                            };
                            app.list_state.select(Some(i));
                        }
                        KeyCode::Up => {
                            let i = match app.list_state.selected() {
                                Some(i) if filtered_count > 0 => (i + filtered_count - 1) % filtered_count,
                                _ => 0,
                            };
                            app.list_state.select(Some(i));
                        }
                        KeyCode::Enter => {
                            let filtered = app.filtered_themes();
                            if let Some(selected) = app.list_state.selected() {
                                if let Some(theme) = filtered.get(selected) {
                                    app.apply_theme(theme)?;
                                    // Feedback visual rápido o salir
                                    break;
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            app.filter.push(c);
                            app.list_state.select(Some(0));
                        }
                        KeyCode::Backspace => {
                            app.filter.pop();
                            app.list_state.select(Some(0));
                        }
                        _ => {}
                    }
                }
            } else if let Event::Mouse(mouse) = event::read()? {
                if app.state == AppState::Main {
                    let filtered_count = app.filtered_themes().len();
                    match mouse.kind {
                        MouseEventKind::ScrollDown => {
                            let i = match app.list_state.selected() {
                                Some(i) if filtered_count > 0 => (i + 1) % filtered_count,
                                _ => 0,
                            };
                            app.list_state.select(Some(i));
                        }
                        MouseEventKind::ScrollUp => {
                            let i = match app.list_state.selected() {
                                Some(i) if filtered_count > 0 => (i + filtered_count - 1) % filtered_count,
                                _ => 0,
                            };
                            app.list_state.select(Some(i));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
