//! Sistema de respaldo automático para archivos de configuración de PowerShell
//!
//! Este módulo proporciona:
//! - Creación automática de backups antes de modificaciones
//! - Restauración de versiones anteriores
//! - Límite de backups por archivo (evita acumulación infinita)
//! - Metadatos de backup (timestamp, motivo, estado)

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Información sobre un backup específico
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub original_path: PathBuf,
    pub timestamp: u64,
    #[allow(dead_code)]
    pub size_bytes: u64,
    #[allow(dead_code)]
    pub description: String,
}

/// Gestor de backups para perfiles de PowerShell
#[derive(Clone)]
pub struct BackupManager {
    backup_dir: PathBuf,
    max_backups_per_file: usize,
}

/// Errores que pueden ocurrir durante operaciones de backup
#[derive(Debug)]
pub enum BackupError {
    Io(io::Error),
    ProfileNotFound(PathBuf),
    BackupNotFound(PathBuf),
    #[allow(dead_code)]
    InvalidBackupName(String),
    RestoreFailed {
        backup: PathBuf,
        target: PathBuf,
        source: io::Error,
    },
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupError::Io(e) => write!(f, "Error de E/S: {}", e),
            BackupError::ProfileNotFound(p) => write!(f, "Perfil no encontrado: {}", p.display()),
            BackupError::BackupNotFound(b) => write!(f, "Backup no encontrado: {}", b.display()),
            BackupError::InvalidBackupName(n) => write!(f, "Nombre de backup inválido: {}", n),
            BackupError::RestoreFailed {
                backup,
                target,
                source,
            } => {
                write!(
                    f,
                    "Fallo al restaurar {} a {}: {}",
                    backup.display(),
                    target.display(),
                    source
                )
            }
        }
    }
}

impl std::error::Error for BackupError {}

impl From<io::Error> for BackupError {
    fn from(err: io::Error) -> Self {
        BackupError::Io(err)
    }
}

impl BackupManager {
    /// Crea un nuevo gestor de backups
    ///
    /// # Arguments
    /// * `max_backups_per_file` - Número máximo de backups a mantener por archivo (default: 5)
    pub fn new(max_backups_per_file: Option<usize>) -> Self {
        let backup_dir = dirs::home_dir()
            .map(|h| h.join(".poshbuddy").join("backups"))
            .unwrap_or_else(|| PathBuf::from(".poshbuddy/backups"));

        Self {
            backup_dir,
            max_backups_per_file: max_backups_per_file.unwrap_or(5),
        }
    }

    /// Crea el directorio de backups si no existe
    fn ensure_backup_dir(&self) -> Result<(), BackupError> {
        if !self.backup_dir.exists() {
            fs::create_dir_all(&self.backup_dir)?;
        }
        Ok(())
    }

    /// Genera un nombre único para el backup basado en timestamp
    fn generate_backup_name(&self, original_path: &Path) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let filename = original_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let extension = original_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("ps1");

        let backup_name = format!("{}_{}.backup.{}", filename, timestamp, extension);
        self.backup_dir.join(backup_name)
    }

    /// Crea un backup del perfil especificado
    ///
    /// # Arguments
    /// * `profile_path` - Ruta al archivo de perfil de PowerShell
    /// * `description` - Descripción del cambio (ej: "Aplicar tema joker", "Instalar plugin zoxide")
    ///
    /// # Returns
    /// Ruta al archivo de backup creado
    pub fn backup_profile(
        &self,
        profile_path: &Path,
        description: &str,
    ) -> Result<PathBuf, BackupError> {
        // Verificar que el perfil existe
        if !profile_path.exists() {
            return Err(BackupError::ProfileNotFound(profile_path.to_path_buf()));
        }

        self.ensure_backup_dir()?;

        let backup_path = self.generate_backup_name(profile_path);
        let content = fs::read(profile_path)?;
        fs::write(&backup_path, content)?;

        // Guardar metadados en archivo .meta
        let meta_path = backup_path.with_extension("meta");
        let meta_content = format!(
            "original_path={}\ndescription={}\ntimestamp={}",
            profile_path.display(),
            description,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        );
        fs::write(meta_path, meta_content)?;

        // Limpiar backups antiguos si excedemos el límite
        self.cleanup_old_backups(profile_path)?;

        Ok(backup_path)
    }

    /// Lista todos los backups disponibles para un perfil específico
    /// Ordenados del más reciente al más antiguo
    pub fn list_backups(&self, profile_path: &Path) -> Result<Vec<BackupInfo>, BackupError> {
        self.ensure_backup_dir()?;

        let profile_name = profile_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                // Buscar backups que coincidan con el nombre del perfil
                if filename.starts_with(&format!("{}_", profile_name))
                    && filename.contains(".backup")
                    && (path.extension() == Some(std::ffi::OsStr::new("backup"))
                        || path.extension() == Some(std::ffi::OsStr::new("ps1")))
                {
                    let metadata = fs::metadata(&path)?;
                    let size_bytes = metadata.len();

                    // Extraer timestamp del nombre
                    let timestamp = filename
                        .split('_')
                        .next_back()
                        .and_then(|t| t.split('.').next())
                        .and_then(|t| t.parse::<u64>().ok())
                        .unwrap_or(0);

                    // Leer descripción del archivo meta
                    let meta_path = path.with_extension("meta");
                    let description = if meta_path.exists() {
                        fs::read_to_string(&meta_path)
                            .ok()
                            .and_then(|content| {
                                content
                                    .lines()
                                    .find(|l| l.starts_with("description="))
                                    .map(|l| l.trim_start_matches("description=").to_string())
                            })
                            .unwrap_or_else(|| "Sin descripción".to_string())
                    } else {
                        "Sin descripción".to_string()
                    };

                    backups.push(BackupInfo {
                        path,
                        original_path: profile_path.to_path_buf(),
                        timestamp,
                        size_bytes,
                        description,
                    });
                }
            }
        }

        // Ordenar por timestamp descendente (más reciente primero)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(backups)
    }

    /// Restaura el backup más reciente para un perfil
    ///
    /// # Arguments
    /// * `profile_path` - Ruta al perfil a restaurar
    ///
    /// # Returns
    /// Información del backup restaurado
    pub fn restore_latest(&self, profile_path: &Path) -> Result<BackupInfo, BackupError> {
        let backups = self.list_backups(profile_path)?;

        if let Some(latest) = backups.first() {
            self.restore_backup(&latest.path, profile_path)?;
            Ok(latest.clone())
        } else {
            Err(BackupError::BackupNotFound(
                self.backup_dir.join("*.backup.ps1"),
            ))
        }
    }

    /// Restaura un backup específico
    ///
    /// # Arguments
    /// * `backup_path` - Ruta al archivo de backup
    /// * `target_path` - Ruta de destino para la restauración
    pub fn restore_backup(
        &self,
        backup_path: &Path,
        target_path: &Path,
    ) -> Result<(), BackupError> {
        if !backup_path.exists() {
            return Err(BackupError::BackupNotFound(backup_path.to_path_buf()));
        }

        // Primero hacer backup del estado actual antes de restaurar
        if target_path.exists() {
            let pre_restore_backup = self.generate_backup_name(target_path);
            let current_content = fs::read(target_path)?;
            fs::write(&pre_restore_backup, current_content)?;
        }

        // Realizar la restauración
        let backup_content = fs::read(backup_path).map_err(|e| BackupError::RestoreFailed {
            backup: backup_path.to_path_buf(),
            target: target_path.to_path_buf(),
            source: e,
        })?;

        fs::write(target_path, backup_content).map_err(|e| BackupError::RestoreFailed {
            backup: backup_path.to_path_buf(),
            target: target_path.to_path_buf(),
            source: e,
        })?;

        Ok(())
    }

    /// Elimina backups antiguos manteniendo solo los N más recientes
    fn cleanup_old_backups(&self, profile_path: &Path) -> Result<(), BackupError> {
        let backups = self.list_backups(profile_path)?;

        // Si tenemos más backups que el límite, eliminar los más antiguos
        if backups.len() > self.max_backups_per_file {
            let to_remove = &backups[self.max_backups_per_file..];
            for backup in to_remove {
                let _ = fs::remove_file(&backup.path);
                let _ = fs::remove_file(backup.path.with_extension("meta"));
            }
        }

        Ok(())
    }

    /// Elimina todos los backups para un perfil específico
    #[allow(dead_code)]
    pub fn delete_all_backups(&self, profile_path: &Path) -> Result<usize, BackupError> {
        let backups = self.list_backups(profile_path)?;
        let count = backups.len();

        for backup in backups {
            let _ = fs::remove_file(&backup.path);
            let _ = fs::remove_file(backup.path.with_extension("meta"));
        }

        Ok(count)
    }

    /// Obtiene el tamaño total utilizado por todos los backups
    #[allow(dead_code)]
    pub fn total_backup_size(&self) -> Result<u64, BackupError> {
        self.ensure_backup_dir()?;

        let mut total = 0u64;
        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total += metadata.len();
            }
        }

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_backup_manager() -> (BackupManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");

        let manager = BackupManager {
            backup_dir,
            max_backups_per_file: 3,
        };

        (manager, temp_dir)
    }

    fn create_test_profile(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_backup_and_restore() {
        let (manager, temp_dir) = create_test_backup_manager();
        let profile =
            create_test_profile(temp_dir.path(), "test_profile.ps1", "contenido original");

        // Crear backup
        let backup_path = manager.backup_profile(&profile, "Test backup").unwrap();
        assert!(backup_path.exists());

        // Wait a tiny bit so the prerestore backup gets a strictly greater timestamp than the actual backup
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Modificar el archivo original
        fs::write(&profile, "contenido modificado").unwrap();

        // Restaurar backup
        manager.restore_latest(&profile).unwrap();

        // Verificar que se restauró
        let restored_content = fs::read_to_string(&profile).unwrap();
        assert_eq!(restored_content, "contenido original");
    }

    #[test]
    fn test_list_backups_order() {
        let (manager, temp_dir) = create_test_backup_manager();
        let profile = create_test_profile(temp_dir.path(), "test_profile.ps1", "v1");

        // Crear múltiples backups
        manager.backup_profile(&profile, "Backup 1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        fs::write(&profile, "v2").unwrap();
        manager.backup_profile(&profile, "Backup 2").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        fs::write(&profile, "v3").unwrap();
        manager.backup_profile(&profile, "Backup 3").unwrap();

        // Listar backups
        let backups = manager.list_backups(&profile).unwrap();
        assert_eq!(backups.len(), 3);

        // Verificar orden (más reciente primero)
        assert!(backups[0].timestamp > backups[1].timestamp);
        assert!(backups[1].timestamp > backups[2].timestamp);

        // Verificar descripciones
        assert_eq!(backups[0].description, "Backup 3");
        assert_eq!(backups[1].description, "Backup 2");
        assert_eq!(backups[2].description, "Backup 1");
    }

    #[test]
    fn test_cleanup_old_backups() {
        let (manager, temp_dir) = create_test_backup_manager();
        let profile = create_test_profile(temp_dir.path(), "test_profile.ps1", "v1");

        // Crear 5 backups (máximo es 3)
        for i in 1..=5 {
            fs::write(&profile, format!("v{}", i)).unwrap();
            manager
                .backup_profile(&profile, &format!("Backup {}", i))
                .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Solo deben quedar 3
        let backups = manager.list_backups(&profile).unwrap();
        assert_eq!(backups.len(), 3);

        // Los más recientes deben permanecer
        assert_eq!(backups[0].description, "Backup 5");
        assert_eq!(backups[1].description, "Backup 4");
        assert_eq!(backups[2].description, "Backup 3");
    }

    #[test]
    fn test_restore_nonexistent_backup() {
        let (manager, _temp_dir) = create_test_backup_manager();
        let fake_backup = PathBuf::from("/nonexistent/backup.backup.ps1");
        let fake_target = PathBuf::from("/nonexistent/profile.ps1");

        let result = manager.restore_backup(&fake_backup, &fake_target);
        assert!(matches!(result, Err(BackupError::BackupNotFound(_))));
    }

    #[test]
    fn test_backup_nonexistent_profile() {
        let (manager, _temp_dir) = create_test_backup_manager();
        let fake_profile = PathBuf::from("/nonexistent/profile.ps1");

        let result = manager.backup_profile(&fake_profile, "Test");
        assert!(matches!(result, Err(BackupError::ProfileNotFound(_))));
    }
}
