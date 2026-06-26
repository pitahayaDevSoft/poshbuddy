# PoshBuddy — Detección Dinámica y Robusta de Perfiles ($PROFILE)

## Objetivo

Solucionar el problema de aplicación de temas detectando las rutas reales de los perfiles de PowerShell en lugar de usar una ruta fija. Esto garantiza compatibilidad con carpetas "Documentos" movidas a otros discos (como `F:\`) y con múltiples versiones de PowerShell (5.1 y 7).

## Cambios Propuestos

### [MODIFY] [src/app.rs](file:///g:/DEVELOPMENT/poshbuddy/src/app.rs)

**1. Nueva Estructura**: Añadir `detected_profiles` para manejar múltiples shells.

```rust
pub struct App {
    // ...
    pub detected_profiles: Vec<PathBuf>,
}
```

**2. Lógica de Detección**: Implementar una función que pregunte a los shells por su configuración real.

```rust
    pub fn detect_profiles() -> Vec<PathBuf> {
        let mut profiles = Vec::new();
        let shells = [("powershell", "5.1"), ("pwsh", "7")];

        for (cmd, _label) in shells {
            let output = std::process::Command::new(cmd)
                .args(["-NoProfile", "-Command", "Write-Host -NoNewline $PROFILE"])
                .output();

            if let Ok(out) = output {
                let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !path_str.is_empty() {
                    profiles.push(PathBuf::from(path_str));
                }
            }
        }
        
        // Eliminar duplicados si los perfiles están linkeados
        profiles.sort();
        profiles.dedup();
        profiles
    }
```

**3. Aplicación Multivalor**: Modificar `apply_theme` para iterar sobre `self.detected_profiles`.

### [MODIFY] [src/ui.rs](file:///g:/DEVELOPMENT/poshbuddy/src/ui.rs)

Actualizar el panel de información para mostrar cuántos perfiles se verán afectados, dando transparencia total al usuario.

---

## Verificación Plan

### Automatizada
1. `cargo check` para validar tipos y lógica.

### Manual
1. Ejecutar `cargo run`.
2. Aplicar un tema (ENTER).
3. Verificar con `Get-Content $PROFILE` en **PowerShell 7** y **Windows PowerShell 5.1** que el tema se ha actualizado en ambos archivos reales.
4. Abrir una nueva terminal (cualquier versión) y confirmar el cambio visual.

## Preguntas Abiertas

> [!NOTE]
> ¿Deseas que también intentemos detectar `.zshrc` o `.bashrc` en Windows (vía WSL o Git Bash) o nos centramos únicamente en el ecosistema nativo de PowerShell?
