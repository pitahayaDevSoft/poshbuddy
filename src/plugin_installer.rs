//! Enhanced plugin installation module for PoshBuddy
//!
//! Provides pre-checks, transactions, and automatic rollback
//! for PowerShell module installations.

use std::io;
use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
pub struct InstallResult {
    pub success: bool,
    pub module_name: String,
    pub version: Option<String>,
    pub message: String,
    pub rolled_back: bool,
}

#[allow(dead_code)]
pub struct PreCheckResult {
    pub can_install: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub module_exists: bool,
    pub has_powershell: bool,
    pub has_permissions: bool,
}

impl PreCheckResult {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            can_install: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            module_exists: false,
            has_powershell: false,
            has_permissions: false,
        }
    }

    #[allow(dead_code)]
    pub fn is_ready(&self) -> bool {
        self.can_install && self.errors.is_empty() && self.has_powershell
    }
}

impl Default for PreCheckResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin installer with transactional support
pub struct PluginInstaller;

impl PluginInstaller {
    /// Creates a new installer
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    /// Runs pre-checks before installation
    #[allow(dead_code)]
    pub fn pre_check(&self, module_name: &str) -> PreCheckResult {
        let mut result = PreCheckResult::new();

        // 1. Verify PowerShell is available
        result.has_powershell = Self::check_powershell_available();
        if !result.has_powershell {
            result
                .errors
                .push("PowerShell is not available".to_string());
            result.can_install = false;
            return result;
        }

        // 2. Check if the module is already installed
        match Self::check_module_installed(module_name) {
            Ok(true) => {
                result.module_exists = true;
                result.warnings.push(format!(
                    "Module '{}' is already installed. It will be updated if a new version exists.",
                    module_name
                ));
            }
            Ok(false) => {
                result.module_exists = false;
            }
            Err(e) => {
                result
                    .warnings
                    .push(format!("Could not verify if module exists: {}", e));
            }
        }

        // 3. Check script execution permissions
        match Self::check_execution_policy() {
            Ok(policy) => {
                if crate::app::contains_ignore_ascii_case(&policy, "restricted") {
                    result.errors.push(
                        "PowerShell execution policy is restricted. \
                         Run: Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser"
                            .to_string(),
                    );
                    result.can_install = false;
                } else {
                    result.has_permissions = true;
                }
            }
            Err(e) => {
                result
                    .warnings
                    .push(format!("Could not verify execution policy: {}", e));
            }
        }

        // 4. Verify connectivity to PSGallery (simplified)
        if !Self::check_internet_connectivity() {
            result
                .warnings
                .push("No internet connection detected. Installation might fail.".to_string());
        }

        result
    }

    /// Installs a module with full transaction
    /// Backs up the profile, attempts installation, and rolls back if it fails
    #[allow(dead_code)]
    pub async fn install_with_transaction(
        &self,
        name: &str,
        module_name: &str,
        profile_path: &Path,
        backup_manager: &crate::backup::BackupManager,
    ) -> Result<InstallResult, io::Error> {
        let mut result = InstallResult {
            success: false,
            module_name: module_name.to_string(),
            version: None,
            message: String::new(),
            rolled_back: false,
        };

        // Step 1: Pre-checks
        let pre_check = self.pre_check(module_name);
        if !pre_check.is_ready() {
            result.message = format!("Pre-checks failed: {}", pre_check.errors.join("; "));
            return Ok(result);
        }

        // Step 2: Backup profile before modifying
        let backup_result = if profile_path.exists() {
            backup_manager
                .backup_profile(
                    profile_path,
                    &format!("Pre-installation of plugin: {}", name),
                )
                .ok()
        } else {
            None
        };

        // Step 3: Attempt to install the module
        match self.install_module(module_name).await {
            Ok(version) => {
                result.success = true;
                result.version = version;
                result.message = format!("Module '{}' installed successfully", module_name);
            }
            Err(e) => {
                result.message = format!("Error installing '{}': {}", module_name, e);

                // Step 4: Rollback if installation failed and we have a backup
                if let Some(backup_info) = backup_result {
                    match backup_manager.restore_backup(&backup_info, profile_path) {
                        Ok(_) => {
                            result.rolled_back = true;
                            result.message.push_str(" (Profile restored from backup)");
                        }
                        Err(restore_err) => {
                            result.message.push_str(&format!(
                                " (WARNING: Could not restore backup: {})",
                                restore_err
                            ));
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Installs a PowerShell module
    #[allow(dead_code)]
    async fn install_module(&self, module_name: &str) -> Result<Option<String>, io::Error> {
        let output = tokio::process::Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Install-Module -Name {} -Scope CurrentUser -Force -AllowClobber -Confirm:$false; \
                     (Get-Module -ListAvailable -Name {}).Version | Select-Object -First 1",
                    module_name, module_name
                ),
            ])
            .output()
            .await?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let version = if version.is_empty() {
                None
            } else {
                Some(version)
            };
            Ok(version)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(io::Error::other(format!("PowerShell error: {}", stderr)))
        }
    }

    /// Verifica si PowerShell está disponible
    #[allow(dead_code)]
    fn check_powershell_available() -> bool {
        Command::new("pwsh")
            .args(["-Command", "$PSVersionTable.PSVersion.Major"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or_else(|_| {
                // Intentar con powershell (versión antigua)
                Command::new("powershell")
                    .args(["-Command", "$PSVersionTable.PSVersion.Major"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            })
    }

    /// Checks if a module is already installed
    #[allow(dead_code)]
    fn check_module_installed(module_name: &str) -> Result<bool, io::Error> {
        let output = Command::new("pwsh")
            .args([
                "-Command",
                &format!(
                    "Get-Module -ListAvailable -Name {} | Select-Object -First 1",
                    module_name
                ),
            ])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(!stdout.trim().is_empty())
        } else {
            Err(io::Error::other("Failed to check module status"))
        }
    }

    /// Checks PowerShell execution policy
    #[allow(dead_code)]
    fn check_execution_policy() -> Result<String, io::Error> {
        let output = Command::new("pwsh")
            .args(["-Command", "Get-ExecutionPolicy -Scope CurrentUser"])
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(io::Error::other("Failed to check execution policy"))
        }
    }

    /// Checks basic internet connectivity
    #[allow(dead_code)]
    fn check_internet_connectivity() -> bool {
        // Attempts to ping Google DNS as a simple test
        #[cfg(windows)]
        {
            Command::new("ping")
                .args(["-n", "1", "-w", "1000", "8.8.8.8"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(windows))]
        {
            Command::new("ping")
                .args(["-c", "1", "-W", "1", "8.8.8.8"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    /// Uninstalls a module
    #[allow(dead_code)]
    pub async fn uninstall_module(&self, module_name: &str) -> Result<(), io::Error> {
        let output = tokio::process::Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Uninstall-Module -Name {} -Scope CurrentUser -Force -Confirm:$false",
                    module_name
                ),
            ])
            .output()
            .await?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(io::Error::other(format!("Failed to uninstall: {}", stderr)))
        }
    }

    /// Gets information about an installed module
    #[allow(dead_code)]
    pub fn get_module_info(&self, module_name: &str) -> Result<Option<ModuleInfo>, io::Error> {
        let output = Command::new("pwsh")
            .args([
                "-Command",
                &format!(
                    "Get-Module -ListAvailable -Name {} | Select-Object Name, Version, Description | Format-List",
                    module_name
                ),
            ])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() {
                return Ok(None);
            }

            // Simple output parsing
            let mut name = None;
            let mut version = None;
            let mut description = None;

            for line in stdout.lines() {
                if line.starts_with("Name") {
                    name = line.split(':').nth(1).map(|s| s.trim().to_string());
                } else if line.starts_with("Version") {
                    version = line.split(':').nth(1).map(|s| s.trim().to_string());
                } else if line.starts_with("Description") {
                    description = line.split(':').nth(1).map(|s| s.trim().to_string());
                }
            }

            Ok(Some(ModuleInfo {
                name: name.unwrap_or_default(),
                version: version.unwrap_or_default(),
                description: description.unwrap_or_default(),
            }))
        } else {
            Err(io::Error::other("Failed to get module info"))
        }
    }
}

impl Default for PluginInstaller {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about an installed module
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ModuleInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_check_result() {
        let mut result = PreCheckResult::new();
        result.has_powershell = true;
        assert!(result.is_ready());

        result.has_powershell = true;
        result.can_install = true;
        assert!(result.is_ready());

        result.errors.push("Test error".to_string());
        assert!(!result.is_ready());
    }

    #[test]
    fn test_install_result() {
        let result = InstallResult {
            success: true,
            module_name: "TestModule".to_string(),
            version: Some("1.0.0".to_string()),
            message: "Installed".to_string(),
            rolled_back: false,
        };

        assert!(result.success);
        assert!(!result.rolled_back);
    }

    // Nota: Los tests que requieren PowerShell están marcados como #[ignore]
    // porque pueden no funcionar en todos los entornos

    #[test]
    #[ignore = "Requires PowerShell"]
    fn test_check_powershell_available() {
        assert!(PluginInstaller::check_powershell_available());
    }

    #[test]
    #[ignore = "Requires PowerShell"]
    fn test_pre_check() {
        let installer = PluginInstaller::new();
        let result = installer.pre_check("Pester");
        assert!(result.has_powershell);
    }
}
