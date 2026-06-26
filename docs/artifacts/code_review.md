# 🦀 PoshBuddy — Code Review Completo

## 📌 Visión General del Proyecto

**PoshBuddy** es una aplicación TUI (Terminal User Interface) escrita en Rust que actúa como suite de personalización integral para PowerShell. Permite gestionar temas de Oh My Posh, validar/instalar Nerd Fonts y modificar el `$PROFILE` de PowerShell automáticamente.

| Campo | Valor |
|---|---|
| **Versión** | `0.2.0-rust` |
| **Lenguaje** | Rust (edition 2021) |
| **Líneas de código** | ~612 (un solo archivo `main.rs`) |
| **Arquitectura** | MVC-inspired |
| **Runtime** | Tokio (async) |
| **UI Framework** | Ratatui + Crossterm |

---

## 🏗️ Arquitectura

```
main.rs (612 líneas)
├── Enums
│   ├── ActiveView { Themes, Fonts }
│   ├── AppState { Loading, Main, Installing(String), Error(String) }
│   └── AppMessage { ThemesLoaded, FontsLoaded, ThemePreviewLoaded, FontInstalled, Error }
├── Structs
│   ├── FontAsset { name: String }
│   └── App (Model) — estado global de la aplicación
├── impl App
│   ├── new()                  — inicialización
│   ├── check_nerd_font()      — detección de Nerd Font via Registry/env
│   ├── filtered_themes()      — filtra temas por búsqueda
│   ├── filtered_fonts()       — filtra fuentes por búsqueda
│   ├── apply_theme()          — modifica $PROFILE de PowerShell
│   ├── install_font()         — invoca oh-my-posh font install (async)
│   └── load_theme_preview()   — invoca oh-my-posh print primary (async)
├── Funciones async
│   ├── fetch_theme_names()    — GitHub API → lista de temas .omp.json
│   ├── fetch_font_names()     — GitHub API → releases de nerd-fonts
│   └── setup_app_task()       — orquesta fetch + descarga paralela de temas
├── fn ui()                    — renderizado Ratatui (View)
└── async fn main()            — setup terminal + event loop (Controller)
```

---

## ✅ Fortalezas del Código

### 1. Arquitectura MVC bien separada
El patrón es claro: `App` gestiona el estado, `ui()` renderiza, y el `main loop` maneja eventos. Fácil de razonar.

### 2. Uso correcto de canales Tokio (`mpsc`)
La comunicación entre tareas async y el hilo principal se hace con `tokio::sync::mpsc`, el patrón correcto para este tipo de TUI.

### 3. Descarga paralela de temas (`join_all`)
```rust
let _results = join_all(download_futures).await;
```
Descarga todos los temas en paralelo usando `futures::join_all`. Muy eficiente.

### 4. Manejo de perfil no destructivo
`apply_theme()` preserva el contenido existente del `$PROFILE` y solo reemplaza (o agrega) la línea de `oh-my-posh init`, sin borrar el resto.

### 5. Filtro en tiempo real
El filtro de temas y fuentes funciona como un fuzzy finder básico con actualización inmediata al escribir.

---

## 🔴 Bugs y Problemas Críticos

### BUG 1 — `std::process::Command` bloqueante dentro de `tokio::spawn`
**Severidad:** 🔴 Alta

**Líneas:** 201, 221

```rust
// En install_font() y load_theme_preview()
tokio::spawn(async move {
    let output = std::process::Command::new(cmd)  // ← BLOQUEANTE
        .args([...])
        .output();
    ...
});
```

`std::process::Command::output()` es una llamada **bloqueante** que congela el thread del runtime de Tokio. Esto puede bloquear el event loop completo y hacer que la UI deje de responder.

**Fix:** Usar `tokio::process::Command` en su lugar:
```rust
use tokio::process::Command;

tokio::spawn(async move {
    let output = Command::new(cmd)
        .args([...])
        .output()
        .await; // ← No bloqueante
    ...
});
```

---

### BUG 2 — `apply_theme()` usa `\n` en vez de `\r\n` en Windows
**Severidad:** 🟠 Media

**Línea:** 188

```rust
fs::write(&self.profile_path, new_content.join("\n"))?;  // ← Siempre \n
```

En Windows, PowerShell espera `\r\n` como line ending. Usar `\n` puede causar problemas de visualización o parsing del perfil en algunos editores y entornos.

**Fix:**
```rust
let line_ending = if cfg!(windows) { "\r\n" } else { "\n" };
fs::write(&self.profile_path, new_content.join(line_ending))?;
```

---

### BUG 3 — Race condition: `apply_theme()` puede fallar sin manejarse el error
**Severidad:** 🟠 Media

**Líneas:** 572-573

```rust
app.apply_theme(theme)?;
break;
```

El `?` aquí propaga el error al `main()`, que lo maneja limpiando el terminal — correcto. Sin embargo, la UI nunca muestra al usuario que la aplicación del tema falló antes de salir. El usuario solo ve que el programa terminó.

**Fix sugerido:** Capturar el error y mostrarlo en `AppState::Error` antes de salir, en lugar de propagar directamente.

---

### BUG 4 — Descarte silencioso de errores de descarga
**Severidad:** 🟡 Baja-Media

**Línea:** 353

```rust
let _results = join_all(download_futures).await;
```

Los errores individuales de descarga de temas son completamente ignorados. Si un tema falla su descarga, no hay ninguna notificación.

**Fix:** Procesar los resultados y loguearlos, o enviar una notificación al canal `tx`:
```rust
let results = join_all(download_futures).await;
let failed: Vec<_> = results.iter().filter(|r| r.is_err()).collect();
// Enviar conteo de fallos al UI si > 0
```

---

## 🟡 Problemas de Calidad / Code Smells

### SMELL 1 — Todo el código en un solo archivo
**Líneas:** 1–612

612 líneas en `src/main.rs` sin módulos. A medida que crezca el proyecto (v0.3.0, v0.4.0), esto será difícil de mantener.

**Refactor sugerido:**
```
src/
├── main.rs          (event loop + entrypoint)
├── app.rs           (struct App + lógica de estado)
├── ui.rs            (función ui() + renderizado)
├── api.rs           (fetch_theme_names, fetch_font_names)
└── installer.rs     (setup_app_task, install_font)
```

---

### SMELL 2 — Navegación `Tab` no distingue entre Tab→1 y Tab→2
**Líneas:** 539-541

```rust
KeyCode::Tab | KeyCode::Char('1') | KeyCode::Char('2') => {
    app.active_view = if app.active_view == ActiveView::Themes { 
        ActiveView::Fonts 
    } else { 
        ActiveView::Themes 
    };
}
```

Las teclas `1` y `2` deberían navegar directamente a la vista correspondiente, no solo hacer toggle. Actualmente `2` en la vista de Fuentes te lleva a Temas (incorrecto).

**Fix:**
```rust
KeyCode::Tab => {
    app.active_view = if app.active_view == ActiveView::Themes { 
        ActiveView::Fonts 
    } else { 
        ActiveView::Themes 
    };
}
KeyCode::Char('1') => { app.active_view = ActiveView::Themes; }
KeyCode::Char('2') => { app.active_view = ActiveView::Fonts; }
```

---

### SMELL 3 — `check_nerd_font()` tiene código muerto
**Líneas:** 110-113

```rust
if std::env::var("TERMINAL_EMULATOR").is_ok() || std::env::var("WT_SESSION").is_ok() {
    // Windows Terminal o emuladores modernos suelen tener fuentes Nerd configuradas
    // pero vamos a intentar ser más precisos con el comando de registro si es Windows
}
```

Este bloque `if` no hace nada. El compilador probablemente emite un warning de bloque vacío.

**Fix:** Eliminar el bloque o implementar la lógica comentada.

---

### SMELL 4 — `check_nerd_font()` ejecuta PowerShell de forma bloqueante en `new()`
**Línea:** 82, 122-134

```rust
let has_nerd_font = Self::check_nerd_font(); // en App::new()
// Dentro de check_nerd_font():
let output = std::process::Command::new(cmd).output(); // bloqueante
```

`App::new()` se llama en el hilo principal antes del runtime de Tokio, así que técnicamente no bloquea el async runtime. Sin embargo, bloquea el inicio de la aplicación por el tiempo que tarde PowerShell en ejecutar el comando de Registry.

**Fix:** Hacer la detección asíncrona y enviarla por el canal `AppMessage`.

---

### SMELL 5 — `spinner_tick` se incrementa en `ui()` (side effect en View)
**Línea:** 391

```rust
app.spinner_tick += 1; // ← mutación del modelo dentro de la vista
```

La función `ui()` debería ser puramente descriptiva (sólo renderizar). Modificar el estado dentro de la vista rompe la separación MVC.

**Fix:** Incrementar `spinner_tick` en el event loop principal, no en `ui()`.

---

### SMELL 6 — Tamaño del canal `mpsc` muy pequeño
**Línea:** 499

```rust
let (tx, mut rx) = mpsc::channel(10);
```

Un buffer de 10 puede ser insuficiente cuando `setup_app_task` envía múltiples mensajes rápidamente (FontsLoaded + ThemesLoaded). Aunque el `try_recv` en el loop lo drena continuamente, si la UI tarda en renderizar podría haber backpressure.

**Fix:** Aumentar a `32` o `64` para mayor seguridad.

---

## 📊 Resumen de Hallazgos

| Categoría | Cantidad | Severidad |
|---|---|---|
| Bugs críticos | 2 | 🔴 Alta |
| Bugs medios | 2 | 🟠 Media |
| Code smells | 6 | 🟡 Baja-Media |
| **Total** | **10** | |

---

## 🗺️ Prioridad de Fixes Recomendada

1. **[CRÍTICO]** Migrar `std::process::Command` → `tokio::process::Command` en `install_font()` y `load_theme_preview()`
2. **[MEDIO]** Separar el código en módulos (`app.rs`, `ui.rs`, `api.rs`)
3. **[MEDIO]** Corregir la navegación de teclas `1` y `2`
4. **[MEDIO]** Fix de line endings `\r\n` en Windows
5. **[BAJO]** Eliminar código muerto en `check_nerd_font()`
6. **[BAJO]** Mover `spinner_tick` fuera de `ui()`

---

## 🔮 Roadmap Técnico (alineado con el README)

| Versión | Feature | Complejidad |
|---|---|---|
| v0.3.0 | Motor de módulos PowerShell (Plugins) | Alta |
| v0.4.0 | Sincronización de perfiles en la nube | Alta |
| v1.0.0 | Suite completa | - |

> **Recomendación:** Antes de v0.3.0 es crítico refactorizar a módulos para que el código sea mantenible.
