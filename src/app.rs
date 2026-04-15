use ratatui::widgets::ListState;
use std::collections::HashSet;
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    ThemesLoaded(Vec<ThemeAsset>),
    FontsLoaded(Vec<FontAsset>),
    ThemePreviewLoaded {
        theme: ThemeAsset,
        preview: String,
        request_id: u64,
    },
    ThemeDownloaded(std::path::PathBuf),
    InstallUpdate {
        stage: usize,
        percentage: f32,
    },
    MassFontProgress {
        index: usize,
        total: usize,
        name: String,
    },
    Success(String),
    InstallProgress {
        line: String,
    },
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
    #[allow(dead_code)]
    Onboarding(SystemSpecs),
    DependencyMissing,
    InstallingDependency {
        log: Vec<String>,
        current_action: String,
    },
    Success(String),
    FontSuccess(String),
    PluginSuccess(String),
    ConfirmMassFontInstallation,
    InstallingAllFonts {
        progress: f32,
        current_font: String,
        index: usize,
        total: usize,
    },
    Installing(String),
    Error(String),
    Welcome,
    ApplyingProgress {
        name: String,
        stage: usize,
        progress: f32,
    },
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
    #[allow(dead_code)]
    pub plugins_filter: String,
    pub segments_filter: String,
    pub spinner_tick: usize,
    pub has_nerd_font: bool,
    pub theme_preview: String,
    pub detected_profiles: Vec<PathBuf>,
    pub active_config_path: Option<PathBuf>,
    pub backup_manager: crate::backup::BackupManager,
    pub last_backup: Option<std::path::PathBuf>,
    #[allow(dead_code)]
    pub diagnostic: crate::diagnostic::Diagnostic,
    #[allow(dead_code)]
    pub plugin_installer: crate::plugin_installer::PluginInstaller,
    // Welcome screen state
    pub welcome_selected_action: usize, // Index of the selected quick action
    pub system_specs: Option<SystemSpecs>, // Cache for system specifications
    pub total_backups: usize,           // Total backed up profiles found
    pub preview_request_id: u64,        // ID to version and cancel obsolete previews
    pub active_preview_task: Option<tokio::task::JoinHandle<()>>, // Handle to abort preview tasks
    pub active_segments: HashSet<String>, // Cache of active segments to avoid repetitive I/O
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

        // Ensure themes directory exists
        if !themes_dir.exists() {
            let _ = fs::create_dir_all(&themes_dir);
        }

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut fonts_list_state = ListState::default();
        fonts_list_state.select(Some(0));

        let mut plugins_list_state = ListState::default();
        plugins_list_state.select(Some(0));

        // 1. Initial system diagnostics
        let has_nerd_font = Self::check_nerd_font();
        let detected_profiles = Self::detect_profiles();
        let specs = Self::gather_system_specs(has_nerd_font);

        // 2. Load existing local themes
        let mut local_themes = Vec::new();
        if let Ok(entries) = fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.replace(".omp.json", ""))
                        .unwrap_or_default();

                    if !name.is_empty() {
                        local_themes.push(ThemeAsset {
                            name,
                            is_local: true,
                            download_url: None,
                        });
                    }
                }
            }
        }
        local_themes.sort_by(|a, b| a.name.cmp(&b.name));

        let mut app = App {
            state: AppState::Welcome,
            active_view: ActiveView::Themes,
            themes: local_themes,
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
                    description: "Shows current branch and Git file status.".to_string(),
                    documentation: "Essential for collaborative development.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Path".to_string(),
                    segment_type: "path".to_string(),
                    description: "Shows current location in the file system.".to_string(),
                    documentation: "Configurable for full or short path display.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Session (User)".to_string(),
                    segment_type: "session".to_string(),
                    description: "Shows current user and host.".to_string(),
                    documentation: "Quickly identify your current account/machine.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Battery".to_string(),
                    segment_type: "battery".to_string(),
                    description: "Displays battery percentage and charging status.".to_string(),
                    documentation: "Changes color based on charge level.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Execution Time".to_string(),
                    segment_type: "executiontime".to_string(),
                    description: "Shows duration of the last command executed.".to_string(),
                    documentation: "Perfect for measuring script performance.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Node.js info".to_string(),
                    segment_type: "node".to_string(),
                    description: "Shows active Node version in the directory.".to_string(),
                    documentation: "Automatically activates in Node projects.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Docker".to_string(),
                    segment_type: "docker".to_string(),
                    description: "Shows current Docker status and context.".to_string(),
                    documentation: "Requires Docker to be installed and running.".to_string(),
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
            preview_request_id: 0,
            active_preview_task: None,
            active_segments: HashSet::new(),
        };

        // Initialize active config path and segments cache
        app.active_config_path = app.find_active_config_path();
        app.refresh_active_segments();

        // Initialize backup count
        app.refresh_backup_count();

        // 3. Pre-check for Oh My Posh installation
        if !app.check_omp_installed() {
            app.state = AppState::DependencyMissing;
        }

        app
    }

    /// Reads the current configuration file once and caches active segment types
    pub fn refresh_active_segments(&mut self) {
        let path = if let Some(p) = &self.active_config_path {
            p
        } else {
            self.active_segments.clear();
            return;
        };

        let mut active = HashSet::new();
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // Look in top-level segments
                if let Some(segments) = json.get("segments").and_then(|v| v.as_array()) {
                    for s in segments {
                        if let Some(t) = s.get("type").and_then(|v| v.as_str()) {
                            active.insert(t.to_string());
                        }
                    }
                }

                // Look in blocks
                if let Some(blocks) = json.get("blocks").and_then(|v| v.as_array()) {
                    for block in blocks {
                        if let Some(segments) = block.get("segments").and_then(|v| v.as_array()) {
                            for s in segments {
                                if let Some(t) = s.get("type").and_then(|v| v.as_str()) {
                                    active.insert(t.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        self.active_segments = active;
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
                            let config_path =
                                if path_part.starts_with('"') || path_part.starts_with('\'') {
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
            let pwsh5 = docs
                .join("WindowsPowerShell")
                .join("Microsoft.PowerShell_profile.ps1");
            let pwsh7 = docs
                .join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1");
            if pwsh5.exists() {
                profiles.push(pwsh5);
            }
            if pwsh7.exists() {
                profiles.push(pwsh7);
            }
        }

        // 2. If nothing found or to be sure, ask the shells (Lazy detection later would be better, but let's at least deduplicate)
        if profiles.is_empty() {
            let shells = if cfg!(windows) {
                vec!["powershell", "pwsh"]
            } else {
                vec!["pwsh"]
            };
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
        let themes_dir = self.themes_dir.clone();
        tokio::spawn(async move {
            crate::api::setup_app_task(tx, themes_dir).await;
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
        self.active_segments.contains(&segment.segment_type)
    }

    /// Surgical JSON edit to toggle a segment in the active Oh My Posh theme
    pub fn toggle_segment(&mut self, segment: &SegmentAsset) -> io::Result<()> {
        let path = self
            .active_config_path
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No active config found"))?;

        let content = fs::read_to_string(path)?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        // Logic to find and remove or add to the FIRST block found
        let mut toggled = false;

        if let Some(blocks) = json.get_mut("blocks").and_then(|v| v.as_array_mut()) {
            if let Some(first_block) = blocks.first_mut() {
                if let Some(segments) = first_block
                    .get_mut("segments")
                    .and_then(|v| v.as_array_mut())
                {
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
            let new_json =
                serde_json::to_string_pretty(&json).map_err(|e| io::Error::other(e.to_string()))?;
            fs::write(path, new_json)?;
            self.refresh_active_segments(); // Update cache after write
            Ok(())
        } else {
            Err(io::Error::other(
                "Could not find a valid segments block to edit",
            ))
        }
    }

    /// Actualiza quirúrgicamente una sección del perfil de PowerShell envuelta en marcadores de PoshBuddy
    pub fn update_profile_safe(
        &self,
        profile: &std::path::Path,
        key: &str,
        content: &str,
        description: &str,
    ) -> io::Result<()> {
        let legacy_start = "# <PoshBuddy: START";
        let legacy_end = "# <PoshBuddy: END";
        let start_marker = format!("## POSHBUDDY AUTO-GENERATED CONFIG - START ({})", key);
        let end_marker = format!("## POSHBUDDY AUTO-GENERATED CONFIG - END ({})", key);

        let mut new_lines = Vec::new();
        let existing_content = if profile.exists() {
            fs::read_to_string(profile)?
        } else {
            String::new()
        };

        let mut inside_block = false;
        let mut found = false;

        for line in existing_content.lines() {
            let trimmed = line.trim();

            // 1. Purge legacy markers (Clean Migration)
            if trimmed.starts_with(legacy_start) || trimmed.starts_with(legacy_end) {
                continue;
            }

            // 2. Handle new standard markers
            if trimmed.starts_with(&start_marker) {
                inside_block = true;
                found = true;
                new_lines.push(start_marker.clone());
                new_lines.push(format!("## Description: {}", description));
                new_lines.push(content.to_string());
            } else if trimmed.starts_with(&end_marker) {
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
            new_lines.push(format!("## Description: {}", description));
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
                    if tx.send(AppMessage::FontInstalled(font_name_cloned)).await.is_err() {}
                }
                Err(e) => {
                    if tx.send(AppMessage::Error(e.to_string())).await.is_err() {}
                }
            }
        });
    }

    /// Installs all available fonts sequentially with progress reporting
    pub fn install_all_fonts(&self, tx: mpsc::Sender<AppMessage>) {
        let fonts = self.fonts.clone();
        let cmd = OMP_BINARY;

        if fonts.is_empty() {
            let _ = tx.try_send(AppMessage::Error(
                "No fonts available to install.".to_string(),
            ));
            return;
        }

        tokio::spawn(async move {
            let total = fonts.len();
            for (idx, font) in fonts.iter().enumerate() {
                let current_name = font.name.clone();

                // Update progress before starting
                if tx
                    .send(AppMessage::MassFontProgress {
                        index: idx + 1,
                        total,
                        name: current_name.clone(),
                    }).await.is_err() { return; }

                // Run installation for this specific font
                let output = tokio::process::Command::new(cmd)
                    .args(["font", "install", &current_name])
                    .output()
                    .await;

                if let Err(e) = output {
                    if tx
                        .send(AppMessage::Error(format!(
                            "Failed to install {}: {}",
                            current_name, e
                        ))).await.is_err() { return; }
                    return;
                }
            }

            if tx
                .send(AppMessage::Success(
                    "All Nerd Fonts have been installed successfully!".to_string(),
                )).await.is_err() {}
        });
    }

    /// Asynchronously generates a real prompt preview for a theme using isolation
    pub fn load_theme_preview(&mut self, theme: ThemeAsset, tx: mpsc::Sender<AppMessage>) {
        // 1. Cancel previous task if active to avoid race conditions
        if let Some(handle) = self.active_preview_task.take() {
            handle.abort();
        }

        // 2. Increment request ID to track the most recent request
        self.preview_request_id += 1;
        let current_id = self.preview_request_id;

        let theme_cloned = theme.clone();
        let themes_dir = self.themes_dir.clone();
        let cmd = OMP_BINARY;

        let handle = tokio::spawn(async move {
            let final_theme_path = if !theme_cloned.is_local {
                if let Some(url) = &theme_cloned.download_url {
                    match crate::api::download_to_temp(&theme_cloned.name, url).await {
                        Ok(p) => p,
                        Err(e) => {
                            if tx
                                .send(AppMessage::ThemePreviewLoaded {
                                    theme: theme_cloned,
                                    preview: format!(" Error downloading preview: {}", e),
                                    request_id: current_id,
                                }).await.is_err() { return; }
                            return;
                        }
                    }
                } else {
                    return;
                }
            } else {
                // Ensure we use the clean name and append the proper extension
                themes_dir.join(format!("{}.omp.json", theme_cloned.name))
            };

            // Verify file exists and get absolute path
            let final_theme_path = match final_theme_path.canonicalize() {
                Ok(p) => p,
                Err(_) => {
                    if !final_theme_path.exists() {
                        if tx
                            .send(AppMessage::ThemePreviewLoaded {
                                theme: theme_cloned,
                                preview: " Error: Theme file not found locally ".to_string(),
                                request_id: current_id,
                            }).await.is_err() { return; }
                        return;
                    }
                    final_theme_path
                }
            };

            let mut cmd_obj = tokio::process::Command::new(cmd);

            // Get current working directory for a more realistic preview
            let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

            // IMPORTANT: Clear environment variables that might force OMP to use the current theme
            cmd_obj
                .env_remove("POSH_THEME")
                .env_remove("OMP_CONFIG")
                .env_remove("POSH_THEME_DIR")
                .kill_on_drop(true)
                .args([
                    "print",
                    "primary",
                    "--config",
                    &final_theme_path.to_string_lossy(),
                    "--shell",
                    "plain", // Using 'plain' avoids shell-specific fallback logic
                    "--force",
                    "--status",
                    "0",
                    "--pwd",
                    &current_dir.to_string_lossy(),
                    "--no-status",
                ]);

            // Set a strict timeout for the preview generation
            let output =
                tokio::time::timeout(std::time::Duration::from_millis(1500), cmd_obj.output())
                    .await;

            match output {
                Ok(Ok(out)) => {
                    let raw = String::from_utf8_lossy(&out.stdout).to_string();
                    let preview = if raw.trim().is_empty() {
                        " Error: Empty preview generated ".to_string()
                    } else {
                        format!(" {}", raw.trim_end())
                    };

                    if tx
                        .send(AppMessage::ThemePreviewLoaded {
                            theme: theme_cloned,
                            preview,
                            request_id: current_id,
                        }).await.is_err() {}
                }
                Ok(Err(e)) => {
                    if tx
                        .send(AppMessage::ThemePreviewLoaded {
                            theme: theme_cloned,
                            preview: format!(" Command error: {}", e),
                            request_id: current_id,
                        }).await.is_err() {}
                }
                Err(_) => {
                    if tx
                        .send(AppMessage::ThemePreviewLoaded {
                            theme: theme_cloned,
                            preview: " Timeout: Theme too complex for quick preview ".to_string(),
                            request_id: current_id,
                        }).await.is_err() {}
                }
            }
        });

        self.active_preview_task = Some(handle);
    }

    /// Handles automatic installation of Oh My Posh via WinGet (Windows) or Homebrew (Linux/macOS)
    pub fn install_omp(&self, tx: mpsc::Sender<AppMessage>) {
        tokio::spawn(async move {
            if tx
                .send(AppMessage::InstallProgress {
                    line: "Starting Oh My Posh installation...".to_string(),
                }).await.is_err() { return; }

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
                Ok(mut child) => match child.wait().await {
                    Ok(status) if status.success() => {
                        if tx.send(AppMessage::InstallFinished).await.is_err() {}
                    }
                    _ => {
                        if tx
                            .send(AppMessage::Error(
                                "Installation failed via Winget".to_string(),
                            )).await.is_err() {}
                    }
                },
                Err(e) => {
                    if tx
                        .send(AppMessage::Error(format!(
                            "Could not start installer: {}",
                            e
                        ))).await.is_err() {}
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
        let legacy_start = "# <PoshBuddy: START";
        let legacy_end = "# <PoshBuddy: END";
        let start_marker = format!("## POSHBUDDY AUTO-GENERATED CONFIG - START ({})", key);
        let end_marker = format!("## POSHBUDDY AUTO-GENERATED CONFIG - END ({})", key);

        let mut new_lines = Vec::new();
        let mut inside_block = false;
        let mut found = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Purge both legacy and new markers
            if trimmed.starts_with(legacy_start) || trimmed.starts_with(legacy_end) {
                found = true;
                continue;
            }

            if trimmed.starts_with(&start_marker) {
                inside_block = true;
                found = true;
                continue;
            }
            if trimmed.starts_with(&end_marker) {
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

    /// Advanced 4-stage theme application flow: Download -> Verify -> Backup -> Apply
    pub fn apply_theme_advanced(&mut self, theme: ThemeAsset, tx: mpsc::Sender<AppMessage>) {
        let name = theme.name.clone();
        let themes_dir = self.themes_dir.clone();
        let profiles = self.detected_profiles.clone();
        let backup_manager = self.backup_manager.clone();
        let tx_cloned = tx.clone();

        self.state = AppState::ApplyingProgress {
            name: name.clone(),
            stage: 0, // Preparing
            progress: 10.0,
        };

        tokio::spawn(async move {
            // Stage 0: Download/Locate
            if tx_cloned
                .send(AppMessage::InstallUpdate {
                    stage: 0,
                    percentage: 25.0,
                }).await.is_err() { return; }

            let source_path = if theme.is_local {
                let name_clean = theme.name.replace(".omp.json", "");
                themes_dir.join(format!("{}.omp.json", name_clean))
            } else {
                if !crate::api::check_internet_connectivity() {
                    if tx_cloned
                        .send(AppMessage::Error(
                            "No internet connection detected. Check your network.".to_string(),
                        )).await.is_err() { return; }
                    return;
                }

                if let Some(url) = theme.download_url {
                    // Download to a temporary location first for verification
                    let temp_dir = std::env::temp_dir();
                    match crate::api::download_theme_file(&theme.name, &url, &temp_dir).await {
                        Ok(p) => p,
                        Err(e) => {
                            if tx_cloned
                                .send(AppMessage::Error(format!("Download failed: {}", e))).await.is_err() { return; }
                            return;
                        }
                    }
                } else {
                    if tx_cloned
                        .send(AppMessage::Error(
                            "Missing download URL for remote theme".to_string(),
                        )).await.is_err() { return; }
                    return;
                }
            };

            // Stage 1: Verify (Try to parse as JSON)
            if tx_cloned
                .send(AppMessage::InstallUpdate {
                    stage: 1,
                    percentage: 50.0,
                }).await.is_err() { return; }
            match tokio::fs::read_to_string(&source_path).await {
                Ok(content) => {
                    if serde_json::from_str::<serde_json::Value>(&content).is_err() {
                        if tx_cloned
                            .send(AppMessage::Error("Invalid theme JSON format".to_string())).await.is_err() { return; }
                        return;
                    }
                }
                Err(e) => {
                    if tx_cloned
                        .send(AppMessage::Error(format!(
                            "Could not read theme file: {}",
                            e
                        ))).await.is_err() { return; }
                    return;
                }
            }

            // Stage 2: Backup
            if tx_cloned
                .send(AppMessage::InstallUpdate {
                    stage: 2,
                    percentage: 75.0,
                }).await.is_err() { return; }
            for profile in &profiles {
                if let Err(e) = backup_manager
                    .backup_profile(profile, &format!("Apply Theme Advanced: {}", name))
                {
                    if tx_cloned
                        .send(AppMessage::Error(format!("Backup failed: {}", e))).await.is_err() { return; }
                    return;
                }
            }

            // Stage 3: Apply
            if tx_cloned
                .send(AppMessage::InstallUpdate {
                    stage: 3,
                    percentage: 90.0,
                }).await.is_err() { return; }

            let final_theme_path = if !theme.is_local {
                let dest = themes_dir.join(format!("{}.omp.json", theme.name));
                if let Err(e) = tokio::fs::copy(&source_path, &dest).await {
                    if tx_cloned
                        .send(AppMessage::Error(format!("Failed to save theme: {}", e))).await.is_err() { return; }
                    return;
                }
                dest
            } else {
                source_path
            };

            let config_line = format!(
                "oh-my-posh init powershell --config \"{}\" | Out-String | Invoke-Expression",
                final_theme_path.to_string_lossy()
            );

            let start_marker = "## POSHBUDDY AUTO-GENERATED CONFIG - START (THEME)";
            let end_marker = "## POSHBUDDY AUTO-GENERATED CONFIG - END (THEME)";
            let line_ending = if cfg!(windows) { "\r\n" } else { "\n" };

            for profile in &profiles {
                let existing_content = tokio::fs::read_to_string(profile).await.unwrap_or_default();
                let mut new_lines = Vec::new();
                let mut inside_block = false;
                let mut found = false;

                for line in existing_content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("# <PoshBuddy: START")
                        || trimmed.starts_with("# <PoshBuddy: END")
                    {
                        continue;
                    }

                    if trimmed.starts_with(start_marker) {
                        inside_block = true;
                        found = true;
                        new_lines.push(start_marker.to_string());
                        new_lines.push(format!("## Description: Apply Oh My Posh theme: {}", name));
                        new_lines.push(config_line.clone());
                    } else if trimmed.starts_with(end_marker) {
                        inside_block = false;
                        new_lines.push(end_marker.to_string());
                    } else if !inside_block {
                        new_lines.push(line.to_string());
                    }
                }

                if !found {
                    if !existing_content.is_empty() && !existing_content.ends_with('\n') {
                        new_lines.push(String::new());
                    }
                    new_lines.push(start_marker.to_string());
                    new_lines.push(format!("## Description: Apply Oh My Posh theme: {}", name));
                    new_lines.push(config_line.clone());
                    new_lines.push(end_marker.to_string());
                }

                if let Err(e) = tokio::fs::write(profile, new_lines.join(line_ending)).await {
                    if tx_cloned
                        .send(AppMessage::Error(format!("Profile update failed: {}", e))).await.is_err() { return; }
                    return;
                }
            }

            if tx_cloned
                .send(AppMessage::Success(format!(
                    "Theme '{}' applied successfully!",
                    name
                ))).await.is_err() { return; }
            if tx_cloned
                .send(AppMessage::ThemeDownloaded(final_theme_path)).await.is_err() {}
        });
    }

    /// Toggles the activation state of a plugin by adding or removing it from all detected profiles
    pub fn toggle_plugin(&mut self, plugin: &PluginAsset) -> io::Result<()> {
        let is_active = self.is_plugin_active(plugin);
        let key = format!("PLUGIN_{}", plugin.module_name.to_uppercase());

        for profile in &self.detected_profiles {
            self.backup_manager
                .backup_profile(profile, &format!("Toggle Plugin: {}", plugin.name))
                .map_err(|e| io::Error::other(e.to_string()))?;

            if is_active {
                self.remove_profile_block(profile, &key)?;
            } else {
                let payload = if let Some(init) = &plugin.init_script {
                    init.clone()
                } else {
                    format!(
                        "Import-Module {} -ErrorAction SilentlyContinue",
                        plugin.module_name
                    )
                };

                self.update_profile_safe(
                    profile,
                    &key,
                    &payload,
                    &format!("Plugin: {} - {}", plugin.name, plugin.description),
                )?;
            }
        }

        Ok(())
    }

    /// Asynchronously installs a PowerShell module via the system shell
    #[allow(dead_code)]
    pub fn install_plugin(&self, name: String, module_name: String, tx: mpsc::Sender<AppMessage>) {
        tokio::spawn(async move {
            if tx
                .send(AppMessage::InstallProgress {
                    line: format!("Installing module: {}...", name),
                }).await.is_err() { return; }

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
                    if tx.send(AppMessage::PluginInstalled(name)).await.is_err() {}
                }
                _ => {
                    if tx
                        .send(AppMessage::Error(format!(
                            "Failed to install module {}",
                            module_name
                        ))).await.is_err() {}
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
                AppMessage::ThemePreviewLoaded {
                    theme,
                    preview,
                    request_id,
                } => {
                    if request_id == self.preview_request_id {
                        let filtered = self.filtered_themes();
                        if let Some(selected_index) = self.list_state.selected() {
                            if let Some(current_theme) = filtered.get(selected_index) {
                                if current_theme.name == theme.name {
                                    self.theme_preview = preview;
                                }
                            }
                        }
                    }
                }
                AppMessage::RemoteThemesLoaded(themes) => {
                    self.remote_themes = themes;
                }
                AppMessage::ThemeDownloaded(path) => {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let clean_name = name.replace(".omp.json", "");
                        let theme_asset = ThemeAsset {
                            name: clean_name,
                            is_local: true,
                            download_url: None,
                        };

                        if !self.themes.iter().any(|t| t.name == theme_asset.name) {
                            self.themes.push(theme_asset.clone());
                            self.themes.sort_by(|a, b| a.name.cmp(&b.name));
                        }
                        self.active_config_path = Some(path);
                        self.refresh_active_segments(); // Update segments cache for the new theme
                    }
                }
                AppMessage::InstallUpdate { stage, percentage } => {
                    if let AppState::ApplyingProgress { name, .. } = &self.state {
                        self.state = AppState::ApplyingProgress {
                            name: name.clone(),
                            stage,
                            progress: percentage,
                        };
                    }
                }
                AppMessage::Success(msg) => {
                    self.state = AppState::Success(msg);
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
                AppMessage::MassFontProgress { index, total, name } => {
                    let percentage = (index as f32 / total as f32) * 100.0;
                    self.state = AppState::InstallingAllFonts {
                        progress: percentage,
                        current_font: name,
                        index,
                        total,
                    };
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

        // --- 1. GLOBAL COMMANDS (Work across most states) ---

        // Force Quit
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(true);
        }

        // Global Backup/Restore
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('r') => {
                    match self.restore_last_backup() {
                        Ok(count) if count > 0 => {
                            self.state = AppState::Success(format!(
                                "Restored {} profiles from backup",
                                count
                            ))
                        }
                        Ok(_) => {
                            self.state = AppState::Error("No backups found to restore".to_string())
                        }
                        Err(e) => self.state = AppState::Error(format!("Restore failed: {}", e)),
                    }
                    return Ok(false);
                }
                KeyCode::Char('b') => {
                    if let Err(e) = self.create_manual_backup() {
                        self.state = AppState::Error(format!("Manual backup failed: {}", e));
                    } else {
                        self.state =
                            AppState::Success("Manual backup created successfully".to_string());
                    }
                    return Ok(false);
                }
                _ => {}
            }
        }

        // Back to Welcome / Help or Clear Filter
        if key.code == KeyCode::Esc {
            if self.state == AppState::Main {
                let current_filter = match self.active_view {
                    ActiveView::Themes => &mut self.filter,
                    ActiveView::Fonts => &mut self.fonts_filter,
                    ActiveView::Segments => &mut self.segments_filter,
                };
                if !current_filter.is_empty() {
                    current_filter.clear();

                    // Reset selection to top after clearing filter
                    match self.active_view {
                        ActiveView::Themes => {
                            self.list_state.select(Some(0));
                            if let Some(t) = self.filtered_themes().first() {
                                self.theme_preview = " Loading preview...".to_string();
                                self.load_theme_preview(t.clone(), tx.clone());
                            } else {
                                self.theme_preview.clear();
                            }
                        },
                        ActiveView::Fonts => self.fonts_list_state.select(Some(0)),
                        ActiveView::Segments => self.plugins_list_state.select(Some(0)),
                    }
                    return Ok(false);
                }
            }
            if self.state != AppState::Welcome {
                self.state = AppState::Welcome;
                self.filter.clear();
                self.fonts_filter.clear();
                self.segments_filter.clear();
                return Ok(false);
            }
        }
        if key.code == KeyCode::Char('h') && !matches!(self.state, AppState::Main) && self.state != AppState::Welcome {
            self.state = AppState::Welcome;
            self.filter.clear();
            self.fonts_filter.clear();
            self.segments_filter.clear();
            return Ok(false);
        }

        // --- 2. STATE-SPECIFIC LOGIC ---

        match &self.state {
            // Dismissal states
            AppState::Success(_)
            | AppState::Error(_)
            | AppState::FontSuccess(_)
            | AppState::PluginSuccess(_) => {
                self.state = AppState::Main;
                return Ok(false);
            }

            // Busy / Progress states
            AppState::Installing(_)
            | AppState::ApplyingProgress { .. }
            | AppState::InstallingAllFonts { .. }
            | AppState::InstallingDependency { .. } => {
                if key.code == KeyCode::Char('q') {
                    return Ok(true);
                }
                return Ok(false);
            }

            AppState::DependencyMissing => {
                match key.code {
                    KeyCode::Enter => self.install_omp(tx.clone()),
                    KeyCode::Char('q') => return Ok(true),
                    _ => {}
                }
                return Ok(false);
            }

            AppState::ConfirmMassFontInstallation => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => self.install_all_fonts(tx.clone()),
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.state = AppState::Welcome
                    }
                    _ => {}
                }
                return Ok(false);
            }

            AppState::Welcome => {
                match key.code {
                    KeyCode::Up => {
                        if self.welcome_selected_action > 0 {
                            self.welcome_selected_action -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.welcome_selected_action < 8 {
                            self.welcome_selected_action += 1;
                        }
                    }
                    KeyCode::Enter => {
                        match self.welcome_selected_action {
                            0 => {
                                // Random Theme
                                if !self.themes.is_empty() {
                                    use std::time::{SystemTime, UNIX_EPOCH};
                                    let idx = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs()
                                        as usize
                                        % self.themes.len();
                                    if let Some(t) = self.themes.get(idx).cloned() {
                                        self.apply_theme_advanced(t, tx.clone());
                                    }
                                }
                            }
                            1 => self.state = AppState::ConfirmMassFontInstallation,
                            2 => {
                                // Terminal Icons
                                if let Some(p) = self
                                    .plugins
                                    .iter()
                                    .find(|p| p.name == "Terminal-Icons")
                                    .cloned()
                                {
                                    if let Err(e) = self.toggle_plugin(&p) {
                                        self.state = AppState::Error(e.to_string());
                                    } else {
                                        self.state = AppState::PluginSuccess(p.name);
                                    }
                                }
                            }
                            5 => {
                                self.state = AppState::Main;
                                self.active_view = ActiveView::Themes;
                                if let Some(t) = self.filtered_themes().first() {
                                    self.load_theme_preview(t.clone(), tx.clone());
                                }
                            }
                            6 => {
                                self.state = AppState::Main;
                                self.active_view = ActiveView::Fonts;
                            }
                            7 => {
                                self.state = AppState::Main;
                                self.active_view = ActiveView::Segments;
                            }
                            8 => {
                                // Manual Backup
                                if let Err(e) = self.create_manual_backup() {
                                    self.state = AppState::Error(format!("Backup failed: {}", e));
                                } else {
                                    self.state = AppState::Success(
                                        "Manual backup created successfully".to_string(),
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                    // --- Standardized Global View Shortcuts ---
                    KeyCode::Char('1') => {
                        self.state = AppState::Main;
                        self.active_view = ActiveView::Themes;
                        if let Some(t) = self.filtered_themes().first() {
                            self.load_theme_preview(t.clone(), tx.clone());
                        }
                    },
                    KeyCode::Char('2') => { self.state = AppState::Main; self.active_view = ActiveView::Fonts; },
                    KeyCode::Char('3') => { self.state = AppState::Main; self.active_view = ActiveView::Segments; },

                    // --- Mnemonic Quick Action Shortcuts ---
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.welcome_selected_action = 0;
                        let _ = self.handle_input(crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), tx);
                    },
                    KeyCode::Char('n') | KeyCode::Char('N') => self.state = AppState::ConfirmMassFontInstallation,
                    KeyCode::Char('i') | KeyCode::Char('I') => {
                        self.welcome_selected_action = 2;
                        let _ = self.handle_input(crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), tx);
                    },
                    KeyCode::Char('d') | KeyCode::Char('D') => self.state = AppState::Success("Diagnostics coming soon!".to_string()),
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        self.welcome_selected_action = 4;
                        let _ = self.handle_input(crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), tx);
                    },
                    KeyCode::Char('b') | KeyCode::Char('B') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.welcome_selected_action = 8;
                        let _ = self.handle_input(crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), tx);
                    },

                    KeyCode::Char('q') => return Ok(true),
                    _ => {}
                }
                return Ok(false);
            }

            AppState::Main => {
                // --- 3. VIEW-SPECIFIC NAVIGATION (In Main) ---
                match key.code {
                    KeyCode::Tab => {
                        self.active_view = match self.active_view {
                            ActiveView::Themes => ActiveView::Fonts,
                            ActiveView::Fonts => ActiveView::Segments,
                            ActiveView::Segments => ActiveView::Themes,
                        };
                        return Ok(false);
                    }
                    KeyCode::Char('1') => {
                        self.active_view = ActiveView::Themes;
                        return Ok(false);
                    }
                    KeyCode::Char('2') => {
                        self.active_view = ActiveView::Fonts;
                        return Ok(false);
                    }
                    KeyCode::Char('3') => {
                        self.active_view = ActiveView::Segments;
                        return Ok(false);
                    }

                    KeyCode::Up | KeyCode::Down => {
                        self.navigate_list(key.code == KeyCode::Down, tx.clone());
                        return Ok(false);
                    }

                    KeyCode::Enter => {
                        self.execute_active_view_action(tx.clone());
                        return Ok(false);
                    }

                    KeyCode::Backspace => {
                        match self.active_view {
                            ActiveView::Themes => {
                                self.filter.pop();
                                self.list_state.select(Some(0));
                            }
                            ActiveView::Fonts => {
                                self.fonts_filter.pop();
                                self.fonts_list_state.select(Some(0));
                            }
                            ActiveView::Segments => {
                                self.segments_filter.pop();
                                self.plugins_list_state.select(Some(0));
                            }
                        }
                        return Ok(false);
                    }

                    KeyCode::Char('q') if key.modifiers.is_empty() => return Ok(true),

                    KeyCode::Char(c) => {
                        match self.active_view {
                            ActiveView::Themes => {
                                self.filter.push(c);
                                self.list_state.select(Some(0));
                            }
                            ActiveView::Fonts => {
                                self.fonts_filter.push(c);
                                self.fonts_list_state.select(Some(0));
                            }
                            ActiveView::Segments => {
                                self.segments_filter.push(c);
                                self.plugins_list_state.select(Some(0));
                            }
                        }
                        return Ok(false);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(false)
    }

    /// Helper to handle list navigation across different views
    fn navigate_list(&mut self, forward: bool, tx: mpsc::Sender<AppMessage>) {
        match self.active_view {
            ActiveView::Themes => {
                let filtered = self.filtered_themes();
                if filtered.is_empty() {
                    return;
                }
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if forward {
                            if i >= filtered.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        } else {
                            if i == 0 {
                                filtered.len() - 1
                            } else {
                                i - 1
                            }
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
                if let Some(t) = filtered.get(i) {
                    self.theme_preview = " Loading preview...".to_string();
                    self.load_theme_preview(t.clone(), tx);
                }
            }
            ActiveView::Fonts => {
                let filtered = self.filtered_fonts();
                if filtered.is_empty() {
                    return;
                }
                let i = match self.fonts_list_state.selected() {
                    Some(i) => {
                        if forward {
                            if i >= filtered.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        } else {
                            if i == 0 {
                                filtered.len() - 1
                            } else {
                                i - 1
                            }
                        }
                    }
                    None => 0,
                };
                self.fonts_list_state.select(Some(i));
            }
            ActiveView::Segments => {
                let filtered = self.filtered_segments();
                if filtered.is_empty() {
                    return;
                }
                let i = match self.plugins_list_state.selected() {
                    Some(i) => {
                        if forward {
                            if i >= filtered.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        } else {
                            if i == 0 {
                                filtered.len() - 1
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
    }

    /// Helper to execute the primary action of the current view
    fn execute_active_view_action(&mut self, tx: mpsc::Sender<AppMessage>) {
        match self.active_view {
            ActiveView::Themes => {
                let filtered = self.filtered_themes();
                if let Some(selected) = self.list_state.selected() {
                    if let Some(theme) = filtered.get(selected) {
                        if !theme.is_local && !crate::api::check_internet_connectivity() {
                            self.state =
                                AppState::Error("No internet connection detected.".to_string());
                        } else {
                            self.apply_theme_advanced(theme.clone(), tx);
                        }
                    }
                }
            }
            ActiveView::Fonts => {
                let filtered = self.filtered_fonts();
                if let Some(selected) = self.fonts_list_state.selected() {
                    if let Some(font) = filtered.get(selected) {
                        self.state = AppState::Installing(font.name.clone());
                        self.install_font(font.name.clone(), tx);
                    }
                }
            }
            ActiveView::Segments => {
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
    }

    /// Restores the last backup created for all detected profiles
    /// Returns the number of successfully restored profiles
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

    /// Gets information about available backups for a specific profile
    pub fn get_backup_info(
        &self,
        profile_path: &std::path::Path,
    ) -> Vec<crate::backup::BackupInfo> {
        self.backup_manager
            .list_backups(profile_path)
            .unwrap_or_default()
    }

    /// Refreshes the total count of backups detected across all profiles
    pub fn refresh_backup_count(&mut self) {
        let mut count = 0;
        for profile in &self.detected_profiles {
            count += self.get_backup_info(profile).len();
        }
        self.total_backups = count;
    }

    /// Creates a manual backup of all detected PowerShell profiles
    pub fn create_manual_backup(&mut self) -> Result<(), String> {
        if self.detected_profiles.is_empty() {
            return Err("No PowerShell profiles detected".to_string());
        }

        let mut errors = Vec::new();
        for profile in &self.detected_profiles {
            // Manual backup for existing profiles
            if profile.exists() {
                if let Err(e) = self
                    .backup_manager
                    .backup_profile(profile, "Manual backup from PoshBuddy")
                {
                    errors.push(format!("{}: {}", profile.display(), e));
                }
            }
        }

        // Refresh count after backup
        self.refresh_backup_count();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("Some backups failed: {}", errors.join("; ")))
        }
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
            preview_request_id: 0,
            active_preview_task: None,
            active_segments: HashSet::new(),
        }
    }

    #[test]
    fn test_filtered_themes() {
        let mut app = mock_app();
        app.themes = vec![
            ThemeAsset {
                name: "bubbles.omp.json".to_string(),
                is_local: true,
                download_url: None,
            },
            ThemeAsset {
                name: "joker.omp.json".to_string(),
                is_local: true,
                download_url: None,
            },
            ThemeAsset {
                name: "M365.omp.json".to_string(),
                is_local: true,
                download_url: None,
            },
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
                std::fs::set_permissions(&where_path, std::fs::Permissions::from_mode(0o755))
                    .unwrap();
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
                ThemeAsset {
                    name: "agnoster".to_string(),
                    is_local: true,
                    download_url: None,
                },
                ThemeAsset {
                    name: "amro".to_string(),
                    is_local: true,
                    download_url: None,
                },
                ThemeAsset {
                    name: "atomic".to_string(),
                    is_local: true,
                    download_url: None,
                },
                ThemeAsset {
                    name: "catppuccin_frappe".to_string(),
                    is_local: true,
                    download_url: None,
                },
                ThemeAsset {
                    name: "Catppuccin_Macchiato".to_string(),
                    is_local: true,
                    download_url: None,
                },
                ThemeAsset {
                    name: "cyberpunk".to_string(),
                    is_local: true,
                    download_url: None,
                },
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
            preview_request_id: 0,
            active_preview_task: None,
            active_segments: HashSet::new(),
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
