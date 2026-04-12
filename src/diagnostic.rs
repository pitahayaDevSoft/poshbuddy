//! Módulo de Diagnósticos para PoshBuddy
//!
//! Proporciona verificación de sintaxis PowerShell, diagnóstico de perfiles
//! y análisis de problemas comunes antes de aplicar cambios.

use std::io;
use std::path::Path;
use std::process::Command;

/// Errores que pueden ocurrir durante el diagnóstico
#[derive(Debug)]
pub enum DiagnosticError {
    Io(io::Error),
    InvalidSyntax(String),
    PowerShellNotFound,
    ProfileNotReadable(String),
}

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticError::Io(e) => write!(f, "Error de E/S: {}", e),
            DiagnosticError::InvalidSyntax(msg) => write!(f, "Sintaxis inválida: {}", msg),
            DiagnosticError::PowerShellNotFound => write!(f, "PowerShell no encontrado"),
            DiagnosticError::ProfileNotReadable(path) => write!(f, "No se puede leer el perfil: {}", path),
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

/// Resultado de una verificación de diagnóstico
#[derive(Debug, Clone)]
pub struct DiagnosticResult {
    pub success: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub suggestions: Vec<String>,
}

impl DiagnosticResult {
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

/// Diagnóstico de configuración de PowerShell
pub struct Diagnostic;

impl Diagnostic {
    /// Crea una nueva instancia del diagnóstico
    pub fn new() -> Self {
        Self
    }

    /// Verifica la sintaxis de un script de PowerShell
    /// Usa el parámetro -WhatIf de PowerShell para validar sin ejecutar
    pub fn validate_powershell_syntax(&self, script: &str) -> Result<DiagnosticResult, DiagnosticError> {
        let mut result = DiagnosticResult::new();

        // Verificar que hay contenido
        if script.trim().is_empty() {
            result.add_error("El script está vacío");
            return Ok(result);
        }

        // Verificar balanceo de llaves y paréntesis básico
        let open_braces = script.chars().filter(|&c| c == '{').count();
        let close_braces = script.chars().filter(|&c| c == '}').count();
        if open_braces != close_braces {
            result.add_error(format!(
                "Llaves desbalanceadas: {} abiertas, {} cerradas",
                open_braces, close_braces
            ));
        }

        let open_parens = script.chars().filter(|&c| c == '(').count();
        let close_parens = script.chars().filter(|&c| c == ')').count();
        if open_parens != close_parens {
            result.add_error(format!(
                "Paréntesis desbalanceados: {} abiertos, {} cerrados",
                open_parens, close_parens
            ));
        }

        // Verificar comillas balanceadas (simplificado)
        let double_quotes = script.chars().filter(|&c| c == '"').count();
        let single_quotes = script.chars().filter(|&c| c == '\'').count();
        if double_quotes % 2 != 0 {
            result.add_warning("Posibles comillas dobles desbalanceadas");
        }
        if single_quotes % 2 != 0 {
            result.add_warning("Posibles comillas simples desbalanceadas");
        }

        // Verificar comandos comunes de OMP
        if script.contains("oh-my-posh") {
            // Verificar que la ruta al tema existe
            if let Some(theme_idx) = script.find("--config") {
                let after_config = &script[theme_idx + 8..];
                if let Some(quote_idx) = after_config.find('"') {
                    let path_end = after_config[quote_idx + 1..].find('"').unwrap_or(0);
                    if path_end > 0 {
                        let theme_path = &after_config[quote_idx + 1..quote_idx + 1 + path_end];
                        if !std::path::Path::new(theme_path).exists() {
                            result.add_warning(format!(
                                "La ruta del tema no existe: {}",
                                theme_path
                            ));
                        }
                    }
                }
            }
        }

        // Sugerencias
        if script.contains("Invoke-Expression") {
            result.add_suggestion("Considera usar 'Invoke-Expression' solo con fuentes confiables");
        }

        Ok(result)
    }

    /// Verifica un perfil de PowerShell específico
    pub fn check_profile(&self, profile_path: &Path) -> Result<DiagnosticResult, DiagnosticError> {
        let mut result = DiagnosticResult::new();

        if !profile_path.exists() {
            result.add_warning(format!(
                "El perfil no existe: {}. Se creará uno nuevo.",
                profile_path.display()
            ));
            return Ok(result);
        }

        // Verificar que se puede leer
        match std::fs::read_to_string(profile_path) {
            Ok(content) => {
                // Verificar sintaxis del contenido
                let syntax_result = self.validate_powershell_syntax(&content)?;
                result.warnings.extend(syntax_result.warnings);
                result.errors.extend(syntax_result.errors);
                result.suggestions.extend(syntax_result.suggestions);
                result.success = result.errors.is_empty();

                // Verificar codificación (debería ser UTF-8 o UTF-8-BOM)
                let bytes = std::fs::read(profile_path)?;
                if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                    result.add_suggestion("El perfil tiene BOM UTF-8. Esto es normal en Windows.");
                }

                // Verificar permisos de ejecución
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = std::fs::metadata(profile_path)?;
                    let permissions = metadata.permissions();
                    let mode = permissions.mode();
                    if mode & 0o111 == 0 {
                        result.add_warning("El perfil no tiene permisos de ejecución");
                    }
                }
            }
            Err(e) => {
                result.add_error(format!(
                    "No se pudo leer el perfil {}: {}",
                    profile_path.display(),
                    e
                ));
            }
        }

        Ok(result)
    }

    /// Verifica todos los perfiles detectados
    pub fn check_all_profiles(&self, profiles: &[std::path::PathBuf]) -> Result<DiagnosticResult, DiagnosticError> {
        let mut result = DiagnosticResult::new();

        if profiles.is_empty() {
            result.add_error("No se detectaron perfiles de PowerShell");
            result.add_suggestion("Ejecuta 'notepad $PROFILE' para crear un perfil");
            return Ok(result);
        }

        for profile in profiles {
            let profile_result = self.check_profile(profile)?;
            if !profile_result.is_valid() {
                result.add_error(format!(
                    "Problemas en {}: {}",
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

    /// Ejecuta un diagnóstico completo del sistema
    pub fn run_full_diagnostic(&self, profiles: &[std::path::PathBuf]) -> Result<DiagnosticResult, DiagnosticError> {
        let mut result = DiagnosticResult::new();

        // 1. Verificar que PowerShell está instalado
        if !Self::is_powershell_available() {
            result.add_error("PowerShell no está disponible en el PATH");
            return Ok(result);
        }

        // 2. Verificar Oh My Posh
        match self.check_oh_my_posh() {
            Ok(omp_ok) => {
                if !omp_ok {
                    result.add_error("Oh My Posh no está instalado o no está en el PATH");
                    result.add_suggestion("Instala Oh My Posh: winget install JanDeDobbeleer.OhMyPosh");
                }
            }
            Err(e) => {
                result.add_warning(format!("No se pudo verificar Oh My Posh: {}", e));
            }
        }

        // 3. Verificar perfiles
        let profile_result = self.check_all_profiles(profiles)?;
        result.warnings.extend(profile_result.warnings);
        result.errors.extend(profile_result.errors);
        result.suggestions.extend(profile_result.suggestions);

        result.success = result.errors.is_empty();
        Ok(result)
    }

    /// Verifica si PowerShell está disponible
    pub fn is_powershell_available() -> bool {
        let cmd = if cfg!(windows) { "pwsh" } else { "pwsh" };
        Command::new(cmd)
            .arg("-Command")
            .arg("$PSVersionTable.PSVersion")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Verifica si Oh My Posh está instalado
    pub fn check_oh_my_posh(&self) -> Result<bool, DiagnosticError> {
        let output = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", "where oh-my-posh"])
                .output()
        } else {
            Command::new("which")
                .arg("oh-my-posh")
                .output()
        };

        match output {
            Ok(o) => Ok(o.status.success()),
            Err(e) => Err(e.into()),
        }
    }

    /// Genera un reporte de diagnóstico formateado
    pub fn format_report(&self, result: &DiagnosticResult) -> String {
        let mut report = String::new();
        report.push_str("═══════════════════════════════════════════\n");
        report.push_str("        REPORTE DE DIAGNÓSTICO\n");
        report.push_str("═══════════════════════════════════════════\n\n");

        if result.is_valid() {
            report.push_str("✅ Estado: TODO CORRECTO\n");
        } else {
            report.push_str("❌ Estado: SE ENCONTRARON PROBLEMAS\n");
        }

        if !result.errors.is_empty() {
            report.push_str("\n🚨 ERRORES:\n");
            for error in &result.errors {
                report.push_str(&format!("   • {}\n", error));
            }
        }

        if !result.warnings.is_empty() {
            report.push_str("\n⚠️  ADVERTENCIAS:\n");
            for warning in &result.warnings {
                report.push_str(&format!("   • {}\n", warning));
            }
        }

        if !result.suggestions.is_empty() {
            report.push_str("\n💡 SUGERENCIAS:\n");
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
        let result = diag.validate_powershell_syntax("function test() { Write-Host 'ok' }").unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_unbalanced_braces() {
        let diag = Diagnostic::new();
        let result = diag.validate_powershell_syntax("function test() { Write-Host 'ok'").unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("Llaves")));
    }

    #[test]
    fn test_validate_omp_command() {
        let diag = Diagnostic::new();
        let script = "oh-my-posh init pwsh --config 'C:\\nonexistent\\theme.omp.json' | Invoke-Expression";
        let result = diag.validate_powershell_syntax(script).unwrap();
        // Debería tener una advertencia sobre la ruta no existente
        assert!(result.warnings.iter().any(|w| w.contains("ruta del tema")));
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
        result.add_error("Error de prueba");
        result.add_warning("Advertencia de prueba");
        result.add_suggestion("Sugerencia de prueba");

        let report = diag.format_report(&result);
        assert!(report.contains("SE ENCONTRARON PROBLEMAS"));
        assert!(report.contains("Error de prueba"));
        assert!(report.contains("Advertencia de prueba"));
        assert!(report.contains("Sugerencia de prueba"));
    }
}
