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

#[derive(PartialEq, Clone, Copy)]
enum ActiveView {
    Themes,
    Fonts,
}

#[derive(PartialEq)]
enum AppState {
    Loading,
    Main,
    Installing(String),
    Error(String),
}

#[derive(Clone, Debug)]
struct FontAsset {
    name: String,
}

enum AppMessage {
    ThemesLoaded(Vec<String>),
    FontsLoaded(Vec<FontAsset>),
    ThemePreviewLoaded(String),
    FontInstalled(String),
    Error(String),
}

struct App {
    state: AppState,
    active_view: ActiveView,
    themes: Vec<String>,
    fonts: Vec<FontAsset>,
    filter: String,
    fonts_filter: String,
    themes_dir: PathBuf,
    profile_path: PathBuf,
    version: String,
    list_state: ListState,
    fonts_list_state: ListState,
    spinner_tick: usize,
    has_nerd_font: bool,
    theme_preview: String,
    last_selected_theme: Option<String>,
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

        let mut fonts_list_state = ListState::default();
        fonts_list_state.select(Some(0));

        let has_nerd_font = Self::check_nerd_font();

        App {
            state: AppState::Loading,
            active_view: ActiveView::Themes,
            themes: Vec::new(),
            fonts: Vec::new(),
            filter: String::new(),
            fonts_filter: String::new(),
            themes_dir,
            profile_path,
            version: "0.2.0-rust".to_string(),
            list_state,
            fonts_list_state,
            spinner_tick: 0,
            has_nerd_font,
            theme_preview: String::new(),
            last_selected_theme: None,
        }
    }

    fn check_nerd_font() -> bool {
        // 1. Verificar variables de entorno conocidas
        if let Ok(term_prog) = std::env::var("TERM_PROGRAM") {
            if term_prog == "vscode" {
                return true; // VS Code suele manejarlo bien si está configurado
            }
        }

        if std::env::var("TERMINAL_EMULATOR").is_ok() || std::env::var("WT_SESSION").is_ok() {
            // Windows Terminal o emuladores modernos suelen tener fuentes Nerd configuradas
            // pero vamos a intentar ser más precisos con el comando de registro si es Windows
        }

        // 2. En Windows, intentar detectar la fuente via Registry
        let cmd = if cfg!(windows) {
            "powershell"
        } else {
            "powershell.exe" // Host Windows desde WSL
        };

        let output = std::process::Command::new(cmd)
            .args(["-Command", "(Get-ItemProperty -Path 'HKCU:\\Console' -ErrorAction SilentlyContinue).FaceName"])
            .output();

        if let Ok(out) = output {
            let name = String::from_utf8_lossy(&out.stdout).to_lowercase();
            if name.trim().is_empty() {
                return true; // Si no hay valor, asumimos que es el default (que podría no ser Nerd, pero no alarmamos)
            }
            name.contains("nf") || name.contains("nerd") || name.contains("retina") || name.contains("code") || name.contains("meslo")
        } else {
            true // Fallback seguro
        }
    }

    fn filtered_themes(&self) -> Vec<String> {
        self.themes
            .iter()
            .filter(|t| t.to_lowercase().contains(&self.filter.to_lowercase()))
            .cloned()
            .collect()
    }

    fn filtered_fonts(&self) -> Vec<FontAsset> {
        self.fonts
            .iter()
            .filter(|f| f.name.to_lowercase().contains(&self.fonts_filter.to_lowercase()))
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

    fn install_font(&self, font_name: String, tx: mpsc::Sender<AppMessage>) {
        let cmd = if cfg!(windows) {
            "oh-my-posh"
        } else {
            "oh-my-posh.exe"
        };

        let font_name_cloned = font_name.clone();
        tokio::spawn(async move {
            let output = std::process::Command::new(cmd)
                .args(["font", "install", &font_name_cloned])
                .output();
            
            match output {
                Ok(_) => { let _ = tx.send(AppMessage::FontInstalled(font_name_cloned)).await; }
                Err(e) => { let _ = tx.send(AppMessage::Error(e.to_string())).await; }
            }
        });
    }

    fn load_theme_preview(&self, theme_name: String, tx: mpsc::Sender<AppMessage>) {
        let cmd = if cfg!(windows) {
            "oh-my-posh"
        } else {
            "oh-my-posh.exe"
        };
        let theme_path = self.themes_dir.join(&theme_name);

        tokio::spawn(async move {
            let output = std::process::Command::new(cmd)
                .args(["print", "primary", "--config", &theme_path.to_string_lossy(), "--shell", "pwsh"])
                .output();

            match output {
                Ok(out) => {
                    let preview = String::from_utf8_lossy(&out.stdout).to_string();
                    let _ = tx.send(AppMessage::ThemePreviewLoaded(preview)).await;
                }
                Err(_) => {
                    let _ = tx.send(AppMessage::ThemePreviewLoaded("No se pudo generar previsualización".to_string())).await;
                }
            }
        });
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

// Función para obtener la lista de Nerd Fonts desde GitHub
async fn fetch_font_names() -> Result<Vec<FontAsset>, String> {
    let url = "https://api.github.com/repos/ryanoasis/nerd-fonts/releases/latest";
    let client = reqwest::Client::builder()
        .user_agent("PoshBuddy-Rust")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    let assets = resp["assets"]
        .as_array()
        .ok_or("No se pudieron encontrar assets en la release")?;

    let fonts = assets
        .iter()
        .filter_map(|a| {
            let raw_name = a["name"].as_str()?;
            if raw_name.ends_with(".zip") {
                Some(FontAsset {
                    name: raw_name.replace(".zip", ""),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(fonts)
}

// Tarea para descargar temas y obtener fuentes
async fn setup_app_task(tx: mpsc::Sender<AppMessage>, themes_dir: PathBuf) {
    // 1. Fetch de fuentes
    match fetch_font_names().await {
        Ok(fonts) => {
            let _ = tx.send(AppMessage::FontsLoaded(fonts)).await;
        }
        Err(e) => {
            let _ = tx.send(AppMessage::Error(format!("Error fuentes: {}", e))).await;
        }
    }

    // 2. Fetch de temas
    let theme_names = match fetch_theme_names().await {
        Ok(names) => names,
        Err(e) => {
            let _ = tx.send(AppMessage::Error(format!("Error temas: {}", e))).await;
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
                return Ok(());
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

    let _results = join_all(download_futures).await;
    let _ = tx.send(AppMessage::ThemesLoaded(theme_names)).await;
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
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
            app.spinner_tick += 1;

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
                    // Limpiamos la previsualización de saltos de línea extra que rompen el layout
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let (tx, mut rx) = mpsc::channel(10);

    let themes_dir = app.themes_dir.clone();
    tokio::spawn(setup_app_task(tx.clone(), themes_dir));

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        while let Ok(msg) = rx.try_recv() {
            match msg {
                AppMessage::ThemesLoaded(mut names) => {
                    names.sort();
                    app.themes = names;
                    if app.state == AppState::Loading { app.state = AppState::Main; }
                    // Cargar primer preview
                    if let Some(t) = app.themes.first() {
                        app.load_theme_preview(t.clone(), tx.clone());
                    }
                }
                AppMessage::FontsLoaded(mut fonts) => {
                    fonts.sort_by(|a, b| a.name.cmp(&b.name));
                    app.fonts = fonts;
                }
                AppMessage::ThemePreviewLoaded(preview) => {
                    app.theme_preview = preview;
                }
                AppMessage::FontInstalled(name) => {
                    app.state = AppState::Main;
                    app.has_nerd_font = true; // Asumimos éxito tras instalar
                }
                AppMessage::Error(e) => { app.state = AppState::Error(e); }
            }
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc { break; }

                if app.state == AppState::Main {
                    match key.code {
                        KeyCode::Tab | KeyCode::Char('1') | KeyCode::Char('2') => {
                            app.active_view = if app.active_view == ActiveView::Themes { ActiveView::Fonts } else { ActiveView::Themes };
                        }
                        KeyCode::Down | KeyCode::Up => {
                            if app.active_view == ActiveView::Themes {
                                let total = app.filtered_themes().len();
                                if total > 0 {
                                    let i = match app.list_state.selected() {
                                        Some(i) => if key.code == KeyCode::Down { (i + 1) % total } else { (i + total - 1) % total },
                                        None => 0,
                                    };
                                    app.list_state.select(Some(i));
                                    if let Some(theme) = app.filtered_themes().get(i) {
                                        app.theme_preview = String::new(); // Clear while loading
                                        app.load_theme_preview(theme.clone(), tx.clone());
                                    }
                                }
                            } else {
                                let total = app.filtered_fonts().len();
                                if total > 0 {
                                    let i = match app.fonts_list_state.selected() {
                                        Some(i) => if key.code == KeyCode::Down { (i + 1) % total } else { (i + total - 1) % total },
                                        None => 0,
                                    };
                                    app.fonts_list_state.select(Some(i));
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if app.active_view == ActiveView::Themes {
                                let filtered = app.filtered_themes();
                                if let Some(selected) = app.list_state.selected() {
                                    if let Some(theme) = filtered.get(selected) {
                                        app.apply_theme(theme)?;
                                        break;
                                    }
                                }
                            } else {
                                let filtered = app.filtered_fonts();
                                if let Some(selected) = app.fonts_list_state.selected() {
                                    if let Some(font) = filtered.get(selected) {
                                        app.state = AppState::Installing(font.name.clone());
                                        app.install_font(font.name.clone(), tx.clone());
                                    }
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            if app.active_view == ActiveView::Themes { app.filter.push(c); app.list_state.select(Some(0)); }
                            else { app.fonts_filter.push(c); app.fonts_list_state.select(Some(0)); }
                        }
                        KeyCode::Backspace => {
                            if app.active_view == ActiveView::Themes { app.filter.pop(); app.list_state.select(Some(0)); }
                            else { app.fonts_filter.pop(); app.fonts_list_state.select(Some(0)); }
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
