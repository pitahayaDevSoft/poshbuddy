use ratatui::widgets::ListState;
use std::collections::HashSet;
use std::env;
use std::fs;

pub mod models;
pub use models::*;
pub mod handlers;
pub mod services;

pub const OMP_BINARY: &str = "oh-my-posh";
pub const WHERE_CMD: &str = "where";

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

        let mut segments_list_state = ListState::default();
        segments_list_state.select(Some(0));

        // 1. Initial system diagnostics
        let has_nerd_font = Self::check_nerd_font();
        let detected_profiles = Self::detect_profiles();
        let specs = Self::gather_system_specs(has_nerd_font);

        // 2. Load existing local themes
        let mut local_themes = Vec::new();
        if let Ok(entries) = fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|s| s.ends_with(".omp.json"))
                {
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
            plugins: crate::assets::get_default_plugins(),
            segments: crate::assets::get_default_segments(),
            filter: String::new(),
            fonts_filter: String::new(),
            segments_filter: String::new(),
            themes_dir,
            version: env!("CARGO_PKG_VERSION").to_string(),
            list_state,
            fonts_list_state,
            segments_list_state,
            spinner_tick: 0,
            has_nerd_font,
            theme_preview: String::new(),
            detected_profiles: detected_profiles.clone(),
            active_config_path: None,
            backup_manager: crate::backup::BackupManager::new(None),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::ListState;
    use std::path::PathBuf;
    use std::sync::Mutex;

    #[test]
    fn test_contains_ignore_ascii_case() {
        // Empty string cases
        assert!(contains_ignore_ascii_case("haystack", ""));
        assert!(!contains_ignore_ascii_case("", "needle"));
        assert!(contains_ignore_ascii_case("", ""));

        // Exact matches
        assert!(contains_ignore_ascii_case("hello world", "world"));
        assert!(contains_ignore_ascii_case("world", "world"));

        // Case-insensitive matches
        assert!(contains_ignore_ascii_case("Hello World", "WORLD"));
        assert!(contains_ignore_ascii_case("hello world", "WORLD"));
        assert!(contains_ignore_ascii_case("HELLO WORLD", "world"));
        assert!(contains_ignore_ascii_case("HeLlO wOrLd", "wOrLd"));

        // Partial matches
        assert!(contains_ignore_ascii_case("HelloWorld", "lowor"));
        assert!(contains_ignore_ascii_case("HelloWorld", "Owo"));

        // No match
        assert!(!contains_ignore_ascii_case("Hello World", "planet"));
        assert!(!contains_ignore_ascii_case("Hello World", "worlds"));

        // Needle longer than haystack
        assert!(!contains_ignore_ascii_case("hi", "hello"));
        assert!(!contains_ignore_ascii_case("", "a"));

        // Special characters
        assert!(contains_ignore_ascii_case("hello-world_123", "-WORLD_"));
        assert!(contains_ignore_ascii_case("test@#$%", "@#$%"));
        assert!(!contains_ignore_ascii_case("test@#$%", "^&*"));
    }

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
            unsafe {
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
            segments_list_state: ListState::default(),
            plugins: Vec::new(),
            segments: Vec::new(),
            segments_filter: String::new(),
            spinner_tick: 0,
            has_nerd_font: false,
            theme_preview: String::new(),
            detected_profiles: Vec::new(),
            active_config_path: None,
            backup_manager: crate::backup::BackupManager::new(Some(10)),
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
        assert_eq!(app.filtered_themes_count(), 3);

        // Case-insensitive matching
        app.filter = "JOKER".to_string();
        assert_eq!(app.filtered_themes()[0].name, "joker.omp.json");
        assert_eq!(app.filtered_themes_count(), 1);

        // Partial matching
        app.filter = "omp".to_string();
        assert_eq!(app.filtered_themes().len(), 3);
        assert_eq!(app.filtered_themes_count(), 3);

        // No match
        app.filter = "nonexistent".to_string();
        assert_eq!(app.filtered_themes().len(), 0);
        assert_eq!(app.filtered_themes_count(), 0);
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
        assert_eq!(app.filtered_fonts_count(), 3);

        // Case-insensitive matching
        app.fonts_filter = "fira".to_string();
        let fira_font = app.filtered_font_at(0).unwrap();
        assert_eq!(fira_font.name, "FiraCode");
        assert_eq!(app.filtered_fonts_count(), 1);

        // Partial matching
        app.fonts_filter = "Code".to_string();
        assert_eq!(app.filtered_fonts_count(), 2);

        // No match
        app.fonts_filter = "Wingdings".to_string();
        assert_eq!(app.filtered_fonts_count(), 0);
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
            unsafe { env::set_var("PATH", &dir) };

            let profiles = App::detect_profiles();

            // Restore original PATH for cleanup (EnvGuard will also restore it)
            unsafe { env::set_var("PATH", &original_path) };

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
            unsafe { env::set_var("PATH", &dir) };

            let profiles = App::detect_profiles();

            // Restore original PATH
            unsafe { env::set_var("PATH", &original_path) };

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
            unsafe {
                env::remove_var("WT_SESSION");
                env::remove_var("TERM_PROGRAM");
            }

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
            unsafe { env::set_var("WT_SESSION", "1") };
            let specs = App::gather_system_specs(true);
            assert!(
                specs.is_windows_terminal,
                "Expected is_windows_terminal to be true when WT_SESSION is set"
            );
            assert!(specs.has_nerd_font, "Expected has_nerd_font to be true");

            // Scenario 3: TERM_PROGRAM=vscode set
            unsafe {
                env::remove_var("WT_SESSION");
                env::set_var("TERM_PROGRAM", "vscode");
            }
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
            unsafe { env::set_var("PATH", &dir) };
            #[cfg(windows)]
            unsafe { env::set_var("PATH", format!("{};{}", dir.display(), original_path)) };

            let specs = App::gather_system_specs(false);

            // Restore original PATH
            unsafe { env::set_var("PATH", &original_path) };

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
            themes_dir: std::path::PathBuf::from("/mock/themes/dir"),
            version: "1.0.0".to_string(),
            list_state: ListState::default(),
            fonts_list_state: ListState::default(),
            segments_list_state: ListState::default(),
            plugins: vec![],
            segments: vec![],
            segments_filter: "".to_string(),
            spinner_tick: 0,
            has_nerd_font: true,
            theme_preview: "".to_string(),
            detected_profiles: vec![],
            active_config_path: None,
            backup_manager: crate::backup::BackupManager::new(Some(10)),
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
