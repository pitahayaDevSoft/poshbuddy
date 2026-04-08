# 🚀 PoshBuddy (v0.2.0-rust)

**PoshBuddy** es la suite de personalización integral definitiva para PowerShell. Ahora con un potente núcleo en **Rust** para una experiencia TUI ultra fluida y profesional.

## 🚧 Estado del Proyecto: Beta (v0.2.0)
Hemos migrado el núcleo a Rust para mejorar el rendimiento y la fiabilidad. PoshBuddy ahora gestiona temas y fuentes de forma asíncrona.

## 🎯 Roadmap 2026
- [x] **v0.1.0:** Dashboard TUI en PowerShell (Legacy).
- [x] **v0.2.0:** Port a Rust + Instalador de Fuentes (Nerd Fonts integration).
- [ ] **v0.3.0:** Motor de Gestión de Plugins (Módulos de PowerShell).
- [ ] **v0.4.0:** Perfiles de Usuario y Backup de Configuraciones en la Nube.
- [ ] **v1.0.0:** Suite Completa de Experiencia de Terminal.

## 🛠️ Arquitectura (Rust Port)
- **Engine:** Rust + Ratatui (TUI).
- **Async:** Tokio + Reqwest para fetching de temas y fuentes sin bloqueo.
- **Integration:** Gestión directa del `$PROFILE` de PowerShell.

## ⌨️ Controles
- **Tab / 1-2:** Cambiar entre pestañas (Temas / Fuentes).
- **↑ / ↓ / Scroll:** Navegación por las listas.
- **Letras / Backspace:** Filtrado dinámico en tiempo real.
- **Enter:** Aplicar tema o instalar fuente.
- **Esc / Q:** Salir de la aplicación.

---
**Desarrollado con rigor técnico por Julio (Senior Engineer).**
