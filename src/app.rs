use serde::{Deserialize, Serialize};
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

#[derive(PartialEq, Debug)]
pub enum AppState {
    Loading,
    Main,
    Installing(String),
    Error(String),
}

#[derive(Clone, Debug)]
pub struct FontAsset {
    pub name: String,
}

pub enum AppMessage {
    ThemesLoaded(Vec<String>),
    FontsLoaded(Vec<FontAsset>),
    ThemePreviewLoaded(String),
    FontInstalled(String),
    Error(String),
}

pub struct App {
    pub state: AppState,
    pub active_view: ActiveView,
    pub themes: Vec<String>,
    pub fonts: Vec<FontAsset>,
    pub filter: String,
    pub fonts_filter: String,
    pub themes_dir: PathBuf,
    pub profile_path: PathBuf,
    pub version: String,
    pub list_state: ListState,
    pub fonts_list_state: ListState,
    pub spinner_tick: usize,
    pub has_nerd_font: bool,
    pub theme_preview: String,
}

impl App {
    pub fn new() -> Self {
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
        }
    }

    pub fn check_nerd_font() -> bool {
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

    pub fn filtered_themes(&self) -> Vec<String> {
        self.themes
            .iter()
            .filter(|t| t.to_lowercase().contains(&self.filter.to_lowercase()))
            .cloned()
            .collect()
    }

    pub fn filtered_fonts(&self) -> Vec<FontAsset> {
        self.fonts
            .iter()
            .filter(|f| f.name.to_lowercase().contains(&self.fonts_filter.to_lowercase()))
            .cloned()
            .collect()
    }

    pub fn apply_theme(&self, theme_name: &str) -> io::Result<()> {
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

    pub fn load_theme_preview(&self, theme_name: String, tx: mpsc::Sender<AppMessage>) {
        let cmd = if cfg!(windows) {
            "oh-my-posh"
        } else {
            "oh-my-posh.exe"
        };
        let theme_path = self.themes_dir.join(&theme_name);

        tokio::spawn(async move {
            let output = tokio::process::Command::new(cmd)
                .args(["print", "primary", "--config", &theme_path.to_string_lossy(), "--shell", "pwsh"])
                .output()
                .await;

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
