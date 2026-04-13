use ratatui::widgets::ListState;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use tokio::sync::mpsc;

const OMP_BINARY: &str = "oh-my-posh";
const WHERE_CMD: &str = "where";

/// Helper function for zero-allocation case-insensitive ASCII substring matching
pub fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    let needle_bytes = needle.as_bytes();
    if needle_bytes.is_empty() {
        return true;
    }
    haystack
        .as_bytes()
        .windows(needle_bytes.len())
        .any(|w| w.eq_ignore_ascii_case(needle_bytes))
}

/// Metadata for a PowerShell module/extension (Legacy Plugins)
#[derive(Clone, Debug)]
pub struct PluginAsset {
    pub name: String,
    pub description: String,
    pub documentation: String,
    pub module_name: String,
    pub init_script: Option<String>,
}

/// Metadata for an Oh My Posh Segment
#[derive(Clone, Debug)]
pub struct SegmentAsset {
    pub name: String,
    pub segment_type: String,
    pub description: String,
    pub documentation: String,
    pub category: String, // e.g., "Development", "System", "Cloud"
}

/// Metadata for a font asset
#[derive(Clone, Debug, PartialEq)]
pub struct FontAsset {
    pub name: String,
    pub download_url: String,
}

/// Dynamic metadata for a remote Oh My Posh theme
#[derive(Clone, Debug, serde::Deserialize)]
pub struct RemoteTheme {
    pub name: String,
    pub download_url: String,
    pub sha: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ThemeAsset {
    pub name: String,
    pub is_local: bool,
    pub download_url: Option<String>,
}

/// System specifications for diagnostic display
#[derive(Debug, Clone, PartialEq)]
pub struct SystemSpecs {
    pub is_pwsh_7: bool,
    pub has_nerd_font: bool,
    pub is_windows_terminal: bool,
}

/// Message types sent across the mpsc channel to update the TUI from background tasks
pub enum AppMessage {
    ThemesLoaded(Vec<ThemeAsset>),
    FontsLoaded(Vec<FontAsset>),
    ThemePreviewLoaded { theme: ThemeAsset, preview: String },
    InstallProgress { line: String },
    InstallFinished,
    Error(String),
    FontInstalled(String),
    PluginInstalled(String),
    RemoteThemesLoaded(Vec<RemoteTheme>),
}

/// Represents the different states the application can be in
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Loading,
    Main,
    Onboarding(SystemSpecs),
    DependencyMissing,
    InstallingDependency {
        log: Vec<String>,
        current_action: String,
    },
    Success(String),
    FontSuccess(String),
    PluginSuccess(String),
    Installing(String),
    Error(String),
    Welcome,
}

/// Active view/tab in the main interface
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveView {
    Themes,
    Fonts,
    Segments,
}

/// State container for the PoshBuddy application
pub struct App {
    pub state: AppState,
    pub active_view: ActiveView,
    pub themes: Vec<ThemeAsset>,
    pub remote_themes: Vec<RemoteTheme>,
    pub fonts: Vec<FontAsset>,
    pub filter: String,
    pub fonts_filter: String,
    pub themes_dir: PathBuf,
    pub version: String,
    pub list_state: ListState,
    pub fonts_list_state: ListState,
    pub plugins_list_state: ListState,
    pub plugins: Vec<PluginAsset>,
    pub segments: Vec<SegmentAsset>,
    pub plugins_filter: String,
    pub segments_filter: String,
    pub spinner_tick: usize,
    pub has_nerd_font: bool,
    pub theme_preview: String,
    pub detected_profiles: Vec<PathBuf>,
    pub active_config_path: Option<PathBuf>,
    pub backup_manager: crate::backup::BackupManager,
    pub last_backup: Option<std::path::PathBuf>,
    pub diagnostic: crate::diagnostic::Diagnostic,
    pub plugin_installer: crate::plugin_installer::PluginInstaller,
    // Welcome screen state
    pub welcome_selected_action: usize, // Índice de la acción rápida seleccionada
    pub system_specs: Option<SystemSpecs>, // Cache de especificaciones del sistema
    pub total_backups: usize, // Total de perfiles respaldados encontrados
}

impl App {
    /// Initializes a new application instance with dynamic system detection
    pub fn new() -> Self {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => {
                eprintln!("Error: Could not find home directory");
                std::process::exit(1);
            }
        };
        let themes_dir = home.join(".poshthemes");

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut fonts_list_state = ListState::default();
        fonts_list_state.select(Some(0));

        let mut plugins_list_state = ListState::default();
        plugins_list_state.select(Some(0));

        // 1. Initial system diagnostics
        let has_nerd_font = Self::check_nerd_font();
        let detected_profiles = Self::detect_profiles();
        let specs = Self::gather_system_specs(has_nerd_font);        let mut app = App {
            state: AppState::Welcome,
            active_view: ActiveView::Themes,
            themes: Vec::new(),
            remote_themes: Vec::new(),
            fonts: Vec::new(),
            plugins: vec![
                PluginAsset {
                    name: "Terminal-Icons".to_string(),
                    description: "Adds file and folder icons to your terminal outputs (ls, dir).".to_string(),
                    documentation: "Requires a Nerd Font. Enhances visual data parsing in long lists.".to_string(),
                    module_name: "Terminal-Icons".to_string(),
                    init_script: None,
                },
                PluginAsset {
                    name: "zoxide (z Explorer)".to_string(),
                    description: "A smarter cd command. It remembers which directories you use most often.".to_string(),
                    documentation: "Usage: type 'z <name>' to jump. Replaces 'cd' with intelligent fuzzy matching.".to_string(),
                    module_name: "zoxide".to_string(),
                    init_script: Some("if (Get-Command zoxide -ErrorAction SilentlyContinue) { zoxide init powershell --hook pwd | Out-String | Invoke-Expression }".to_string()),
                },
                PluginAsset {
                    name: "PSReadLine Mastery".to_string(),
                    description: "Enables Predictive IntelliSense (fish-like) and syntax highlighting.".to_string(),
                    documentation: "Optimizes command history search and adds visual feedback while typing.".to_string(),
                    module_name: "PSReadLine".to_string(),
                    init_script: Some("Set-PSReadLineOption -PredictionSource History\nSet-PSReadLineOption -PredictionViewStyle ListView".to_string()),
                },
            ],
            segments: vec![
                SegmentAsset {
                    name: "Git Status".to_string(),
                    segment_type: "git".to_string(),
                    description: "Muestra la rama actual y el estado de archivos de Git.".to_string(),
                    documentation: "Esencial para el desarrollo colaborativo.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Path (Ruta)".to_string(),
                    segment_type: "path".to_string(),
                    description: "Muestra la ubicación actual en el sistema de archivos.".to_string(),
                    documentation: "Configurable para mostrar ruta completa o corta.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Session (User)".to_string(),
                    segment_type: "session".to_string(),
                    description: "Muestra el usuario y host actual.".to_string(),
                    documentation: "Útil para identificar rápidamente en qué cuenta/máquina estás.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Battery (Batería)".to_string(),
                    segment_type: "battery".to_string(),
                    description: "Visualiza el porcentaje de batería y estado de carga.".to_string(),
                    documentation: "Cambia de color según el nivel de carga.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Execution Time".to_string(),
                    segment_type: "executiontime".to_string(),
                    description: "Muestra cuánto duró el último comando ejecutado.".to_string(),
                    documentation: "Perfecto para medir rendimiento de scripts.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Node.js info".to_string(),
                    segment_type: "node".to_string(),
                    description: "Muestra la versión de Node activa en el directorio.".to_string(),
                    documentation: "Se activa automáticamente en proyectos Node.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Docker".to_string(),
                    segment_type: "docker".to_string(),
                    description: "Muestra el estado de Docker y el contexto actual.".to_string(),
                    documentation: "Requiere que Docker esté instalado y corriendo.".to_string(),
                    category: "Cloud".to_string(),
                },
            ],
            filter: String::new(),
            fonts_filter: String::new(),
            plugins_filter: String::new(),
            segments_filter: String::new(),
            themes_dir,
            version: env!("CARGO_PKG_VERSION").to_string(),
            list_state,
            fonts_list_state,
            plugins_list_state,
            spinner_tick: 0,
            has_nerd_font,
            theme_preview: String::new(),
            detected_profiles: detected_profiles.clone(),
            active_config_path: None,
            backup_manager: crate::backup::BackupManager::new(None),
            last_backup: None,
            diagnostic: crate::diagnostic::Diagnostic::new(),
            plugin_installer: crate::plugin_installer::PluginInstaller::new(),
            welcome_selected_action: 0,
            system_specs: Some(specs),
            total_backups: 0,
        };

        // Initialize active config path
        app.active_config_path = app.find_active_config_path();

        // Initialize backup count
        app.refresh_backup_count();

        // 2. Pre-check for Oh My Posh installation
        if !app.check_omp_installed() {
            app.state = AppState::DependencyMissing;
        }

        app
    }

    /// Busca la ruta del archivo de configuración activo desde el perfil de PowerShell
    pub fn find_active_config_path(&self) -> Option<PathBuf> {
        for profile in &self.detected_profiles {
            if !profile.exists() {
                continue;
            }

            if let Ok(content) = fs::read_to_string(profile) {
                for line in content.lines() {
                    if line.contains("oh-my-posh init") && line.contains("--config") {
                        // Intentar extraer la ruta entre comillas o después de --config
                        let parts: Vec<&str> = line.split("--config").collect();
                        if parts.len() > 1 {
                            let path_part = parts[1].trim();
                            // Tomar el contenido entre comillas si existe
                            let config_path = if path_part.starts_with('"') || path_part.starts_with('\'') {
                                let quote = path_part.chars().next().unwrap();
                                path_part.split(quote).nth(1).map(|s| s.to_string())
                            } else {
                                path_part.split_whitespace().next().map(|s| s.to_string())
                            };

                            if let Some(p_str) = config_path {
                                let path = PathBuf::from(p_str);
                                if path.exists() {
                                    return Some(path);
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Verifies if 'oh-my-posh' binary is present in the system PATH
    pub fn check_omp_installed(&self) -> bool {
        let cmd = WHERE_CMD;
        std::process::Command::new(cmd)
            .arg("oh-my-posh")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Checks the current terminal environment and PowerShell version capabilities
    pub fn gather_system_specs(has_nerd_font: bool) -> SystemSpecs {
        // Detecting Windows Terminal via WT_SESSION environment variable
        let is_windows_terminal = std::env::var("WT_SESSION").is_ok()
            || std::env::var("TERM_PROGRAM")
                .map(|v| v == "vscode")
                .unwrap_or(false);

        // Checking for PowerShell 7 binary (pwsh)
        let cmd = WHERE_CMD;
        let is_pwsh_7 = std::process::Command::new(cmd)
            .arg("pwsh")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        SystemSpecs {
            is_pwsh_7,
            has_nerd_font,
            is_windows_terminal,
        }
    }

    /// Dynamically identifies all active PowerShell profile paths ($PROFILE) on the system
    pub fn detect_profiles() -> Vec<PathBuf> {
        let mut profiles = Vec::new();

        // 1. Try to guess standard paths first (Zero-latency)
        if let Some(docs) = dirs::document_dir() {
            let pwsh5 = docs.join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1");
            let pwsh7 = docs.join("PowerShell").join("Microsoft.PowerShell_profile.ps1");
            if pwsh5.exists() { profiles.push(pwsh5); }
            if pwsh7.exists() { profiles.push(pwsh7); }
        }

        // 2. If nothing found or to be sure, ask the shells (Lazy detection later would be better, but let's at least deduplicate)
        if profiles.is_empty() {
            let shells = if cfg!(windows) { vec!["powershell", "pwsh"] } else { vec!["pwsh"] };
            for shell in shells {
                if let Ok(out) = std::process::Command::new(shell)
                    .args(["-NoProfile", "-Command", "Write-Host -NoNewline $PROFILE"])
                    .output()
                {
                    let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        profiles.push(PathBuf::from(path_str));
                    }
                }
            }
        }

        profiles.sort();
        profiles.dedup();
        profiles
    }

    /// Heuristic to check if a Nerd Font is likely being used by the system
    pub fn check_nerd_font() -> bool {
        if let Ok(term_prog) = std::env::var("TERM_PROGRAM") {
            if term_prog == "vscode" {
                return true;
            }
        }

        let cmd = if cfg!(windows) {
            "powershell"
        } else {
            "powershell.exe"
        };

        let output = std::process::Command::new(cmd)
            .args([
                "-Command",
                "(Get-ItemProperty -Path 'HKCU:\\Console' -ErrorAction SilentlyContinue).FaceName",
            ])
            .output();

        if let Ok(out) = output {
            let name = String::from_utf8_lossy(&out.stdout);
            if name.trim().is_empty() {
                return true;
            }
            contains_ignore_ascii_case(&name, "nf")
                || contains_ignore_ascii_case(&name, "nerd")
                || contains_ignore_ascii_case(&name, "retina")
                || contains_ignore_ascii_case(&name, "code")
                || contains_ignore_ascii_case(&name, "meslo")
        } else {
            true
        }
    }

    /// Returns a unified list of filtered themes (Local + Unique Remote)
    pub fn filtered_themes(&self) -> Vec<ThemeAsset> {
        let filter = &self.filter;
        let mut unified = Vec::new();

        // Add Local
        for t in &self.themes {
            if contains_ignore_ascii_case(&t.name, filter) {
                unified.push(t.clone());
            }
        }

        // Add Remote (only if not local)
        for rt in &self.remote_themes {
            if contains_ignore_ascii_case(&rt.name, filter)
                && !self.themes.iter().any(|t| t.name == rt.name) {
                    unified.push(ThemeAsset {
                        name: rt.name.clone(),
                        is_local: false,
                        download_url: Some(rt.download_url.clone()),
                    });
                }
        }

        unified
    }

    /// Asynchronously fetches the official themes catalog from GitHub
    pub fn fetch_official_themes(&self, tx: mpsc::Sender<AppMessage>) {
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let res = client.get("https://api.github.com/repos/JanDeDobbeleer/oh-my-posh/contents/themes")
                .header("User-Agent", "PoshBuddy-Updater")
                .send()
                .await;

            match res {
                Ok(response) => {
                    if let Ok(items) = response.json::<Vec<serde_json::Value>>().await {
                        let themes: Vec<RemoteTheme> = items.into_iter()
                            .filter(|i| i["name"].as_str().unwrap_or("").ends_with(".omp.json"))
                            .map(|i| RemoteTheme {
                                name: i["name"].as_str().unwrap_or("").replace(".omp.json", "").to_string(),
                                download_url: i["download_url"].as_str().unwrap_or("").to_string(),
                                sha: i["sha"].as_str().unwrap_or("").to_string(),
                            })
                            .collect();
                        let _ = tx.send(AppMessage::RemoteThemesLoaded(themes)).await;
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("Fetch themes failed: {}", e))).await;
                }
            }
        });
    }

    /// Downloads and installs a remote theme locally
    pub fn download_theme(&self, theme: RemoteTheme, tx: mpsc::Sender<AppMessage>) {
        let path = self.themes_dir.join(format!("{}.omp.json", theme.name));
        tokio::spawn(async move {
            match reqwest::get(&theme.download_url).await {
                Ok(resp) => {
                    if let Ok(bytes) = resp.bytes().await {
                        if let Err(e) = fs::write(&path, &bytes) {
                            let _ = tx.send(AppMessage::Error(format!("Save failed: {}", e))).await;
                        } else {
                            // After download, tell the app to refresh local themes
                            let _ = tx.send(AppMessage::ThemesLoaded(vec![ThemeAsset {
                                name: format!("{}.omp.json", theme.name),
                                is_local: true,
                                download_url: None,
                            }])).await;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("Download failed: {}", e))).await;
                }
            }
        });
    }


    /// Returns a filtered list of fonts based on search criteria
    pub fn filtered_fonts(&self) -> Vec<FontAsset> {
        self.fonts
            .iter()
            .filter(|f| contains_ignore_ascii_case(&f.name, &self.fonts_filter))
            .cloned()
            .collect()
    }

    /// Returns a filtered list of segments based on search criteria
    pub fn filtered_segments(&self) -> Vec<SegmentAsset> {
        self.segments
            .iter()
            .filter(|p| {
                contains_ignore_ascii_case(&p.name, &self.segments_filter)
                    || contains_ignore_ascii_case(&p.description, &self.segments_filter)
                    || contains_ignore_ascii_case(&p.category, &self.segments_filter)
            })
            .cloned()
            .collect()
    }

    /// Returns a filtered list of legacy plugins based on search criteria
    pub fn filtered_plugins(&self) -> Vec<PluginAsset> {
        self.plugins
            .iter()
            .filter(|p| contains_ignore_ascii_case(&p.name, &self.plugins_filter))
            .cloned()
            .collect()
    }

    /// Checks if a segment is active in the currently loaded Oh My Posh config
    pub fn is_segment_active(&self, segment: &SegmentAsset) -> bool {
        let path = if let Some(p) = &self.active_config_path {
            p
        } else {
            return false;
        };

        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // In modern OMP themes, segments are usually in blocks[].segments[]
                // We'll check top-level segments first, then look into blocks
                if let Some(segments) = json.get("segments").and_then(|v| v.as_array()) {
                    if segments.iter().any(|s| {
                        s.get("type").and_then(|v| v.as_str()) == Some(&segment.segment_type)
                    }) {
                        return true;
                    }
                }
                
                if let Some(blocks) = json.get("blocks").and_then(|v| v.as_array()) {
                    for block in blocks {
                        if let Some(segments) = block.get("segments").and_then(|v| v.as_array()) {
                            if segments.iter().any(|s| {
                                s.get("type").and_then(|v| v.as_str()) == Some(&segment.segment_type)
                            }) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Surgical JSON edit to toggle a segment in the active Oh My Posh theme
    pub fn toggle_segment(&mut self, segment: &SegmentAsset) -> io::Result<()> {
        let path = self.active_config_path.as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No active config found"))?;

        let content = fs::read_to_string(path)?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        // Logic to find and remove or add to the FIRST block found
        let mut toggled = false;
        
        if let Some(blocks) = json.get_mut("blocks").and_then(|v| v.as_array_mut()) {
            if let Some(first_block) = blocks.first_mut() {
                if let Some(segments) = first_block.get_mut("segments").and_then(|v| v.as_array_mut()) {
                    let pos = segments.iter().position(|s| {
                        s.get("type").and_then(|v| v.as_str()) == Some(&segment.segment_type)
                    });

                    if let Some(i) = pos {
                        segments.remove(i);
                        toggled = true;
                    } else {
                        // Add new segment
                        let new_segment = serde_json::json!({
                            "type": segment.segment_type,
                            "style": "powerline",
                            "powerline_symbol": "\u{e0b0}",
                            "foreground": "#ffffff",
                            "background": "#61afef",
                            "template": format!(" {} ", segment.segment_type)
                        });
                        segments.push(new_segment);
                        toggled = true;
                    }
                }
            }
        }

        if toggled {
            let new_json = serde_json::to_string_pretty(&json)
                .map_err(|e| io::Error::other(e.to_string()))?;
            fs::write(path, new_json)?;
            Ok(())
        } else {
            Err(io::Error::other("Could not find a valid segments block to edit"))
        }
    }


    /// Actualiza quirúrgicamente una sección del perfil de PowerShell envuelta en marcadores de PoshBuddy
    pub fn update_profile_safe(&self, profile: &std::path::Path, key: &str, content: &str, description: &str) -> io::Result<()> {
        let start_marker = format!("# <PoshBuddy: START - {}>", key);
        let end_marker = format!("# <PoshBuddy: END - {}>", key);
        let mut new_lines = Vec::new();

        let existing_content = if profile.exists() {
            fs::read_to_string(profile)?
        } else {
            String::new()
        };

        let mut inside_block = false;
        let mut found = false;

        for line in existing_content.lines() {
            if line.contains(&start_marker) {
                inside_block = true;
                found = true;
                new_lines.push(start_marker.clone());
                new_lines.push(format!("# {}", description));
                new_lines.push(content.to_string());
            } else if line.contains(&end_marker) {
                inside_block = false;
                new_lines.push(end_marker.clone());
            } else if !inside_block {
                new_lines.push(line.to_string());
            }
        }

        if !found {
            if !existing_content.is_empty() && !existing_content.ends_with('\n') {
                new_lines.push(String::new());
            }
            new_lines.push(start_marker);
            new_lines.push(format!("# {}", description));
            new_lines.push(content.to_string());
            new_lines.push(end_marker);
        }

        let line_ending = if cfg!(windows) { "\r\n" } else { "\n" };
        fs::write(profile, new_lines.join(line_ending))?;
        Ok(())
    }

    /// Triggers an asynchronous font installation via Oh My Posh CLI
    pub fn install_font(&self, font_name: String, tx: mpsc::Sender<AppMessage>) {
        let cmd = OMP_BINARY;

        let font_name_cloned = font_name.clone();
        tokio::spawn(async move {
            let output = tokio::process::Command::new(cmd)
                .args(["font", "install", &font_name_cloned])
                .output()
                .await;

            match output {
                Ok(_) => {
                    let _ = tx.send(AppMessage::FontInstalled(font_name_cloned)).await;
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(e.to_string())).await;
                }
            }
        });
    }

    /// Asynchronously generates a real prompt preview for a theme using isolation
    pub fn load_theme_preview(&self, theme: ThemeAsset, tx: mpsc::Sender<AppMessage>) {
        if !theme.is_local {
            let _ = tx.send(AppMessage::ThemePreviewLoaded {
                theme,
                preview: " [Remote Theme: Pulsa ENTER para descargar y previsualizar] ".to_string(),
            });
            return;
        }

        let cmd = OMP_BINARY;
        let theme_name = theme.name.clone();
        let theme_path = self.themes_dir.join(format!("{}.omp.json", theme_name));
        let theme_cloned = theme.clone();

        tokio::spawn(async move {
            let mut cmd_obj = tokio::process::Command::new(cmd);
            cmd_obj
                .env_clear()
                .env("PATH", std::env::var("PATH").unwrap_or_default())
                .env("USERPROFILE", std::env::var("USERPROFILE").unwrap_or_default())
                .env("SYSTEMROOT", std::env::var("SYSTEMROOT").unwrap_or_default())
                .env("SystemDrive", std::env::var("SystemDrive").unwrap_or_default())
                .env("TEMP", std::env::var("TEMP").unwrap_or_default())
                .args([
                    "print",
                    "primary",
                    "--config",
                    &theme_path.to_string_lossy(),
                    "--shell",
                    "pwsh",
                    "--force",
                    "--status",
                    "0",
                ]);

            let output = cmd_obj.output().await;

            match output {
                Ok(out) => {
                    let raw = String::from_utf8_lossy(&out.stdout).to_string();
                    let preview = format!(" {}", raw.trim_end());
                    let _ = tx.send(AppMessage::ThemePreviewLoaded {
                        theme: theme_cloned,
                        preview,
                    }).await;
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::ThemePreviewLoaded {
                        theme: theme_cloned,
                        preview: format!(" Error: {}", e),
                    }).await;
                }
            }
        });
    }

    /// Handles automatic installation of Oh My Posh via WinGet (Windows) or Homebrew (Linux/macOS)
    pub fn install_omp(&self, tx: mpsc::Sender<AppMessage>) {
        tokio::spawn(async move {
            let _ = tx
                .send(AppMessage::InstallProgress {
                    line: "Starting Oh My Posh installation...".to_string(),
                })
                .await;

            let child = if cfg!(windows) {
                tokio::process::Command::new("winget")
                    .args([
                        "install",
                        "JanDeDobbeleer.OhMyPosh",
                        "--accept-package-agreements",
                        "--accept-source-agreements",
                    ])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
            } else {
                tokio::process::Command::new("brew")
                    .args(["install", "oh-my-posh"])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
            }
            .map_err(|e| e.to_string());

            match child {
                Ok(mut child) => {
                    match child.wait().await {
                        Ok(status) if status.success() => {
                            let _ = tx.send(AppMessage::InstallFinished).await;
                        }
                        _ => {
                            let _ = tx
                                .send(AppMessage::Error(
                                    "Installation failed via Winget".to_string(),
                                ))
                                .await;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(AppMessage::Error(format!(
                            "Could not start installer: {}",
                            e
                        )))
                        .await;
                }
            }
        });
    }

    /// Precise matching for plugin lines to avoid partial matches
    fn is_plugin_line(line: &str, plugin: &PluginAsset) -> bool {
        let trimmed = line.trim();
        if let Some(init) = &plugin.init_script {
            // Check for exact match of init script lines
            init.lines().any(|init_line| trimmed == init_line.trim())
        } else {
            // Check for exact "Import-Module <module_name>" pattern
            let module_pattern = format!("Import-Module {}", plugin.module_name);
            trimmed == module_pattern
                || trimmed.starts_with(&format!("Import-Module {} -", plugin.module_name))
        }
    }

    /// Checks if a plugin is currently 'activated' (imported) in at least one PowerShell profile
    pub fn is_plugin_active(&self, plugin: &PluginAsset) -> bool {
        for profile in &self.detected_profiles {
            if !profile.exists() {
                continue;
            }
            if let Ok(content) = fs::read_to_string(profile) {
                if content
                    .lines()
                    .any(|line| Self::is_plugin_line(line, plugin))
                {
                    return true;
                }
            }
        }
        false
    }

    /// Elimina un bloque gestionado por PoshBuddy del perfil
    pub fn remove_profile_block(&self, profile: &std::path::Path, key: &str) -> io::Result<()> {
        if !profile.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(profile)?;
        let start_marker = format!("# <PoshBuddy: START - {}>", key);
        let end_marker = format!("# <PoshBuddy: END - {}>", key);

        let mut new_lines = Vec::new();
        let mut inside_block = false;
        let mut found = false;

        for line in content.lines() {
            if line.contains(&start_marker) {
                inside_block = true;
                found = true;
                continue;
            }
            if line.contains(&end_marker) {
                inside_block = false;
                continue;
            }
            if !inside_block {
                new_lines.push(line.to_string());
            }
        }

        if found {
            fs::write(profile, new_lines.join("\n"))?;
        }
        Ok(())
    }

    /// Atomically updates all detected shell profiles to initialize Oh My Posh with the selected theme
    pub fn apply_theme(&mut self, theme_name: &str) -> io::Result<()> {
        let theme_path = self.themes_dir.join(theme_name);
        
        let config_line = format!(
            "oh-my-posh init powershell --config \"{}\" | Out-String | Invoke-Expression",
            theme_path.display()
        );

        for profile in &self.detected_profiles {
            self.backup_manager.backup_profile(profile, &format!("Apply Theme: {}", theme_name))
                .map_err(|e| io::Error::other(e.to_string()))?;

            self.update_profile_safe(
                profile, 
                "THEME", 
                &config_line, 
                &format!("Aplica el tema de Oh My Posh: {}", theme_name)
            )?;
        }

        // Dejar rastro del tema actual para la UI
        self.active_config_path = Some(theme_path);
        
        Ok(())
    }

    /// Toggles the activation state of a plugin by adding or removing it from all detected profiles
    pub fn toggle_plugin(&mut self, plugin: &PluginAsset) -> io::Result<()> {
        let is_active = self.is_plugin_active(plugin);
        let key = format!("PLUGIN_{}", plugin.module_name.to_uppercase());

        for profile in &self.detected_profiles {
            self.backup_manager.backup_profile(profile, &format!("Toggle Plugin: {}", plugin.name))
                .map_err(|e| io::Error::other(e.to_string()))?;

            if is_active {
                self.remove_profile_block(profile, &key)?;
            } else {
                let payload = if let Some(init) = &plugin.init_script {
                    init.clone()
                } else {
                    format!("Import-Module {} -ErrorAction SilentlyContinue", plugin.module_name)
                };

                self.update_profile_safe(
                    profile, 
                    &key, 
                    &payload, 
                    &format!("Plugin: {} - {}", plugin.name, plugin.description)
                )?;
            }
        }

        Ok(())
    }

    /// Asynchronously installs a PowerShell module via the system shell
    #[allow(dead_code)]
    pub fn install_plugin(&self, name: String, module_name: String, tx: mpsc::Sender<AppMessage>) {
        tokio::spawn(async move {
            let _ = tx
                .send(AppMessage::InstallProgress {
                    line: format!("Installing module: {}...", name),
                })
                .await;

            let output = tokio::process::Command::new("powershell")
                .args([
                    "-Command",
                    &format!(
                        "Install-Module -Name {} -Scope CurrentUser -Force -Confirm:$false",
                        module_name
                    ),
                ])
                .output()
                .await;

            match output {
                Ok(out) if out.status.success() => {
                    let _ = tx.send(AppMessage::PluginInstalled(name)).await;
                }
                _ => {
                    let _ = tx
                        .send(AppMessage::Error(format!(
                            "Failed to install module {}",
                            module_name
                        )))
                        .await;
                }
            }
        });
    }

    pub fn handle_messages(
        &mut self,
        rx: &mut tokio::sync::mpsc::Receiver<AppMessage>,
        tx: tokio::sync::mpsc::Sender<AppMessage>,
    ) {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                AppMessage::ThemesLoaded(new_themes) => {
                    for t in new_themes {
                        if !self.themes.iter().any(|existing| existing.name == t.name) {
                            self.themes.push(t);
                        }
                    }
                    self.themes.sort_by(|a, b| a.name.cmp(&b.name));
                    if self.state == AppState::Loading {
                        self.state = AppState::Main;
                    }
                    if let Some(t) = self.themes.first() {
                        self.load_theme_preview(t.clone(), tx.clone());
                    }
                }
                AppMessage::FontsLoaded(mut fonts) => {
                    fonts.sort_by(|a, b| a.name.cmp(&b.name));
                    self.fonts = fonts;
                }
                AppMessage::ThemePreviewLoaded { theme, preview } => {
                    let filtered = self.filtered_themes();
                    if let Some(selected_index) = self.list_state.selected() {
                        if let Some(current_theme) = filtered.get(selected_index) {
                            if current_theme.name == theme.name {
                                self.theme_preview = preview;
                            }
                        }
                    }
                }
                AppMessage::RemoteThemesLoaded(themes) => {
                    self.remote_themes = themes;
                }
                AppMessage::FontInstalled(name) => {
                    self.state = AppState::FontSuccess(name);
                    self.has_nerd_font = true;
                }
                AppMessage::PluginInstalled(name) => {
                    self.state = AppState::PluginSuccess(name);
                }
                AppMessage::InstallProgress { line } => {
                    if let AppState::InstallingDependency { log, .. } = &mut self.state {
                        log.push(line.clone());
                        if log.len() > 100 {
                            log.remove(0);
                        }
                        self.state = AppState::InstallingDependency {
                            current_action: line,
                            log: log.clone(),
                        };
                    }
                }
                AppMessage::InstallFinished => {
                    self.state = AppState::Loading;
                    let themes_dir = self.themes_dir.clone();
                    tokio::spawn(crate::api::setup_app_task(tx.clone(), themes_dir));
                }
                AppMessage::Error(e) => {
                    self.state = AppState::Error(e);
                }
            }
        }
    }

    pub fn handle_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        tx: tokio::sync::mpsc::Sender<AppMessage>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};

        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        // Global shortcut: Ctrl+R to restore last backup
        if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
            match self.restore_last_backup() {
                Ok(count) => {
                    if count > 0 {
                        self.state =
                            AppState::Success(format!("Backup restaurado ({} perfiles)", count));
                    } else {
                        self.state = AppState::Error("No hay backups disponibles".to_string());
                    }
                }
                Err(e) => {
                    self.state = AppState::Error(format!("Error al restaurar: {}", e));
                }
            }
            return Ok(false);
        }

        if self.state == AppState::DependencyMissing {
            if key.code == KeyCode::Enter {
                self.install_omp(tx.clone());
            }
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                return Ok(true);
            }
            return Ok(false);
        }

        if let AppState::Onboarding(_) = self.state {
            match key.code {
                KeyCode::Enter => {
                    if self.themes.is_empty() {
                        self.state = AppState::Loading;
                    } else {
                        self.state = AppState::Main;
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                _ => {}
            }
            return Ok(false);
        }

        if let AppState::Success(_) = self.state {
            return Ok(true);
        }

        if let AppState::FontSuccess(_) = self.state {
            self.state = AppState::Main;
            return Ok(false);
        }

        if let AppState::PluginSuccess(_) = self.state {
            self.state = AppState::Main;
            self.active_view = ActiveView::Segments;
            return Ok(false);
        }

        // Welcome screen - Quick actions navigation
        if self.state == AppState::Welcome {
            const NUM_ACTIONS: usize = 8;

            match key.code {
                KeyCode::Up => {
                    if self.welcome_selected_action > 0 {
                        self.welcome_selected_action -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.welcome_selected_action < NUM_ACTIONS - 1 {
                        self.welcome_selected_action += 1;
                    }
                }
                KeyCode::Enter => {
                    // Ejecutar acción seleccionada
                    match self.welcome_selected_action {
                        0 => {
                            // Aplicar tema aleatorio
                            if !self.themes.is_empty() {
                                use std::time::{SystemTime, UNIX_EPOCH};
                                let seed = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as usize;
                                let idx = seed % self.themes.len();
                                if let Some(random_theme) = self.themes.get(idx) {
                                    let name = random_theme.name.clone();
                                    if let Err(e) = self.apply_theme(&name) {
                                        self.state =
                                            AppState::Error(format!("Failed to apply fallback: {}", e));
                                    } else {
                                        self.state = AppState::Success(format!(
                                            "Tema '{}' aplicado!",
                                            name
                                        ));
                                    }
                                }
                            } else {
                                self.state =
                                    AppState::Error("No hay temas disponibles".to_string());
                            }
                        }
                        1 => {
                            // Instalar Nerd Font (Acción real)
                            let font_name = "CascadiaCode".to_string();
                            self.state = AppState::Installing(font_name.clone());
                            self.install_font(font_name, tx.clone());
                        }
                        2 => {
                            // Instalar Terminal-Icons (Acción real)
                            let plugin = self.plugins.iter().find(|p| p.name == "Terminal-Icons").cloned();
                            if let Some(p) = plugin {
                                if let Err(e) = self.toggle_plugin(&p) {
                                    self.state = AppState::Error(format!("Failed to install plugin: {}", e));
                                } else {
                                    self.state = AppState::PluginSuccess(p.name);
                                }
                            }
                        }
                        3 => {
                            // Ver diagnóstico
                            match self.diagnostic.run_full_diagnostic(&self.detected_profiles) {
                                Ok(result) => {
                                    let report = self.diagnostic.format_report(&result);
                                    self.state = AppState::Success(report);
                                }
                                Err(e) => {
                                    self.state =
                                        AppState::Error(format!("Diagnóstico falló: {}", e));
                                }
                            }
                        }
                        4 => {
                            // Mostrar info de backups disponibles
                            if let Some(ref last) = self.last_backup {
                                self.state = AppState::Success(format!(
                                    "Backup disponible: {}",
                                    last.display()
                                ));
                            } else {
                                self.state =
                                    AppState::Error("No hay backups disponibles".to_string());
                            }
                        }
                        5 => {
                            // Ir a temas
                            self.state = AppState::Main;
                            self.active_view = ActiveView::Themes;
                        }
                        6 => {
                            // Ir a fuentes
                            self.state = AppState::Main;
                            self.active_view = ActiveView::Fonts;
                        }
                        7 => {
                            // Ir a plugins
                            self.state = AppState::Main;
                            self.active_view = ActiveView::Segments;
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('1') => {
                    // Accion 1: Tema Aleatorio (Se requiere seleccion en Enter)
                    self.welcome_selected_action = 0;
                }
                KeyCode::Char('2') | KeyCode::Char('f') => {
                    // Accion 2: Instalar Nerd Font
                    let font_name = "CascadiaCode".to_string();
                    self.state = AppState::Installing(font_name.clone());
                    self.install_font(font_name, tx.clone());
                }
                KeyCode::Char('3') | KeyCode::Char('i') => {
                    // Accion 3: Terminal-Icons
                    let plugin = self.plugins.iter().find(|p| p.name == "Terminal-Icons").cloned();
                    if let Some(p) = plugin {
                        if let Err(e) = self.toggle_plugin(&p) {
                            self.state = AppState::Error(format!("Error: {}", e));
                        } else {
                            self.state = AppState::PluginSuccess(p.name);
                        }
                    }
                }
                KeyCode::Char('4') | KeyCode::Char('d') => {
                    // Accion 4: Diagnostico
                    match self.diagnostic.run_full_diagnostic(&self.detected_profiles) {
                        Ok(result) => {
                            let report = self.diagnostic.format_report(&result);
                            self.state = AppState::Success(report);
                        }
                        Err(e) => {
                            self.state = AppState::Error(format!("Error: {}", e));
                        }
                    }
                }
                KeyCode::Char('5') | KeyCode::Char('b') => {
                    // Accion 5: Backups
                    if let Some(ref last) = self.last_backup {
                        self.state = AppState::Success(format!("Backup: {}", last.display()));
                    } else {
                        self.state = AppState::Error("No backups found".to_string());
                    }
                }
                KeyCode::Char('6') | KeyCode::Char('t') => {
                    self.state = AppState::Main;
                    self.active_view = ActiveView::Themes;
                }
                KeyCode::Char('7') | KeyCode::Char('F') => {
                    self.state = AppState::Main;
                    self.active_view = ActiveView::Fonts;
                }
                KeyCode::Char('8') | KeyCode::Char('p') | KeyCode::Char('s') => {
                    self.state = AppState::Main;
                    self.active_view = ActiveView::Segments;
                }
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                _ => {}
            }
            return Ok(false);
        }

        if self.state == AppState::Main {
            match key.code {
                KeyCode::Tab => {
                    self.active_view = match self.active_view {
                        ActiveView::Themes => ActiveView::Fonts,
                        ActiveView::Fonts => ActiveView::Segments,
                        ActiveView::Segments => ActiveView::Themes,
                    };
                }
                KeyCode::Char('1') => {
                    self.active_view = ActiveView::Themes;
                }
                KeyCode::Char('2') => {
                    self.active_view = ActiveView::Fonts;
                }
                KeyCode::Char('3') => {
                    self.active_view = ActiveView::Segments;
                }
                KeyCode::Down | KeyCode::Up => {
                    if self.active_view == ActiveView::Themes {
                        let filtered = self.filtered_themes();
                        let i = match self.list_state.selected() {
                            Some(i) => {
                                if key.code == KeyCode::Down {
                                    if i >= filtered.len().saturating_sub(1) {
                                        0
                                    } else {
                                        i + 1
                                    }
                                } else {
                                    if i == 0 {
                                        filtered.len().saturating_sub(1)
                                    } else {
                                        i - 1
                                    }
                                }
                            }
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                        self.theme_preview = String::new();
                        if let Some(t) = filtered.get(i) {
                            self.load_theme_preview(t.clone(), tx.clone());
                        }
                    } else if self.active_view == ActiveView::Fonts {
                        let filtered = self.filtered_fonts();
                        let i = match self.fonts_list_state.selected() {
                            Some(i) => {
                                if key.code == KeyCode::Down {
                                    if i >= filtered.len().saturating_sub(1) {
                                        0
                                    } else {
                                        i + 1
                                    }
                                } else {
                                    if i == 0 {
                                        filtered.len().saturating_sub(1)
                                    } else {
                                        i - 1
                                    }
                                }
                            }
                            None => 0,
                        };
                        self.fonts_list_state.select(Some(i));
                    } else if self.active_view == ActiveView::Segments {
                        let filtered = self.filtered_segments();
                        let i = match self.plugins_list_state.selected() {
                            Some(i) => {
                                if key.code == KeyCode::Down {
                                    if i >= filtered.len().saturating_sub(1) {
                                        0
                                    } else {
                                        i + 1
                                    }
                                } else {
                                    if i == 0 {
                                        filtered.len().saturating_sub(1)
                                    } else {
                                        i - 1
                                    }
                                }
                            }
                            None => 0,
                        };
                        self.plugins_list_state.select(Some(i));
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                KeyCode::Char(c) => {
                    if self.active_view == ActiveView::Themes {
                        self.filter.push(c);
                        self.list_state.select(Some(0));
                    } else if self.active_view == ActiveView::Fonts {
                        self.fonts_filter.push(c);
                        self.fonts_list_state.select(Some(0));
                    } else if self.active_view == ActiveView::Segments {
                        self.segments_filter.push(c);
                        self.plugins_list_state.select(Some(0));
                    }
                }
                KeyCode::Backspace => {
                    if self.active_view == ActiveView::Themes {
                        self.filter.pop();
                    } else if self.active_view == ActiveView::Fonts {
                        self.fonts_filter.pop();
                    } else if self.active_view == ActiveView::Segments {
                        self.segments_filter.pop();
                    }
                }
                KeyCode::Enter => {
                    if self.active_view == ActiveView::Themes {
                        let filtered = self.filtered_themes();
                        if let Some(selected) = self.list_state.selected() {
                            if let Some(theme) = filtered.get(selected) {
                                if theme.is_local {
                                    self.apply_theme(&theme.name)?;
                                    self.state = AppState::Success(format!("Tema '{}' aplicado!", theme.name));
                                } else if let Some(url) = &theme.download_url {
                                    // Trigger download
                                    self.state = AppState::Installing(format!("Descargando {}...", theme.name));
                                    self.download_theme(RemoteTheme {
                                        name: theme.name.clone(),
                                        download_url: url.clone(),
                                        sha: String::new(), // Not critical for simple download
                                    }, tx.clone());
                                }
                            }
                        }
                    } else if self.active_view == ActiveView::Fonts {
                        let filtered = self.filtered_fonts();
                        if let Some(selected) = self.fonts_list_state.selected() {
                            if let Some(font) = filtered.get(selected) {
                                self.state = AppState::Installing(font.name.clone());
                                self.install_font(font.name.clone(), tx.clone());
                            }
                        }
                    } else if self.active_view == ActiveView::Segments {
                        let filtered = self.filtered_segments();
                        if let Some(selected) = self.plugins_list_state.selected() {
                            if let Some(segment) = filtered.get(selected) {
                                if let Err(e) = self.toggle_segment(segment) {
                                    self.state =
                                        AppState::Error(format!("Failed to toggle segment: {}", e));
                                } else {
                                    self.state = AppState::PluginSuccess(segment.name.clone());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(false)
    }

    /// Restaura el último backup creado para todos los perfiles detectados
    /// Retorna el número de perfiles restaurados exitosamente
    pub fn restore_last_backup(&mut self) -> Result<usize, crate::backup::BackupError> {
        let mut restored_count = 0;

        for profile in &self.detected_profiles {
            if profile.exists() {
                match self.backup_manager.restore_latest(profile) {
                    Ok(_) => {
                        restored_count += 1;
                    }
                    Err(crate::backup::BackupError::BackupNotFound(_)) => {
                        // No hay backup para este perfil, ignorar
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(restored_count)
    }

    /// Obtiene información de los backups disponibles para un perfil específico
    pub fn get_backup_info(
        &self,
        profile_path: &std::path::Path,
    ) -> Vec<crate::backup::BackupInfo> {
        self.backup_manager
            .list_backups(profile_path)
            .unwrap_or_default()
    }

    /// Actualiza el contador total de backups detectados en todos los perfiles
    pub fn refresh_backup_count(&mut self) {
        let mut count = 0;
        for profile in &self.detected_profiles {
            count += self.get_backup_info(profile).len();
        }
        self.total_backups = count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::ListState;
    use std::path::PathBuf;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        wt_session: Option<String>,
        term_program: Option<String>,
        path: Option<String>,
    }

    impl EnvGuard {
        fn new() -> Self {
            Self {
                wt_session: env::var("WT_SESSION").ok(),
                term_program: env::var("TERM_PROGRAM").ok(),
                path: env::var("PATH").ok(),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(ref v) = self.wt_session {
                env::set_var("WT_SESSION", v);
            } else {
                env::remove_var("WT_SESSION");
            }
            if let Some(ref v) = self.term_program {
                env::set_var("TERM_PROGRAM", v);
            } else {
                env::remove_var("TERM_PROGRAM");
            }
            if let Some(ref v) = self.path {
                env::set_var("PATH", v);
            } else {
                env::remove_var("PATH");
            }
        }
    }

    fn mock_app() -> App {
        App {
            state: AppState::Loading,
            active_view: ActiveView::Themes,
            themes: Vec::new(),
            remote_themes: Vec::new(),
            fonts: Vec::new(),
            filter: String::new(),
            fonts_filter: String::new(),
            themes_dir: PathBuf::from("/tmp"),
            version: "test".to_string(),
            list_state: ListState::default(),
            fonts_list_state: ListState::default(),
            plugins_list_state: ListState::default(),
            plugins: Vec::new(),
            segments: Vec::new(),
            plugins_filter: String::new(),
            segments_filter: String::new(),
            spinner_tick: 0,
            has_nerd_font: false,
            theme_preview: String::new(),
            detected_profiles: Vec::new(),
            active_config_path: None,
            backup_manager: crate::backup::BackupManager::new(Some(10)),
            last_backup: None,
            diagnostic: crate::diagnostic::Diagnostic::new(),
            plugin_installer: crate::plugin_installer::PluginInstaller::new(),
            welcome_selected_action: 0,
            system_specs: None,
            total_backups: 0,
        }
    }

    #[test]
    fn test_filtered_themes() {
        let mut app = mock_app();
        app.themes = vec![
            ThemeAsset { name: "bubbles.omp.json".to_string(), is_local: true, download_url: None },
            ThemeAsset { name: "joker.omp.json".to_string(), is_local: true, download_url: None },
            ThemeAsset { name: "M365.omp.json".to_string(), is_local: true, download_url: None },
        ];

        // Empty filter should return all
        assert_eq!(app.filtered_themes().len(), 3);

        // Case-insensitive matching
        app.filter = "JOKER".to_string();
        assert_eq!(app.filtered_themes()[0].name, "joker.omp.json");

        // Partial matching
        app.filter = "omp".to_string();
        assert_eq!(app.filtered_themes().len(), 3);

        // No match
        app.filter = "nonexistent".to_string();
        assert_eq!(app.filtered_themes().len(), 0);
    }

    #[test]
    fn test_filtered_fonts() {
        let mut app = mock_app();
        app.fonts = vec![
            FontAsset {
                name: "CascaidaCode".to_string(),
                download_url: "https://example.com/cascadia".to_string(),
            },
            FontAsset {
                name: "FiraCode".to_string(),
                download_url: "https://example.com/fira".to_string(),
            },
            FontAsset {
                name: "JetBrainsMono".to_string(),
                download_url: "https://example.com/jetbrains".to_string(),
            },
        ];

        // Empty filter should return all
        assert_eq!(app.filtered_fonts().len(), 3);

        // Case-insensitive matching
        app.fonts_filter = "fira".to_string();
        assert_eq!(
            app.filtered_fonts(),
            vec![FontAsset {
                name: "FiraCode".to_string(),
                download_url: "https://example.com/fira".to_string(),
            }]
        );

        // Partial matching
        app.fonts_filter = "Code".to_string();
        assert_eq!(app.filtered_fonts().len(), 2);

        // No match
        app.fonts_filter = "Wingdings".to_string();
        assert_eq!(app.filtered_fonts().len(), 0);
    }

    #[test]
    #[cfg_attr(windows, ignore = "Windows mocking requires .exe files")]
    fn test_detect_profiles() {
        // Use catch_unwind to prevent mutex poisoning if this test fails
        let result = std::panic::catch_unwind(|| {
            let _lock = ENV_LOCK.lock().unwrap();
            let _guard = EnvGuard::new();

            let original_path = env::var("PATH").unwrap_or_default();
            let dir = env::temp_dir().join("fake_detect_profiles_bin");
            std::fs::create_dir_all(&dir).unwrap();

            let pwsh_name = if cfg!(windows) { "pwsh.cmd" } else { "pwsh" };
            let pwsh_path = dir.join(pwsh_name);

            // Create mock pwsh that outputs the expected profile path
            // In Windows, we use a batch file that ignores PowerShell-specific arguments
            let content = if cfg!(windows) {
                "@echo off\nREM Mock pwsh - ignores args and outputs profile path\necho C:\\mock\\path\\profile.ps1"
            } else {
                "#!/bin/sh\n# Mock pwsh - ignores args and outputs profile path\necho -n '/mock/path/profile.ps1'"
            };

            std::fs::write(&pwsh_path, content).unwrap();

            if cfg!(windows) {
                let powershell_path = dir.join("powershell.cmd");
                std::fs::write(
                    &powershell_path,
                    "@echo off\nREM Mock powershell\necho C:\\mock\\path\\powershell_profile.ps1",
                )
                .unwrap();
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&pwsh_path, std::fs::Permissions::from_mode(0o755))
                    .unwrap();
            }

            // Use ONLY the mock directory in PATH - this ensures Windows finds our .cmd mocks
            // instead of any real pwsh.exe/powershell.exe that might be installed
            env::set_var("PATH", &dir);

            let profiles = App::detect_profiles();

            // Restore original PATH for cleanup (EnvGuard will also restore it)
            env::set_var("PATH", &original_path);

            assert!(!profiles.is_empty(), "Profiles should not be empty");

            // On Windows, the mock outputs Windows paths; on Unix, Unix paths
            let expected_pwsh_profile = if cfg!(windows) {
                PathBuf::from("C:\\mock\\path\\profile.ps1")
            } else {
                PathBuf::from("/mock/path/profile.ps1")
            };

            assert!(
                profiles.contains(&expected_pwsh_profile),
                "Should contain mocked pwsh profile: {:?}. Got: {:?}",
                expected_pwsh_profile,
                profiles
            );

            if cfg!(windows) {
                assert!(
                    profiles.contains(&PathBuf::from("C:\\mock\\path\\powershell_profile.ps1")),
                    "Should contain mocked powershell profile"
                );
            }
        });

        // Re-panic if the test failed, but after releasing the mutex guard
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    #[cfg_attr(windows, ignore = "Windows mocking requires .exe files")]
    fn test_detect_profiles_empty_output() {
        let result = std::panic::catch_unwind(|| {
            let _lock = ENV_LOCK.lock().unwrap();
            let _guard = EnvGuard::new();

            let original_path = env::var("PATH").unwrap_or_default();
            let dir = env::temp_dir().join("fake_detect_profiles_empty_bin");
            std::fs::create_dir_all(&dir).unwrap();

            let pwsh_name = if cfg!(windows) { "pwsh.cmd" } else { "pwsh" };
            let pwsh_path = dir.join(pwsh_name);

            let content = if cfg!(windows) {
                "@echo off"
            } else {
                "#!/bin/sh\nexit 0"
            };

            std::fs::write(&pwsh_path, content).unwrap();

            if cfg!(windows) {
                let powershell_path = dir.join("powershell.cmd");
                std::fs::write(&powershell_path, "@echo off").unwrap();
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&pwsh_path, std::fs::Permissions::from_mode(0o755))
                    .unwrap();
            }

            // Use ONLY the mock directory in PATH
            env::set_var("PATH", &dir);

            let profiles = App::detect_profiles();

            // Restore original PATH
            env::set_var("PATH", &original_path);

            assert!(
                profiles.is_empty(),
                "Profiles should be empty when output is blank"
            );
        });

        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    #[cfg_attr(windows, ignore = "Windows mocking requires .exe files")]
    fn test_gather_system_specs() {
        let result = std::panic::catch_unwind(|| {
            let _lock = ENV_LOCK.lock().unwrap();
            let _guard = EnvGuard::new();

            // Ensure we save original path so 'which' or 'where.exe' still work
            let original_path = env::var("PATH").unwrap_or_default();

            // Scenario 1: Default behavior (no WT_SESSION, no vscode, mock pwsh absent)
            env::remove_var("WT_SESSION");
            env::remove_var("TERM_PROGRAM");

            // Ensure pwsh is not in PATH. But we need 'which' to work, so we keep original path but make sure there's no pwsh in it.
            // For testing purposes, we can just let 'pwsh' command fail naturally on CI if it's not installed,
            // but if it is installed, it might be true. Wait, we want to deterministically test both false and true.
            // Actually, if we prepend a fake empty directory to PATH, it won't override a real 'pwsh' if it exists.
            // So Scenario 1 might not be reliably tested to be 'false' if the system actually has pwsh installed.
            // However, we can test Scenario 1 assuming pwsh isn't there, OR we skip asserting is_pwsh_7 in Scenario 1 and focus on Scenario 4.

            // Let's create an isolated path that only contains the system binaries.
            // Since it's tricky, let's just test that without the fake pwsh, it matches what 'which pwsh' says normally,
            // but wait, if it's installed, it will be true. So let's just assert on terminal properties for Scenario 1.
            let specs = App::gather_system_specs(false);
            assert!(
                !specs.is_windows_terminal,
                "Expected is_windows_terminal to be false"
            );
            assert!(!specs.has_nerd_font, "Expected has_nerd_font to be false");

            // Scenario 2: WT_SESSION set
            env::set_var("WT_SESSION", "1");
            let specs = App::gather_system_specs(true);
            assert!(
                specs.is_windows_terminal,
                "Expected is_windows_terminal to be true when WT_SESSION is set"
            );
            assert!(specs.has_nerd_font, "Expected has_nerd_font to be true");

            // Scenario 3: TERM_PROGRAM=vscode set
            env::remove_var("WT_SESSION");
            env::set_var("TERM_PROGRAM", "vscode");
            let specs = App::gather_system_specs(false);
            assert!(
                specs.is_windows_terminal,
                "Expected is_windows_terminal to be true when TERM_PROGRAM is vscode"
            );

            // Scenario 4: Pwsh command available
            let dir = env::temp_dir().join("fake_pwsh_bin");
            std::fs::create_dir_all(&dir).unwrap();

            let pwsh_name = if cfg!(windows) { "pwsh.exe" } else { "pwsh" };
            let pwsh_path = dir.join(pwsh_name);

            std::fs::write(&pwsh_path, "#!/bin/sh\nexit 0").unwrap();

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&pwsh_path, std::fs::Permissions::from_mode(0o755))
                    .unwrap();
            }

            // The 'where' command might not exist on linux/unix systems.
            // On Unix systems, whereis or which is used, but app.rs hardcodes `where` via `WHERE_CMD`.
            // We need to provide a mock `where` script to pass the test on Unix.
            #[cfg(unix)]
            {
                let where_path = dir.join("where");
                std::fs::write(&where_path, format!("#!/bin/sh\nif [ \"$1\" = \"pwsh\" ]; then echo '{}'; exit 0; else exit 1; fi", pwsh_path.display())).unwrap();
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&where_path, std::fs::Permissions::from_mode(0o755)).unwrap();
            }

            // Use ONLY mock directory in PATH if we are mocking `where` as well
            #[cfg(unix)]
            env::set_var("PATH", &dir);
            #[cfg(windows)]
            env::set_var("PATH", format!("{};{}", dir.display(), original_path));

            let specs = App::gather_system_specs(false);

            // Restore original PATH
            env::set_var("PATH", &original_path);

            assert!(
                specs.is_pwsh_7,
                "Expected is_pwsh_7 to be true when pwsh is in PATH"
            );
        });

        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }
}

#[cfg(test)]
mod filtering_tests {
    use super::*;
    use ratatui::widgets::ListState;

    fn create_test_app() -> App {
        App {
            state: AppState::Main,
            active_view: ActiveView::Themes,
            themes: vec![
                ThemeAsset { name: "agnoster".to_string(), is_local: true, download_url: None },
                ThemeAsset { name: "amro".to_string(), is_local: true, download_url: None },
                ThemeAsset { name: "atomic".to_string(), is_local: true, download_url: None },
                ThemeAsset { name: "catppuccin_frappe".to_string(), is_local: true, download_url: None },
                ThemeAsset { name: "Catppuccin_Macchiato".to_string(), is_local: true, download_url: None },
                ThemeAsset { name: "cyberpunk".to_string(), is_local: true, download_url: None },
            ],
            remote_themes: vec![],
            fonts: vec![],
            filter: "".to_string(),
            fonts_filter: "".to_string(),
            themes_dir: PathBuf::from("/mock/themes/dir"),
            version: "1.0.0".to_string(),
            list_state: ListState::default(),
            fonts_list_state: ListState::default(),
            plugins_list_state: ListState::default(),
            plugins: vec![],
            segments: vec![],
            plugins_filter: "".to_string(),
            segments_filter: "".to_string(),
            spinner_tick: 0,
            has_nerd_font: true,
            theme_preview: "".to_string(),
            detected_profiles: vec![],
            active_config_path: None,
            backup_manager: crate::backup::BackupManager::new(Some(10)),
            last_backup: None,
            diagnostic: crate::diagnostic::Diagnostic::new(),
            plugin_installer: crate::plugin_installer::PluginInstaller::new(),
            welcome_selected_action: 0,
            system_specs: None,
            total_backups: 0,
        }
    }

    #[test]
    fn test_filtered_themes_empty_filter() {
        let app = create_test_app();
        let filtered = app.filtered_themes();
        assert_eq!(filtered.len(), 6);
    }

    #[test]
    fn test_filtered_themes_case_insensitive() {
        let mut app = create_test_app();
        app.filter = "cAtP".to_string();
        let filtered = app.filtered_themes();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|t| t.name == "catppuccin_frappe"));
        assert!(filtered.iter().any(|t| t.name == "Catppuccin_Macchiato"));
    }

    #[test]
    fn test_filtered_themes_partial_match() {
        let mut app = create_test_app();
        app.filter = "amro".to_string();
        let filtered = app.filtered_themes();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "amro");
    }

    #[test]
    fn test_filtered_themes_no_match() {
        let mut app = create_test_app();
        app.filter = "nonexistent".to_string();
        let filtered = app.filtered_themes();
        assert_eq!(filtered.len(), 0);
    }
}
