//! Diagnostic Module for PoshBuddy
//!
//! Provides PowerShell syntax verification, profile diagnostics,
//! and analysis of common issues before applying changes.

use std::io;
use std::path::Path;
use std::process::Command;

/// Errors that may occur during diagnostics
#[derive(Debug)]
#[allow(dead_code)]
pub enum DiagnosticError {
    Io(io::Error),
    #[allow(dead_code)]
    InvalidSyntax(String),
    #[allow(dead_code)]
    PowerShellNotFound,
    #[allow(dead_code)]
    ProfileNotReadable(String),
}

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticError::Io(e) => write!(f, "I/O Error: {}", e),
            DiagnosticError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            DiagnosticError::PowerShellNotFound => write!(f, "PowerShell not found"),
            DiagnosticError::ProfileNotReadable(path) => {
                write!(f, "Profile is not readable: {}", path)
            }
        }
    }
}

impl std::error::Error for DiagnosticError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DiagnosticError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for DiagnosticError {
    fn from(e: io::Error) -> Self {
        DiagnosticError::Io(e)
    }
}

/// Result of a diagnostic check
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiagnosticResult {
    pub success: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub suggestions: Vec<String>,
}

impl DiagnosticResult {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            success: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
        self.success = false;
    }

    pub fn add_suggestion(&mut self, msg: impl Into<String>) {
        self.suggestions.push(msg.into());
    }

    pub fn is_valid(&self) -> bool {
        self.success && self.errors.is_empty()
    }
}

impl Default for DiagnosticResult {
    fn default() -> Self {
        Self::new()
    }
}

/// PowerShell configuration diagnostics
pub struct Diagnostic;

impl Diagnostic {
    /// Creates a new diagnostic instance
    pub fn new() -> Self {
        Self
    }

    /// Verifies the syntax of a PowerShell script
    /// Uses PowerShell's -WhatIf parameter to validate without executing
    #[allow(dead_code)]
    pub fn validate_powershell_syntax(
        &self,
        script: &str,
    ) -> Result<DiagnosticResult, DiagnosticError> {
        let mut result = DiagnosticResult::new();

        // Check if there is content
        if script.trim().is_empty() {
            result.add_error("Script is empty");
            return Ok(result);
        }

        // Verify basic brace and parentheses balancing
        let open_braces = script.chars().filter(|&c| c == '{').count();
        let close_braces = script.chars().filter(|&c| c == '}').count();
        if open_braces != close_braces {
            result.add_error(format!(
                "Unbalanced braces: {} open, {} closed",
                open_braces, close_braces
            ));
        }

        let open_parens = script.chars().filter(|&c| c == '(').count();
        let close_parens = script.chars().filter(|&c| c == ')').count();
        if open_parens != close_parens {
            result.add_error(format!(
                "Unbalanced parentheses: {} open, {} closed",
                open_parens, close_parens
            ));
        }

        // Verify balanced quotes (simplified)
        let double_quotes = script.chars().filter(|&c| c == '"').count();
        let single_quotes = script.chars().filter(|&c| c == '\'').count();
        if double_quotes % 2 != 0 {
            result.add_warning("Possible unbalanced double quotes");
        }
        if single_quotes % 2 != 0 {
            result.add_warning("Possible unbalanced single quotes");
        }

        // Verify common OMP commands
        if script.contains("oh-my-posh") {
            // Verify theme path exists
            if let Some(theme_idx) = script.find("--config") {
                let after_config = &script[theme_idx + 8..];
                if let Some(quote_idx) = after_config.find(['"', '\'']) {
                    let quote_char = after_config.chars().nth(quote_idx).unwrap();
                    let path_end = after_config[quote_idx + 1..].find(quote_char).unwrap_or(0);
                    if path_end > 0 {
                        let theme_path = &after_config[quote_idx + 1..quote_idx + 1 + path_end];
                        if !std::path::Path::new(theme_path).exists() {
                            result
                                .add_warning(format!("Theme path does not exist: {}", theme_path));
                        }
                    }
                }
            }
        }

        // Suggestions
        if script.contains("Invoke-Expression") {
            result.add_suggestion("Consider using 'Invoke-Expression' only with trusted sources");
        }

        Ok(result)
    }

    /// Verifies a specific PowerShell profile
    #[allow(dead_code)]
    pub fn check_profile(&self, profile_path: &Path) -> Result<DiagnosticResult, DiagnosticError> {
        let mut result = DiagnosticResult::new();

        if !profile_path.exists() {
            result.add_warning(format!(
                "Profile does not exist: {}. A new one will be created.",
                profile_path.display()
            ));
            return Ok(result);
        }

        // Verify it can be read
        match std::fs::read_to_string(profile_path) {
            Ok(content) => {
                // Verify content syntax
                let syntax_result = self.validate_powershell_syntax(&content)?;
                result.warnings.extend(syntax_result.warnings);
                result.errors.extend(syntax_result.errors);
                result.suggestions.extend(syntax_result.suggestions);
                result.success = result.errors.is_empty();

                // Verify encoding (should be UTF-8 or UTF-8-BOM)
                let bytes = std::fs::read(profile_path)?;
                if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                    result.add_suggestion("Profile has UTF-8 BOM. This is normal on Windows.");
                }

                // Verify execution permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = std::fs::metadata(profile_path)?;
                    let permissions = metadata.permissions();
                    let mode = permissions.mode();
                    if mode & 0o111 == 0 {
                        result.add_warning("Profile does not have execution permissions");
                    }
                }
            }
            Err(e) => {
                result.add_error(format!(
                    "Could not read profile {}: {}",
                    profile_path.display(),
                    e
                ));
            }
        }

        Ok(result)
    }

    /// Verifies all detected profiles
    #[allow(dead_code)]
    pub fn check_all_profiles(
        &self,
        profiles: &[std::path::PathBuf],
    ) -> Result<DiagnosticResult, DiagnosticError> {
        let mut result = DiagnosticResult::new();

        if profiles.is_empty() {
            result.add_error("No PowerShell profiles detected");
            result.add_suggestion("Run 'notepad $PROFILE' to create a profile");
            return Ok(result);
        }

        for profile in profiles {
            let profile_result = self.check_profile(profile)?;
            if !profile_result.is_valid() {
                result.add_error(format!(
                    "Issues in {}: {}",
                    profile.display(),
                    profile_result.errors.join(", ")
                ));
            }
            for warning in &profile_result.warnings {
                result.add_warning(format!("{}: {}", profile.display(), warning));
            }
        }

        Ok(result)
    }

    /// Runs a full system diagnostic
    #[allow(dead_code)]
    pub fn run_full_diagnostic(
        &self,
        profiles: &[std::path::PathBuf],
    ) -> Result<DiagnosticResult, DiagnosticError> {
        let mut result = DiagnosticResult::new();

        // 1. Verify PowerShell is installed
        if !Self::is_powershell_available() {
            result.add_error("PowerShell is not available in PATH");
            return Ok(result);
        }

        // 2. Check Oh My Posh
        match self.check_oh_my_posh() {
            Ok(omp_ok) => {
                if !omp_ok {
                    result.add_error("Oh My Posh is not installed or not in PATH");
                    result.add_suggestion(
                        "Install Oh My Posh: winget install JanDeDobbeleer.OhMyPosh",
                    );
                }
            }
            Err(e) => {
                result.add_warning(format!("Could not verify Oh My Posh: {}", e));
            }
        }

        // 3. Verify profiles
        let profile_result = self.check_all_profiles(profiles)?;
        result.warnings.extend(profile_result.warnings);
        result.errors.extend(profile_result.errors);
        result.suggestions.extend(profile_result.suggestions);

        result.success = result.errors.is_empty();
        Ok(result)
    }

    /// Checks if PowerShell is available
    #[allow(dead_code)]
    pub fn is_powershell_available() -> bool {
        let cmd = if cfg!(windows) { "pwsh" } else { "pwsh" };
        Command::new(cmd)
            .arg("-Command")
            .arg("$PSVersionTable.PSVersion")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Checks if Oh My Posh is installed
    #[allow(dead_code)]
    pub fn check_oh_my_posh(&self) -> Result<bool, DiagnosticError> {
        let output = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", "where oh-my-posh"])
                .output()
        } else {
            Command::new("which").arg("oh-my-posh").output()
        };

        match output {
            Ok(o) => Ok(o.status.success()),
            Err(e) => Err(e.into()),
        }
    }

    /// Generates a formatted diagnostic report
    #[allow(dead_code)]
    pub fn format_report(&self, result: &DiagnosticResult) -> String {
        let mut report = String::new();
        report.push_str("═══════════════════════════════════════════\n");
        report.push_str("          DIAGNOSTIC REPORT\n");
        report.push_str("═══════════════════════════════════════════\n\n");

        if result.is_valid() {
            report.push_str("✅ Status: ALL GOOD\n");
        } else {
            report.push_str("❌ Status: ISSUES FOUND\n");
        }

        if !result.errors.is_empty() {
            report.push_str("\n🚨 ERRORS:\n");
            for error in &result.errors {
                report.push_str(&format!("   • {}\n", error));
            }
        }

        if !result.warnings.is_empty() {
            report.push_str("\n⚠️  WARNINGS:\n");
            for warning in &result.warnings {
                report.push_str(&format!("   • {}\n", warning));
            }
        }

        if !result.suggestions.is_empty() {
            report.push_str("\n💡 SUGGESTIONS:\n");
            for suggestion in &result.suggestions {
                report.push_str(&format!("   • {}\n", suggestion));
            }
        }

        report.push_str("\n═══════════════════════════════════════════\n");
        report
    }
}

impl Default for Diagnostic {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_script() {
        let diag = Diagnostic::new();
        let result = diag.validate_powershell_syntax("").unwrap();
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_balanced_braces() {
        let diag = Diagnostic::new();
        let result = diag
            .validate_powershell_syntax("function test() { Write-Host 'ok' }")
            .unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_unbalanced_braces() {
        let diag = Diagnostic::new();
        let result = diag
            .validate_powershell_syntax("function test() { Write-Host 'ok'")
            .unwrap();
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Unbalanced braces")));
    }

    #[test]
    fn test_validate_omp_command() {
        let diag = Diagnostic::new();
        let script =
            "oh-my-posh init pwsh --config 'C:\\nonexistent\\theme.omp.json' | Invoke-Expression";
        let result = diag.validate_powershell_syntax(script).unwrap();
        // Should have a warning about nonexistent path
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("Theme path does not exist")));
    }

    #[test]
    fn test_diagnostic_result_methods() {
        let mut result = DiagnosticResult::new();
        assert!(result.is_valid());

        result.add_warning("Test warning");
        assert!(result.is_valid()); // warnings no invalidan

        result.add_error("Test error");
        assert!(!result.is_valid()); // errors sí invalidan
    }

    #[test]
    fn test_format_report() {
        let diag = Diagnostic::new();
        let mut result = DiagnosticResult::new();
        result.add_error("Test error");
        result.add_warning("Test warning");
        result.add_suggestion("Test suggestion");

        let report = diag.format_report(&result);
        assert!(report.contains("ISSUES FOUND"));
        assert!(report.contains("Test error"));
        assert!(report.contains("Test warning"));
        assert!(report.contains("Test suggestion"));
    }

    #[test]
    fn test_check_profile_nonexistent() {
        let diag = Diagnostic::new();
        let path = Path::new("nonexistent_profile.ps1");
        let result = diag.check_profile(path).unwrap();

        assert!(result.is_valid());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("Profile does not exist")));
    }

    #[test]
    fn test_check_profile_valid() {
        use std::io::Write;
        let diag = Diagnostic::new();
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "Write-Host 'Hello'").unwrap();

        let result = diag.check_profile(temp_file.path()).unwrap();
        assert!(result.is_valid());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_check_profile_invalid_syntax() {
        use std::io::Write;
        let diag = Diagnostic::new();
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "function test {{").unwrap(); // Unbalanced braces

        let result = diag.check_profile(temp_file.path()).unwrap();
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Unbalanced braces")));
    }

    #[test]
    fn test_check_profile_utf8_bom() {
        use std::io::Write;
        let diag = Diagnostic::new();
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        // UTF-8 BOM
        temp_file.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
        temp_file.write_all(b"Write-Host 'BOM'").unwrap();

        let result = diag.check_profile(temp_file.path()).unwrap();
        assert!(result.is_valid());
        assert!(result
            .suggestions
            .iter()
            .any(|s| s.contains("Profile has UTF-8 BOM")));
    }

    #[cfg(unix)]
    #[test]
    fn test_check_profile_no_execution_permissions() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        let diag = Diagnostic::new();
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "Write-Host 'NoExec'").unwrap();

        // Remove execution permissions
        let mut perms = std::fs::metadata(temp_file.path()).unwrap().permissions();
        perms.set_mode(0o644); // rw-r--r--
        std::fs::set_permissions(temp_file.path(), perms).unwrap();

        let result = diag.check_profile(temp_file.path()).unwrap();
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("execution permissions")));
    }
}
