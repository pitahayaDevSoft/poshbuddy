# PoshBuddy — Walkthrough de Mejoras y Refactorización

Se han completado todas las fases para modernizar y estabilizar la base de código de PoshBuddy en Rust.

## Cambios Realizados

### 1. Fixes Críticos (Fase 1)
- **Migración a `tokio::process::Command`**: Se reemplazó el uso de comandos bloqueantes en hilos asíncronos. Esto evita que la interfaz se congele al navegar por los temas o instalar fuentes.
- **Navegación Directa**: Las teclas `1` y `2` ahora llevan directamente a "Temas" y "Fuentes" respectivamente.

### 2. Refactorización Modular (Fase 2)
El archivo `src/main.rs` se ha dividido en módulos especializados:
- `src/app.rs`: Estado central (`App`).
- `src/api.rs`: Red y descarga.
- `src/ui.rs`: Renderizado.
- `main.rs`: Orquestador de eventos.

### 3. Ajustes de Calidad (Fase 3)
- **Line Endings (`\r\n`)**: Perfil compatible con Windows.
- **Robustez del Canal**: Buffer de 32 mensajes.

### 4. Soporte ANSI y Colores (Fase 4)
- **Renderizado ANSI**: Integración de `ansi-to-tui` para previsualización real en el TUI.

### 5. Validador y Fixes (Fase 5)
- **Fix Salto de 3 Líneas**: Filtrado de `KeyEventKind::Press`.
- **Validador de Oh My Posh**: Detección e instalador automático vía Winget con logs en tiempo real.

### 6. Aislamiento de Previsualización (Fase 6)
- **Entorno Limpio**: Uso de `env_clear` para forzar la visualización del tema seleccionado sin interferencias del sistema.

### 7. Pulido de UI (Fase 7)
- **Margen de Cortesía**: Espaciado interno para evitar que iconos anchos rompan el borde del recuadro amarillo.

### 8. Detección Dinámica de $PROFILE (Fase 8)
- **Soporte Multi-Shell**: El programa detecta automáticamente las rutas de perfil reales para **Windows PowerShell 5.1** y **PowerShell 7** de forma dinámica.
- **Independencia de Ruta**: Ya no asume la carpeta "Documentos" por defecto; pregunta directamente a la shell por su configuración, resolviendo problemas de carpetas movidas a discos como `F:\`.
- **Soporte Pro**: Los temas se aplican en todas las shells detectadas simultáneamente.

## Verificación

Se realizaron pruebas con `cargo check` y validación manual del archivo de perfil en disco `F:\`.

## Historial de Commits (Log Reciente)

```text
6144e70 feat(app): dynamic shell profile detection and multi-shell support
0374897 style(ui): trim preview output and add margin for better box rendering
92e0877 fix(app): total environment isolation for theme previews
892a6da feat(app): add OMP dependency check and automatic installer with transparent logs
```

---
> [!IMPORTANT]
> PoshBuddy ahora es compatible con configuraciones de usuario avanzadas y múltiples instalaciones de shell.
