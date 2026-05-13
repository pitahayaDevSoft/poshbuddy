use crate::app::models::*;
use crate::app::{OMP_BINARY, WHERE_CMD, contains_ignore_ascii_case};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;
use tokio::sync::mpsc;

impl App {
    /// Reads the current configuration file once and caches active segment types
    pub fn refresh_active_segments(&mut self) {
        let path = if let Some(p) = &self.active_config_path {
            p
        } else {
            self.active_segments.clear();
            return;
        };

        let mut active = HashSet::new();
        if let Ok(content) = fs::read_to_string(path)
            && let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
        {
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
        if let Ok(term_prog) = std::env::var("TERM_PROGRAM")
            && term_prog == "vscode"
        {
            return true;
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

    /// Returns a count of filtered themes without allocating a Vec
    pub fn filtered_themes_count(&self) -> usize {
        let filter = &self.filter;
        let local_count = self
            .themes
            .iter()
            .filter(|t| contains_ignore_ascii_case(&t.name, filter))
            .count();
        let remote_count = self
            .remote_themes
            .iter()
            .filter(|rt| {
                contains_ignore_ascii_case(&rt.name, filter)
                    && self
                        .themes
                        .binary_search_by(|t| t.name.cmp(&rt.name))
                        .is_err()
            })
            .count();
        local_count + remote_count
    }

    /// Returns a specific filtered theme without allocating a full Vec
    pub fn filtered_theme_at(&self, index: usize) -> Option<ThemeAsset> {
        let filter = &self.filter;

        let mut current_idx = 0;

        // Search Local
        for t in &self.themes {
            if contains_ignore_ascii_case(&t.name, filter) {
                if current_idx == index {
                    return Some(t.clone());
                }
                current_idx += 1;
            }
        }

        // Search Remote (only if not local)
        for rt in &self.remote_themes {
            if contains_ignore_ascii_case(&rt.name, filter)
                && self
                    .themes
                    .binary_search_by(|t| t.name.cmp(&rt.name))
                    .is_err()
            {
                if current_idx == index {
                    return Some(ThemeAsset {
                        name: rt.name.clone(),
                        is_local: false,
                        download_url: Some(rt.download_url.clone()),
                    });
                }
                current_idx += 1;
            }
        }

        None
    }

    /// Returns a specific filtered font without allocating a full Vec
    pub fn filtered_font_at(&self, index: usize) -> Option<FontAsset> {
        self.fonts
            .iter()
            .filter(|f| contains_ignore_ascii_case(&f.name, &self.fonts_filter))
            .nth(index)
            .cloned()
    }

    /// Returns a specific filtered segment without allocating a full Vec
    pub fn filtered_segment_at(&self, index: usize) -> Option<SegmentAsset> {
        self.segments
            .iter()
            .filter(|p| {
                contains_ignore_ascii_case(&p.name, &self.segments_filter)
                    || contains_ignore_ascii_case(&p.description, &self.segments_filter)
                    || contains_ignore_ascii_case(&p.category, &self.segments_filter)
            })
            .nth(index)
            .cloned()
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
                && self
                    .themes
                    .binary_search_by(|t| t.name.cmp(&rt.name))
                    .is_err()
            {
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

    /// Returns a count of filtered fonts without allocating a Vec
    pub fn filtered_fonts_count(&self) -> usize {
        self.fonts
            .iter()
            .filter(|f| contains_ignore_ascii_case(&f.name, &self.fonts_filter))
            .count()
    }

    /// Returns the count of filtered segments without allocating a new Vec
    pub fn filtered_segments_count(&self) -> usize {
        self.segments
            .iter()
            .filter(|p| {
                contains_ignore_ascii_case(&p.name, &self.segments_filter)
                    || contains_ignore_ascii_case(&p.description, &self.segments_filter)
                    || contains_ignore_ascii_case(&p.category, &self.segments_filter)
            })
            .count()
    }

    /// Checks if a segment is active in the currently loaded Oh My Posh config
    pub fn is_segment_active(&self, segment: &SegmentAsset) -> bool {
        self.active_segments.contains(&segment.segment_type)
    }

    /// Surgical JSON edit to toggle a segment in the active Oh My Posh theme.
    /// It searches for the segment across all blocks and top-level segments.
    /// If found, it removes all instances (deactivation).
    /// If not found, it adds it to the first available block (activation).
    pub fn toggle_segment(&mut self, segment: &SegmentAsset) -> io::Result<()> {
        let path = self
            .active_config_path
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No active config found"))?;

        let content = fs::read_to_string(path)?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        let mut found_any = false;

        // 1. Remove from top-level segments if present
        if let Some(segments) = json.get_mut("segments").and_then(|v| v.as_array_mut()) {
            let initial_len = segments.len();
            segments
                .retain(|s| s.get("type").and_then(|v| v.as_str()) != Some(&segment.segment_type));
            if segments.len() < initial_len {
                found_any = true;
            }
        }

        // 2. Remove from all blocks if present
        if let Some(blocks) = json.get_mut("blocks").and_then(|v| v.as_array_mut()) {
            for block in blocks.iter_mut() {
                if let Some(segments) = block.get_mut("segments").and_then(|v| v.as_array_mut()) {
                    let initial_len = segments.len();
                    segments.retain(|s| {
                        s.get("type").and_then(|v| v.as_str()) != Some(&segment.segment_type)
                    });
                    if segments.len() < initial_len {
                        found_any = true;
                    }
                }
            }
        }

        // 3. If not found, add to the first block (or create one if missing)
        if !found_any {
            let new_segment = serde_json::json!({
                "type": segment.segment_type,
                "style": "powerline",
                "powerline_symbol": "\u{e0b0}",
                "foreground": "#ffffff",
                "background": "#61afef",
                "template": format!(" {} ", segment.segment_type)
            });

            // Ensure "blocks" exists and has at least one element
            if json.get("blocks").is_none() {
                json["blocks"] = serde_json::json!([]);
            }

            let blocks = json
                .get_mut("blocks")
                .and_then(|v| v.as_array_mut())
                .unwrap();
            if blocks.is_empty() {
                blocks.push(serde_json::json!({
                    "type": "prompt",
                    "alignment": "left",
                    "segments": []
                }));
            }

            if let Some(segments) = blocks[0].get_mut("segments").and_then(|v| v.as_array_mut()) {
                segments.push(new_segment);
            } else {
                // If the block exists but segments field is missing
                blocks[0]["segments"] = serde_json::json!([new_segment]);
            }
        }

        let new_json =
            serde_json::to_string_pretty(&json).map_err(|e| io::Error::other(e.to_string()))?;
        fs::write(path, new_json)?;
        self.refresh_active_segments(); // Update cache after write
        Ok(())
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
        if !self.fonts.iter().any(|f| f.name == font_name) {
            let _ = tx.try_send(AppMessage::Error(format!(
                "Invalid font name: {}",
                font_name
            )));
            return;
        }

        let cmd = OMP_BINARY;

        let font_name_cloned = font_name.clone();
        tokio::spawn(async move {
            let output = tokio::process::Command::new(cmd)
                .args(["font", "install", &font_name_cloned])
                .output()
                .await;

            match output {
                Ok(_) => {
                    if tx
                        .send(AppMessage::FontInstalled(font_name_cloned))
                        .await
                        .is_err()
                    {}
                }
                Err(e) => if tx.send(AppMessage::Error(e.to_string())).await.is_err() {},
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
                    })
                    .await
                    .is_err()
                {
                    return;
                }

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
                        )))
                        .await
                        .is_err()
                    {
                        return;
                    }
                    return;
                }
            }

            if tx
                .send(AppMessage::Success(
                    "All Nerd Fonts have been installed successfully!".to_string(),
                ))
                .await
                .is_err()
            {}
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
                                })
                                .await
                                .is_err()
                            {
                                return;
                            }
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
                            })
                            .await
                            .is_err()
                        {
                            return;
                        }
                        return;
                    }
                    final_theme_path
                }
            };

            let mut cmd_obj = tokio::process::Command::new(cmd);

            // Get current working directory for a more realistic preview
            let current_dir =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

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
                        })
                        .await
                        .is_err()
                    {}
                }
                Ok(Err(e)) => {
                    if tx
                        .send(AppMessage::ThemePreviewLoaded {
                            theme: theme_cloned,
                            preview: format!(" Command error: {}", e),
                            request_id: current_id,
                        })
                        .await
                        .is_err()
                    {}
                }
                Err(_) => {
                    if tx
                        .send(AppMessage::ThemePreviewLoaded {
                            theme: theme_cloned,
                            preview: " Timeout: Theme too complex for quick preview ".to_string(),
                            request_id: current_id,
                        })
                        .await
                        .is_err()
                    {}
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
                })
                .await
                .is_err()
            {
                return;
            }

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
                            ))
                            .await
                            .is_err()
                        {}
                    }
                },
                Err(e) => {
                    if tx
                        .send(AppMessage::Error(format!(
                            "Could not start installer: {}",
                            e
                        )))
                        .await
                        .is_err()
                    {}
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
            if let Ok(content) = fs::read_to_string(profile)
                && content
                    .lines()
                    .any(|line| Self::is_plugin_line(line, plugin))
            {
                return true;
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
    async fn download_or_locate_theme(
        theme: &ThemeAsset,
        themes_dir: &std::path::Path,
    ) -> Result<PathBuf, String> {
        if theme.is_local {
            let name_clean = theme.name.replace(".omp.json", "");
            Ok(themes_dir.join(format!("{}.omp.json", name_clean)))
        } else {
            if !crate::api::check_internet_connectivity() {
                return Err("No internet connection detected. Check your network.".to_string());
            }

            if let Some(url) = &theme.download_url {
                let temp_dir = std::env::temp_dir();
                match crate::api::download_theme_file(&theme.name, url, &temp_dir).await {
                    Ok(p) => Ok(p),
                    Err(e) => Err(format!("Download failed: {}", e)),
                }
            } else {
                Err("Missing download URL for remote theme".to_string())
            }
        }
    }

    async fn verify_theme_json(source_path: &std::path::Path) -> Result<(), String> {
        match tokio::fs::read_to_string(source_path).await {
            Ok(content) => {
                if serde_json::from_str::<serde_json::Value>(&content).is_err() {
                    Err("Invalid theme JSON format".to_string())
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(format!("Could not read theme file: {}", e)),
        }
    }

    fn backup_profiles(
        profiles: &[PathBuf],
        backup_manager: &crate::backup::BackupManager,
        theme_name: &str,
    ) -> Result<(), String> {
        for profile in profiles {
            if let Err(e) = backup_manager
                .backup_profile(profile, &format!("Apply Theme Advanced: {}", theme_name))
            {
                return Err(format!("Backup failed: {}", e));
            }
        }
        Ok(())
    }

    async fn update_profiles_with_theme(
        profiles: &[PathBuf],
        theme_name: &str,
        config_line: &str,
    ) -> Result<(), String> {
        let start_marker = "## POSHBUDDY AUTO-GENERATED CONFIG - START (THEME)";
        let end_marker = "## POSHBUDDY AUTO-GENERATED CONFIG - END (THEME)";
        let line_ending = if cfg!(windows) { "\r\n" } else { "\n" };

        for profile in profiles {
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
                    new_lines.push(format!(
                        "## Description: Apply Oh My Posh theme: {}",
                        theme_name
                    ));
                    new_lines.push(config_line.to_string());
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
                new_lines.push(format!(
                    "## Description: Apply Oh My Posh theme: {}",
                    theme_name
                ));
                new_lines.push(config_line.to_string());
                new_lines.push(end_marker.to_string());
            }

            if let Err(e) = tokio::fs::write(profile, new_lines.join(line_ending)).await {
                return Err(format!("Profile update failed: {}", e));
            }
        }
        Ok(())
    }

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
                })
                .await
                .is_err()
            {
                return;
            }
            let source_path = match Self::download_or_locate_theme(&theme, &themes_dir).await {
                Ok(path) => path,
                Err(e) => {
                    let _ = tx_cloned.send(AppMessage::Error(e)).await;
                    return;
                }
            };

            // Stage 1: Verify (Try to parse as JSON)
            if tx_cloned
                .send(AppMessage::InstallUpdate {
                    stage: 1,
                    percentage: 50.0,
                })
                .await
                .is_err()
            {
                return;
            }
            if let Err(e) = Self::verify_theme_json(&source_path).await {
                let _ = tx_cloned.send(AppMessage::Error(e)).await;
                return;
            }

            // Stage 2: Backup
            if tx_cloned
                .send(AppMessage::InstallUpdate {
                    stage: 2,
                    percentage: 75.0,
                })
                .await
                .is_err()
            {
                return;
            }
            if let Err(e) = Self::backup_profiles(&profiles, &backup_manager, &name) {
                let _ = tx_cloned.send(AppMessage::Error(e)).await;
                return;
            }

            // Stage 3: Apply
            if tx_cloned
                .send(AppMessage::InstallUpdate {
                    stage: 3,
                    percentage: 90.0,
                })
                .await
                .is_err()
            {
                return;
            }

            let final_theme_path = if !theme.is_local {
                let dest = themes_dir.join(format!("{}.omp.json", theme.name));
                if let Err(e) = tokio::fs::copy(&source_path, &dest).await {
                    let _ = tx_cloned
                        .send(AppMessage::Error(format!("Failed to save theme: {}", e)))
                        .await;
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

            if let Err(e) = Self::update_profiles_with_theme(&profiles, &name, &config_line).await {
                let _ = tx_cloned.send(AppMessage::Error(e)).await;
                return;
            }

            if tx_cloned
                .send(AppMessage::Success(format!(
                    "Theme '{}' applied successfully!",
                    name
                )))
                .await
                .is_err()
            {
                return;
            }
            let _ = tx_cloned
                .send(AppMessage::ThemeDownloaded(final_theme_path))
                .await;
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
                })
                .await
                .is_err()
            {
                return;
            }

            let output = tokio::process::Command::new("powershell")
                .env("POSHBUDDY_MODULE_NAME", &module_name)
                .args([
                    "-Command",
                    "Install-Module -Name $env:POSHBUDDY_MODULE_NAME -Scope CurrentUser -Force -Confirm:$false",
                ])
                .output()
                .await;

            match output {
                Ok(out) if out.status.success() => {
                    if tx.send(AppMessage::SegmentToggled(name)).await.is_err() {}
                }
                _ => {
                    if tx
                        .send(AppMessage::Error(format!(
                            "Failed to install module {}",
                            module_name
                        )))
                        .await
                        .is_err()
                    {}
                }
            }
        });
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
            if profile.exists()
                && let Err(e) = self
                    .backup_manager
                    .backup_profile(profile, "Manual backup from PoshBuddy")
            {
                errors.push(format!("{}: {}", profile.display(), e));
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
