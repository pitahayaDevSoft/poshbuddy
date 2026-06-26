# PoshBuddy — Sistema de Onboarding y Diagnóstico

## Objetivo

Asegurar que el usuario tenga la mejor experiencia posible informándole sobre los requisitos técnicos (Nerd Fonts, PowerShell 7, Windows Terminal) antes de entrar a la gestión de temas. Esto evitará frustraciones por previsualizaciones rotas o perfiles fallidos.

## Cambios Propuestos

### [MODIFY] [src/app.rs](file:///g:/DEVELOPMENT/poshbuddy/src/app.rs)

**1. Nueva Estructura de Diagnóstico**:
```rust
pub struct SystemSpecs {
    pub ps_version: String,
    pub is_pwsh_7: bool,
    pub has_nerd_font: bool,
    pub is_windows_terminal: bool,
}
```

**2. Nuevo Estado**: `AppState::Onboarding(SystemSpecs)`.

**3. Lógica de Detección**: Implementar métodos en `App` para obtener la versión de PS y el tipo de terminal (vía variables de entorno como `WT_SESSION`).

### [MODIFY] [src/ui.rs](file:///g:/DEVELOPMENT/poshbuddy/src/ui.rs)

**Diseño de la Pantalla de Inicio**:
- **Checklist dinámico**:
  - [x] Nerd Font detectada (o [!] Recomendación).
  - [x] PowerShell 7 (o [!] Advertencia de versión vieja).
  - [x] Windows Terminal (o [!] Recomendación "terminal.app").
- **Sección de Consejos**: Explicar por qué estas herramientas son necesarias para Oh My Posh.
- **Botón de Acción**: "[ENTER] Entrar a PoshBuddy".

### [MODIFY] [src/main.rs](file:///g:/DEVELOPMENT/poshbuddy/src/main.rs)

- Iniciar la aplicación en el estado `Onboarding`.
- Manejar la transición al estado `Main` (o `Loading`) cuando el usuario pulse ENTER.

---

## Verificación Plan

### Automatizada
1. `cargo check` para validar la nueva lógica de estados.

### Manual
1. Ejecutar `cargo run`.
2. Verificar que la primera pantalla muestra correctamente tus specs (PS 7.5, Nerd Font detectada, etc.).
3. Pulsar ENTER y verificar que entras a la lista de temas normalmente.

## Preguntas Abiertas

> [!IMPORTANT]
> ¿Deseas que esta pantalla aparezca **siempre** al iniciar, o solo la primera vez (mediante un archivo de config persistente)? Por ahora la implementaré para que aparezca siempre hasta que definamos la persistencia de configuración.
