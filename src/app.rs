use std::path::PathBuf;
use tokio::sync::mpsc;
use std::fs;
use std::io;
use ratatui::widgets::ListState;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActiveView {
    Themes,
    Fonts,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SystemSpecs {
    pub is_pwsh_7: bool,
    pub has_nerd_font: bool,
    pub is_windows_terminal: bool,
}

#[derive(PartialEq, Debug)]
pub enum AppState {
    Onboarding(SystemSpecs),
    Loading,
    Main,
    DependencyMissing,
    InstallingDependency { current_action: String, log: Vec<String> },
    Installing(String),
    Success(String),
    FontSuccess(String),
    Error(String),
}

#[derive(Clone, Debug)]
pub struct FontAsset {
    pub name: String,
}

/// Message types sent across the mpsc channel to update the TUI from background tasks
pub enum AppMessage {
    ThemesLoaded(Vec<String>),
    FontsLoaded(Vec<FontAsset>),
    ThemePreviewLoaded { theme: String, preview: String },
    FontInstalled(String),
    InstallProgress { line: String },
    InstallFinished,
    Error(String),
}

/// State container for the PoshBuddy application
pub struct App {
    pub state: AppState,
    pub active_view: ActiveView,
    pub themes: Vec<String>,
    pub fonts: Vec<FontAsset>,
    pub filter: String,
    pub fonts_filter: String,
    pub themes_dir: PathBuf,
    pub version: String,
    pub list_state: ListState,
    pub fonts_list_state: ListState,
    pub spinner_tick: usize,
    pub has_nerd_font: bool,
    pub theme_preview: String,
    pub detected_profiles: Vec<PathBuf>,
}

impl App {
    /// Initializes a new application instance with dynamic system detection
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("Could not find home directory");
        let themes_dir = home.join(".poshthemes");

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut fonts_list_state = ListState::default();
        fonts_list_state.select(Some(0));

        // 1. Initial system diagnostics
        let has_nerd_font = Self::check_nerd_font();
        let detected_profiles = Self::detect_profiles();
        let specs = Self::gather_system_specs(has_nerd_font);

        let mut app = App {
            state: AppState::Onboarding(specs),
            active_view: ActiveView::Themes,
            themes: Vec::new(),
            fonts: Vec::new(),
            filter: String::new(),
            fonts_filter: String::new(),
            themes_dir,
            version: "0.2.1-rust".to_string(),
            list_state,
            fonts_list_state,
            spinner_tick: 0,
            has_nerd_font,
            theme_preview: String::new(),
            detected_profiles,
        };

        // 2. Pre-check for Oh My Posh installation
        if !app.check_omp_installed() {
            app.state = AppState::DependencyMissing;
        }

        app
    }

    /// Verifies if 'oh-my-posh' binary is present in the system PATH
    pub fn check_omp_installed(&self) -> bool {
        let cmd = if cfg!(windows) { "where.exe" } else { "which" };
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
            || std::env::var("TERM_PROGRAM").map(|v| v == "vscode").unwrap_or(false);
        
        // Checking for PowerShell 7 binary (pwsh)
        let cmd = if cfg!(windows) { "where.exe" } else { "which" };
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
        // Check both classic PowerShell and modern pwsh
        let shells = if cfg!(windows) {
            vec!["powershell", "pwsh"]
        } else {
            vec!["pwsh"]
        };

        for shell in shells {
            let output = std::process::Command::new(shell)
                .args(["-NoProfile", "-Command", "Write-Host -NoNewline $PROFILE"])
                .output();

            if let Ok(out) = output {
                let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !path_str.is_empty() {
                    let path = PathBuf::from(path_str);
                    profiles.push(path);
                }
            }
        }
        
        // Cleanup and deduplicate (linked profiles)
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
            .args(["-Command", "(Get-ItemProperty -Path 'HKCU:\\Console' -ErrorAction SilentlyContinue).FaceName"])
            .output();

        if let Ok(out) = output {
            let name = String::from_utf8_lossy(&out.stdout).to_lowercase();
            if name.trim().is_empty() {
                return true;
            }
            name.contains("nf") || name.contains("nerd") || name.contains("retina") || name.contains("code") || name.contains("meslo")
        } else {
            true
        }
    }

    /// Returns a filtered list of themes based on search criteria
    pub fn filtered_themes(&self) -> Vec<String> {
        self.themes
            .iter()
            .filter(|t| t.to_lowercase().contains(&self.filter.to_lowercase()))
            .cloned()
            .collect()
    }

    /// Returns a filtered list of fonts based on search criteria
    pub fn filtered_fonts(&self) -> Vec<FontAsset> {
        self.fonts
            .iter()
            .filter(|f| f.name.to_lowercase().contains(&self.fonts_filter.to_lowercase()))
            .cloned()
            .collect()
    }

    /// Atomically updates all detected shell profiles to initialize Oh My Posh with the selected theme
    pub fn apply_theme(&self, theme_name: &str) -> io::Result<()> {
        let theme_path = self.themes_dir.join(theme_name);
        let config_line = format!(
            "oh-my-posh init pwsh --config '{}' | Invoke-Expression",
            theme_path.to_string_lossy().replace("'", "''")
        );

        for profile in &self.detected_profiles {
            if let Some(parent) = profile.parent() {
                fs::create_dir_all(parent)?;
            }

            let content = if profile.exists() {
                fs::read_to_string(profile)?
            } else {
                String::new()
            };

            let mut new_content = Vec::new();
            let mut found = false;

            // Search for existing Oh My Posh init line to replace or add a new one at the end
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
                new_content.push(config_line.clone());
            }

            let line_ending = if cfg!(windows) { "\r\n" } else { "\n" };
            fs::write(profile, new_content.join(line_ending))?;
        }

        Ok(())
    }

    /// Triggers an asynchronous font installation via Oh My Posh CLI
    pub fn install_font(&self, font_name: String, tx: mpsc::Sender<AppMessage>) {
        let cmd = if cfg!(windows) {
            "oh-my-posh"
        } else {
            "oh-my-posh.exe"
        };

        let font_name_cloned = font_name.clone();
        tokio::spawn(async move {
            let output = tokio::process::Command::new(cmd)
                .args(["font", "install", &font_name_cloned])
                .output()
                .await;
            
            match output {
                Ok(_) => { let _ = tx.send(AppMessage::FontInstalled(font_name_cloned)).await; }
                Err(e) => { let _ = tx.send(AppMessage::Error(e.to_string())).await; }
            }
        });
    }

    /// Asynchronously generates a real prompt preview for a theme using isolation (no parent environment inheritance)
    pub fn load_theme_preview(&self, theme_name: String, tx: mpsc::Sender<AppMessage>) {
        let cmd = if cfg!(windows) {
            "oh-my-posh.exe"
        } else {
            "oh-my-posh"
        };
        let theme_path = self.themes_dir.join(&theme_name);
        let theme_name_cloned = theme_name.clone();

        tokio::spawn(async move {
            let mut cmd_obj = tokio::process::Command::new(cmd);
            // Cleaning parent env vars to ensure we see the theme as specified, ignoring current shell profile
            cmd_obj.env_clear()
                  .env("PATH", std::env::var("PATH").unwrap_or_default())
                  .env("USERPROFILE", std::env::var("USERPROFILE").unwrap_or_default())
                  .env("SYSTEMROOT", std::env::var("SYSTEMROOT").unwrap_or_default())
                  .env("SystemDrive", std::env::var("SystemDrive").unwrap_or_default())
                  .env("TEMP", std::env::var("TEMP").unwrap_or_default())
                  .args(["print", "primary", "--config", &theme_path.to_string_lossy(), "--shell", "pwsh", "--force", "--status", "0"]);

            let output = cmd_obj.output().await;

            match output {
                Ok(out) => {
                    let raw = String::from_utf8_lossy(&out.stdout).to_string();
                    let preview = format!(" {}", raw.trim_end());
                    let _ = tx.send(AppMessage::ThemePreviewLoaded { 
                        theme: theme_name_cloned, 
                        preview 
                    }).await;
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::ThemePreviewLoaded { 
                        theme: theme_name_cloned, 
                        preview: format!(" Error: {}", e) 
                    }).await;
                }
            }
        });
    }

    /// Handles automatic installation of Oh My Posh via WinGet (Windows) or Homebrew (Linux/macOS)
    pub fn install_omp(&self, tx: mpsc::Sender<AppMessage>) {
        tokio::spawn(async move {
            let _ = tx.send(AppMessage::InstallProgress { line: "Starting Oh My Posh installation...".to_string() }).await;
            
            let child = if cfg!(windows) {
                tokio::process::Command::new("winget")
                    .args(["install", "JanDeDobbeleer.OhMyPosh", "--accept-package-agreements", "--accept-source-agreements"])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
            } else {
                tokio::process::Command::new("brew")
                    .args(["install", "oh-my-posh"])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
            }.map_err(|e| e.to_string());

            match child {
                Ok(mut child) => {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let stdout = child.stdout.take().unwrap();
                    let mut reader = BufReader::new(stdout).lines();

                    // Stream output lines to the TUI debug box in real-time
                    while let Ok(Some(line)) = reader.next_line().await {
                        let _ = tx.send(AppMessage::InstallProgress { line }).await;
                    }

                    match child.wait().await {
                        Ok(status) if status.success() => {
                            let _ = tx.send(AppMessage::InstallFinished).await;
                        }
                        _ => {
                            let _ = tx.send(AppMessage::Error("Installation failed via Winget".to_string())).await;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("Could not start installer: {}", e))).await;
                }
            }
        });
    }
}
