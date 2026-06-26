# PoshBuddy — Task Tracker

## Fase 1 — Bugs Críticos

- [/] **1.1** Fix `install_font()` → `tokio::process::Command`
- [ ] **1.2** Fix `load_theme_preview()` → `tokio::process::Command`
- [ ] **1.3** `cargo check` post-Fix 1
- [ ] **1.4** `git commit` — fix(tui): use tokio::process::Command in async tasks
- [ ] **1.5** Fix teclas `1` y `2` → navegación directa
- [ ] **1.6** `cargo check` post-Fix 2
- [ ] **1.7** `git commit` — fix(tui): make keys 1 and 2 navigate directly to their view

## Fase 2 — Refactor a Módulos (pasos atómicos)

- [ ] **2.1** Crear `src/api.rs` con fetch functions + setup_app_task
- [ ] **2.2** `cargo check`
- [ ] **2.3** `git commit` — refactor(api): extract fetch and setup functions to api module
- [ ] **2.4** Crear `src/app.rs` con enums + struct App + impl App
- [ ] **2.5** `cargo check`
- [ ] **2.6** `git commit` — refactor(app): extract App struct and impl to app module
- [ ] **2.7** Crear `src/ui.rs` con la función ui()
- [ ] **2.8** `cargo check`
- [ ] **2.9** `git commit` — refactor(ui): extract ui rendering function to ui module
- [ ] **2.10** Limpiar `src/main.rs` (solo event loop + mods)
- [ ] **2.11** `cargo check`
- [ ] **2.12** `git commit` — refactor(main): reduce main.rs to event loop and entrypoint

## Fase 3 — Limpieza Menor

- [ ] **3.1** Fix `\r\n` en `app.rs::apply_theme()`
- [ ] **3.2** Eliminar bloque `if` vacío en `check_nerd_font()`
- [ ] **3.3** Mover `spinner_tick += 1` de `ui.rs` a `main.rs`
- [ ] **3.4** Aumentar buffer de canal: `channel(10)` → `channel(32)`
- [ ] **3.5** `cargo check` final
- [ ] **3.6** `git commit` — fix(app): crlf endings, dead code, spinner side-effect, channel buffer
