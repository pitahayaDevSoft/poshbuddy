# PoshBuddy — Plan de Fixes y Refactor

## Objetivo

Aplicar las mejoras identificadas en el code review sin agregar features nuevas.
Tres fases ordenadas por riesgo/impacto, con un commit al final de cada una.

## Cambios que requieren revisión

> [!IMPORTANT]
> **Fase 2 (refactor a módulos)** reestructura los archivos del proyecto.
> El comportamiento externo del programa NO cambia — solo la organización interna.
> Considera hacer un `cargo build --release` de verificación entre fases.

> [!WARNING]
> `src/main.rs` será reemplazado por 4 archivos nuevos. El original quedará
> reducido a ~60 líneas (solo el event loop y el entrypoint).

---

## Fase 1 — Bugs Críticos

### [MODIFY] [main.rs](file:///g:/DEVELOPMENT/poshbuddy/src/main.rs)

#### Fix 1 — `tokio::process::Command` (líneas 192–235)

Reemplazar `std::process::Command` por `tokio::process::Command` en las dos
funciones que lo usan dentro de `tokio::spawn`:

- `App::install_font()` — línea ~201
- `App::load_theme_preview()` — línea ~221

```diff
- use std::process::Command;  // implícito vía std
+ use tokio::process::Command;  // al top del bloque async
```

Ambas funciones necesitan `.output().await` en vez de `.output()`.

#### Fix 2 — Teclas `1` y `2` (línea ~539)

```diff
- KeyCode::Tab | KeyCode::Char('1') | KeyCode::Char('2') => {
-     app.active_view = if ... Themes { Fonts } else { Themes };
- }

+ KeyCode::Tab => {
+     app.active_view = if ... Themes { Fonts } else { Themes };
+ }
+ KeyCode::Char('1') => { app.active_view = ActiveView::Themes; }
+ KeyCode::Char('2') => { app.active_view = ActiveView::Fonts; }
```

**Commit al final de Fase 1:**
```
fix(tui): use tokio::process::Command and fix 1/2 key navigation
```

---

## Fase 2 — Refactor a Módulos

### Estructura resultante

```
src/
├── main.rs      [MODIFY]  ~60 líneas  — event loop + entrypoint
├── app.rs       [NEW]     ~170 líneas — struct App, impl App, enums
├── ui.rs        [NEW]     ~130 líneas — fn ui()
└── api.rs       [NEW]     ~120 líneas — fetch_theme_names, fetch_font_names, setup_app_task
```

### [NEW] [app.rs](file:///g:/DEVELOPMENT/poshbuddy/src/app.rs)

Contendrá:
- Enums: `ActiveView`, `AppState`, `AppMessage`
- Struct: `FontAsset`, `App`
- `impl App` completo: `new()`, `check_nerd_font()`, `filtered_themes()`, `filtered_fonts()`, `apply_theme()`, `install_font()`, `load_theme_preview()`

### [NEW] [ui.rs](file:///g:/DEVELOPMENT/poshbuddy/src/ui.rs)

Contendrá:
- `pub fn ui(f: &mut ratatui::Frame, app: &mut App)`
- Todo el renderizado de Ratatui (layouts, widgets, estados de Loading/Main/Error/Installing)
- `spinner_tick` se incrementará en el event loop (`main.rs`), no aquí.

### [NEW] [api.rs](file:///g:/DEVELOPMENT/poshbuddy/src/api.rs)

Contendrá:
- `pub async fn fetch_theme_names()`
- `pub async fn fetch_font_names()`
- `pub async fn setup_app_task(tx, themes_dir)`

### [MODIFY] [main.rs](file:///g:/DEVELOPMENT/poshbuddy/src/main.rs)

Quedará con:
- `mod app; mod ui; mod api;`
- `use` declarations
- `async fn main()` — solo el event loop y la gestión del terminal

**Commit al final de Fase 2:**
```
refactor(src): split main.rs into app, ui, and api modules
```

---

## Fase 3 — Limpieza Menor

### [MODIFY] [app.rs](file:///g:/DEVELOPMENT/poshbuddy/src/app.rs) (post-Fase 2)

- **`\r\n` en Windows:** `apply_theme()` usará `if cfg!(windows) { "\r\n" } else { "\n" }`
- **Código muerto:** eliminar el bloque `if` vacío en `check_nerd_font()` (líneas 110–113 del original)
- **Canal más amplio:** cambiar `mpsc::channel(10)` → `mpsc::channel(32)` en `main.rs`

### [MODIFY] [ui.rs](file:///g:/DEVELOPMENT/poshbuddy/src/ui.rs) (post-Fase 2)

- **`spinner_tick` fuera de `ui()`:** eliminar `app.spinner_tick += 1` de `ui()`. El incremento pasará al event loop en `main.rs`.

**Commit al final de Fase 3:**
```
fix(app): crlf line endings, dead code removal, and ui side-effect fix
```

---

## Plan de Verificación

### Compilación
```bash
cargo check       # entre fases para detectar errores pronto
cargo build       # al final para confirmar que todo compila
```

### Manual (post-ejecución)
- [ ] El programa arranca y muestra el spinner de carga
- [ ] Las teclas `1` y `2` navegan a la vista correcta directamente
- [ ] La previsualización de tema no congela la UI al navegar rápido
- [ ] `Enter` en un tema aplica y sale limpiamente

---

## Open Questions

> [!NOTE]
> No hay preguntas bloqueantes. El plan es conservador y reversible.
> Todo lo que se crea es nuevo (archivos `app.rs`, `ui.rs`, `api.rs`).
> El único archivo existente que se modifica significativamente es `src/main.rs`.
