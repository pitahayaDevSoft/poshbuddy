use ratatui::widgets::ListState;
use std::fs;
use std::io;
use std::path::PathBuf;
use tokio::sync::mpsc;

const OMP_BINARY: &str = if cfg!(windows) {
    "oh-my-posh.exe"
} else {
    "oh-my-posh"
};
const WHERE_CMD: &str = if cfg!(windows) { "where.exe" } else { "which" };

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActiveView {
    Themes,
    Fonts,
    Plugins,
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
    InstallingDependency {
        current_action: String,
        log: Vec<String>,
    },
    Installing(String),
    Success(String),
    FontSuccess(String),
    PluginSuccess(String),
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct FontAsset {
    pub name: String,
}

/// Metadata for a PowerShell module/extension
#[derive(Clone, Debug)]
pub struct PluginAsset {
    pub name: String,
    pub description: String,
    pub documentation: String,
    pub module_name: String,
    pub init_script: Option<String>,
}

/// Message types sent across the mpsc channel to update the TUI from background tasks
pub enum AppMessage {
    ThemesLoaded(Vec<String>),
    FontsLoaded(Vec<FontAsset>),
    ThemePreviewLoaded { theme: String, preview: String },
    FontInstalled(String),
    PluginInstalled(String),
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
    pub plugins_list_state: ListState,
    pub plugins: Vec<PluginAsset>,
    pub plugins_filter: String,
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

        let mut plugins_list_state = ListState::default();
        plugins_list_state.select(Some(0));

        // 1. Initial system diagnostics
        let has_nerd_font = Self::check_nerd_font();
        let detected_profiles = Self::detect_profiles();
        let specs = Self::gather_system_specs(has_nerd_font);

        let mut app = App {
            state: AppState::Onboarding(specs),
            active_view: ActiveView::Themes,
            themes: Vec::new(),
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
                    name: "posh-git".to_string(),
                    description: "Powerful Git status summary and tab-completion for PowerShell.".to_string(),
                    documentation: "Provides info about your current branch, staged files, and ahead/behind status.".to_string(),
                    module_name: "posh-git".to_string(),
                    init_script: None,
                },
                PluginAsset {
                    name: "zoxide (z Explorer)".to_string(),
                    description: "A smarter cd command. It remembers which directories you use most often.".to_string(),
                    documentation: "Usage: type 'z <name>' to jump. Replaces 'cd' with intelligent fuzzy matching.".to_string(),
                    module_name: "zoxide".to_string(), 
                    init_script: Some("zoxide init pwsh | Invoke-Expression".to_string()),
                },
                PluginAsset {
                    name: "PSReadLine Mastery".to_string(),
                    description: "Enables Predictive IntelliSense (fish-like) and syntax highlighting.".to_string(),
                    documentation: "Optimizes command history search and adds visual feedback while typing.".to_string(),
                    module_name: "PSReadLine".to_string(),
                    init_script: Some("Set-PSReadLineOption -PredictionSource History\nSet-PSReadLineOption -PredictionViewStyle ListView".to_string()),
                },
            ],
            filter: String::new(),
            fonts_filter: String::new(),
            plugins_filter: String::new(),
            themes_dir,
            version: "0.2.1-rust".to_string(),
            list_state,
            fonts_list_state,
            plugins_list_state,
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
            .args([
                "-Command",
                "(Get-ItemProperty -Path 'HKCU:\\Console' -ErrorAction SilentlyContinue).FaceName",
            ])
            .output();

        if let Ok(out) = output {
            let name = String::from_utf8_lossy(&out.stdout).to_lowercase();
            if name.trim().is_empty() {
                return true;
            }
            name.contains("nf")
                || name.contains("nerd")
                || name.contains("retina")
                || name.contains("code")
                || name.contains("meslo")
        } else {
            true
        }
    }

    /// Returns a filtered list of themes based on search criteria
    pub fn filtered_themes(&self) -> Vec<String> {
        let filter_lower = self.filter.to_lowercase();
        self.themes
            .iter()
            .filter(|t| t.to_lowercase().contains(&filter_lower))
            .cloned()
            .collect()
    }

    /// Returns a filtered list of fonts based on search criteria
    pub fn filtered_fonts(&self) -> Vec<FontAsset> {
        let filter_lower = self.fonts_filter.to_lowercase();
        self.fonts
            .iter()
            .filter(|f| f.name.to_lowercase().contains(&filter_lower))
            .cloned()
            .collect()
    }

    /// Returns a filtered list of plugins based on search criteria
    pub fn filtered_plugins(&self) -> Vec<PluginAsset> {
        let filter_lower = self.plugins_filter.to_lowercase();
        self.plugins
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&filter_lower))
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

    /// Asynchronously generates a real prompt preview for a theme using isolation (no parent environment inheritance)
    pub fn load_theme_preview(&self, theme_name: String, tx: mpsc::Sender<AppMessage>) {
        let cmd = OMP_BINARY;
        let theme_path = self.themes_dir.join(&theme_name);
        let theme_name_cloned = theme_name.clone();

        tokio::spawn(async move {
            let mut cmd_obj = tokio::process::Command::new(cmd);
            // Cleaning parent env vars to ensure we see the theme as specified, ignoring current shell profile
            cmd_obj
                .env_clear()
                .env("PATH", std::env::var("PATH").unwrap_or_default())
                .env(
                    "USERPROFILE",
                    std::env::var("USERPROFILE").unwrap_or_default(),
                )
                .env(
                    "SYSTEMROOT",
                    std::env::var("SYSTEMROOT").unwrap_or_default(),
                )
                .env(
                    "SystemDrive",
                    std::env::var("SystemDrive").unwrap_or_default(),
                )
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
                    let _ = tx
                        .send(AppMessage::ThemePreviewLoaded {
                            theme: theme_name_cloned,
                            preview,
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(AppMessage::ThemePreviewLoaded {
                            theme: theme_name_cloned,
                            preview: format!(" Error: {}", e),
                        })
                        .await;
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

    /// Checks if a plugin is currently 'activated' (imported) in at least one PowerShell profile
    pub fn is_plugin_active(&self, plugin: &PluginAsset) -> bool {
        for profile in &self.detected_profiles {
            if !profile.exists() {
                continue;
            }
            if let Ok(content) = fs::read_to_string(profile) {
                let check_str = if let Some(init) = &plugin.init_script {
                    init.split('\n').next().unwrap_or(init).to_string()
                } else {
                    format!("Import-Module {}", plugin.module_name)
                };
                if content.contains(&check_str) {
                    return true;
                }
            }
        }
        false
    }

    /// Toggles the activation state of a plugin by adding or removing it from all detected profiles
    pub fn toggle_plugin(&self, plugin: &PluginAsset) -> io::Result<()> {
        let is_active = self.is_plugin_active(plugin);
        let line_ending = if cfg!(windows) { "\r\n" } else { "\n" };

        let payload = if let Some(init) = &plugin.init_script {
            init.clone()
        } else {
            format!("Import-Module {}", plugin.module_name)
        };

        for profile in &self.detected_profiles {
            if let Some(parent) = profile.parent() {
                fs::create_dir_all(parent)?;
            }

            let content = if profile.exists() {
                fs::read_to_string(profile)?
            } else {
                String::new()
            };

            let mut new_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

            if is_active {
                // Remove the plugin
                new_lines.retain(|l| !l.contains(&payload.split('\n').next().unwrap_or(&payload)));
            } else {
                // Add the plugin
                if !new_lines
                    .iter()
                    .any(|l| l.contains(&payload.split('\n').next().unwrap_or(&payload)))
                {
                    new_lines.push(payload.clone());
                }
            }

            fs::write(profile, new_lines.join(line_ending))?;
        }
        Ok(())
    }

    /// Asynchronously installs a PowerShell module via the system shell
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::ListState;
    use std::path::PathBuf;

    fn mock_app() -> App {
        App {
            state: AppState::Loading,
            active_view: ActiveView::Themes,
            themes: Vec::new(),
            fonts: Vec::new(),
            plugins: Vec::new(),
            filter: String::new(),
            fonts_filter: String::new(),
            plugins_filter: String::new(),
            themes_dir: PathBuf::from("/tmp"),
            version: "test".to_string(),
            list_state: ListState::default(),
            fonts_list_state: ListState::default(),
            plugins_list_state: ListState::default(),
            spinner_tick: 0,
            has_nerd_font: false,
            theme_preview: String::new(),
            detected_profiles: Vec::new(),
        }
    }

    #[test]
    fn test_filtered_themes() {
        let mut app = mock_app();
        app.themes = vec![
            "bubbles.omp.json".to_string(),
            "joker.omp.json".to_string(),
            "M365.omp.json".to_string(),
        ];

        // Empty filter should return all
        assert_eq!(app.filtered_themes().len(), 3);

        // Case-insensitive matching
        app.filter = "JOKER".to_string();
        assert_eq!(app.filtered_themes(), vec!["joker.omp.json".to_string()]);

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
            },
            FontAsset {
                name: "FiraCode".to_string(),
            },
            FontAsset {
                name: "JetBrainsMono".to_string(),
            },
        ];

        // Empty filter should return all
        assert_eq!(app.filtered_fonts().len(), 3);

        // Case-insensitive matching
        app.fonts_filter = "fira".to_string();
        assert_eq!(
            app.filtered_fonts(),
            vec![FontAsset {
                name: "FiraCode".to_string()
            }]
        );

        // Partial matching
        app.fonts_filter = "Code".to_string();
        assert_eq!(app.filtered_fonts().len(), 2);

        // No match
        app.fonts_filter = "Wingdings".to_string();
        assert_eq!(app.filtered_fonts().len(), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::ListState;

    fn create_test_app() -> App {
        App {
            state: AppState::Main,
            active_view: ActiveView::Themes,
            themes: vec![
                "agnoster".to_string(),
                "amro".to_string(),
                "atomic".to_string(),
                "catppuccin_frappe".to_string(),
                "Catppuccin_Macchiato".to_string(),
                "cyberpunk".to_string(),
            ],
            fonts: vec![],
            filter: "".to_string(),
            fonts_filter: "".to_string(),
            themes_dir: PathBuf::from("/mock/themes/dir"),
            version: "1.0.0".to_string(),
            list_state: ListState::default(),
            fonts_list_state: ListState::default(),
            spinner_tick: 0,
            has_nerd_font: true,
            theme_preview: "".to_string(),
            detected_profiles: vec![],
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
        assert!(filtered.contains(&"catppuccin_frappe".to_string()));
        assert!(filtered.contains(&"Catppuccin_Macchiato".to_string()));
    }

    #[test]
    fn test_filtered_themes_partial_match() {
        let mut app = create_test_app();
        app.filter = "amro".to_string();
        let filtered = app.filtered_themes();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "amro");
    }

    #[test]
    fn test_filtered_themes_no_match() {
        let mut app = create_test_app();
        app.filter = "nonexistent".to_string();
        let filtered = app.filtered_themes();
        assert_eq!(filtered.len(), 0);
    }
}
