use crate::app::models::*;
use crate::app::App;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use tokio::sync::mpsc;

impl App {
    pub fn handle_messages(
        &mut self,
        rx: &mut mpsc::Receiver<AppMessage>,
        tx: mpsc::Sender<AppMessage>,
    ) {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                AppMessage::ThemesLoaded(new_themes) => {
                    for t in new_themes {
                        if !self.themes.iter().any(|existing| existing.name == t.name) {
                            self.local_theme_names.insert(t.name.clone());
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
                            self.local_theme_names.insert(theme_asset.name.clone());
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
        tx: mpsc::Sender<AppMessage>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
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
                        }
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
        if key.code == KeyCode::Char('h')
            && !matches!(self.state, AppState::Main)
            && self.state != AppState::Welcome
        {
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
                    }
                    KeyCode::Char('2') => {
                        self.state = AppState::Main;
                        self.active_view = ActiveView::Fonts;
                    }
                    KeyCode::Char('3') => {
                        self.state = AppState::Main;
                        self.active_view = ActiveView::Segments;
                    }

                    // --- Mnemonic Quick Action Shortcuts ---
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.welcome_selected_action = 0;
                        let _ = self.handle_input(
                            crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                            tx,
                        );
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.state = AppState::ConfirmMassFontInstallation
                    }
                    KeyCode::Char('i') | KeyCode::Char('I') => {
                        self.welcome_selected_action = 2;
                        let _ = self.handle_input(
                            crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                            tx,
                        );
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        self.state = AppState::Success("Diagnostics coming soon!".to_string())
                    }
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        self.welcome_selected_action = 4;
                        let _ = self.handle_input(
                            crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                            tx,
                        );
                    }
                    KeyCode::Char('b') | KeyCode::Char('B')
                        if !key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        self.welcome_selected_action = 8;
                        let _ = self.handle_input(
                            crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                            tx,
                        );
                    }

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
}
