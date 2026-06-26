# PoshBuddy — Fixes y Validador de Dependencias

## Objetivos

1.  **Corregir la Navegación**: Eliminar el salto de 3 líneas filtrando los eventos de teclado (`Press` únicamente).
2.  **Corregir el Preview**: Asegurar que `oh-my-posh` muestre el tema seleccionado ignorando configuraciones previas del sistema.
3.  **Validador de Oh My Posh**: 
    - Detectar si la herramienta está instalada al arrancar.
    - Si falta, mostrar una pantalla de instalación automática y transparente.
    - Automatizar la instalación mediante `winget` (Windows) o `brew` (macOS/Linux).

## Cambios Propuestos

### [MODIFY] [main.rs](file:///g:/DEVELOPMENT/poshbuddy/src/main.rs)

- Filtrar `event::KeyEventKind::Press` para evitar el triple procesamiento por tecla.
- Añadir lógica para manejar el nuevo estado `AppState::DependencyMissing`.

### [MODIFY] [app.rs](file:///g:/DEVELOPMENT/poshbuddy/src/app.rs)

**Tipos y Lógica**:
- Añadir `DependencyMissing` y `InstallingDependency` a `AppState`.
- Método `check_dependencies()` para verificar `oh-my-posh`.
- Método `install_omp()` que ejecuta `winget install JanDeDobbeleer.OhMyPosh` y reporta progreso.

**Fix Preview**:
- Usar `.env_clear()` o `.env_remove("POSH_THEME")` en `load_theme_preview`.

### [MODIFY] [ui.rs](file:///g:/DEVELOPMENT/poshbuddy/src/ui.rs)

- **Pantalla de Error de Dependencia**: Un diseño elegante que explique que falta Oh My Posh y ofrezca instalarlo con [ENTER].
- **Pantalla de Instalación**: Mostrar el progreso de la instalación (o un spinner con estado transparente).

## Verificación Plan

### Automatizada
1. `cargo check` para asegurar integridad del refactor.
2. Simular falta de OMP (renombrando temporalmente el exe) para forzar el flujo de instalación.

### Manual
1. Pulsar ↓ y verificar que baja solo 1 línea.
2. Navegar temas y ver el cambio instantáneo de color/diseño.
3. Si OMP no estuviera, ver el aviso y poder instalarlo con un toque.

## Preguntas Abiertas

> [!IMPORTANT]
> Para la "transparencia" en la instalación, ¿prefieres ver el log detallado de Winget en una caja de texto o un indicador de progreso simplificado con mensajes de estado?
