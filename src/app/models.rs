use ratatui::widgets::ListState;
use std::collections::HashSet;
use std::path::PathBuf;

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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    pub local_theme_names: HashSet<String>, // Cache of local theme names to avoid O(N*M) loop inside filtered_themes
}
