# 🚀 PoshBuddy

[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.2.0--rust-blue.svg)](https://github.com/julio/poshbuddy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20(WSL)-brightgreen.svg)]()

**PoshBuddy** es una suite de personalización integral para PowerShell. Diseñada para transformar tu terminal en una herramienta profesional, permite gestionar temas de **Oh My Posh**, validar la instalación de **Nerd Fonts** y automatizar la configuración de tu perfil con una interfaz TUI fluida y moderna escrita en Rust.

---

## 🎯 ¿Por qué PoshBuddy?

Configurar una terminal estética y funcional en PowerShell suele requerir múltiples pasos manuales: descargar temas, buscar la fuente adecuada, editar el `$PROFILE`, etc. PoshBuddy centraliza todo esto en una única aplicación de terminal:

- **Gestión Asíncrona:** Descarga cientos de temas en segundo plano sin bloquear la interfaz.
- **Validación Inteligente:** Detecta si tu fuente actual es compatible con iconos (Nerd Fonts).
- **Integración Nativa:** Modifica tu perfil de PowerShell de forma segura.
- **Ligero y Rápido:** Construido con Rust para un consumo mínimo de recursos.

---

## 📋 Tabla de Contenidos
1. [Requisitos Previos](#-requisitos-previos)
2. [Instalación](#-instalación)
3. [Uso y Controles](#-uso-y-controles)
4. [Arquitectura](#-arquitectura)
5. [Desarrollo](#-desarrollo)
6. [Roadmap 2026](#-roadmap-2026)
7. [Contribuciones](#-contribuciones)
8. [Licencia](#-licencia)

---

## 🔧 Requisitos Previos

Antes de empezar, asegúrate de tener instalado:
- [Oh My Posh](https://ohmyposh.dev/docs/installation/windows) (Core engine).
- [PowerShell 7+](https://github.com/PowerShell/PowerShell) (Recomendado).
- [Rust](https://www.rust-lang.org/tools/install) (Para compilar desde el código fuente).

---

## 📦 Instalación

### Desde el código fuente (Recomendado)

1. Clona el repositorio:
   ```bash
   git clone https://github.com/julio/poshbuddy.git
   cd poshbuddy
   ```

2. Compila el binario:
   ```bash
   cargo build --release
   ```

3. Ejecuta la aplicación:
   ```bash
   ./target/release/poshbuddy
   ```

### Verificación
Para confirmar que PoshBuddy tiene acceso a tus herramientas de terminal, el dashboard mostrará un mensaje de carga mientras sincroniza los temas oficiales desde el repositorio de `JanDeDobbeleer/oh-my-posh`.

---

## ⌨️ Uso y Controles

PoshBuddy utiliza una interfaz TUI intuitiva dividida en dos vistas principales:

### Controles Generales
- **Tab / 1 / 2:** Alternar entre la pestaña de **Temas** y **Fuentes**.
- **↑ / ↓ / Rueda del ratón:** Navegar por las listas.
- **Letras / Backspace:** Filtrar temas o fuentes en tiempo real.
- **Enter:** Aplicar el tema seleccionado o instalar la fuente elegida.
- **Esc / Q:** Salir de la aplicación.

### Vista de Temas
Permite previsualizar el diseño del prompt (via `oh-my-posh print`) antes de aplicarlo. Al presionar **Enter**, PoshBuddy actualizará tu archivo `$PROFILE` automáticamente.

### Vista de Fuentes
Muestra las últimas **Nerd Fonts** disponibles. Si PoshBuddy detecta que no estás usando una fuente compatible, verás una advertencia ⚠️ en el panel de información.

---

## 🏗️ Arquitectura

El proyecto sigue una estructura inspirada en MVC para mantener la lógica separada de la renderización:

- **App (Model):** Gestiona el estado de los temas, filtros y rutas de archivos.
- **UI (View):** Implementado con `Ratatui`, define el layout y los widgets de la interfaz.
- **Main Loop (Controller):** Basado en `Tokio` y `Crossterm`, maneja eventos de teclado/ratón y tareas asíncronas de red.

---

## 🛠️ Desarrollo

Si deseas contribuir o modificar PoshBuddy, puedes configurar tu entorno de desarrollo así:

```bash
# Ejecutar en modo debug con logs
cargo run

# Ejecutar el linter para asegurar calidad de código
cargo clippy

# Ejecutar tests
cargo test
```

### Estructura del Proyecto
- `src/main.rs`: Lógica principal de la TUI y el bucle de eventos.
- `Cargo.toml`: Dependencias críticas (Tokio, Ratatui, Reqwest).
- `docs/superpowers/plans/`: Documentación técnica de los planes de implementación.

---

## 🗺️ Roadmap 2026

- [x] **v0.1.0:** Versión legacy en PowerShell.
- [x] **v0.2.0:** Port a Rust, gestión asíncrona y validación de fuentes.
- [ ] **v0.3.0:** Motor de gestión de módulos de PowerShell (Plugins).
- [ ] **v0.4.0:** Sincronización de perfiles en la nube y backups.
- [ ] **v1.0.0:** Suite completa "Everything for PowerShell".

---

## 🤝 Contribuyendo

Las contribuciones son bienvenidas. Por favor, lee `CONTRIBUTING.md` antes de abrir un Pull Request.

1. Haz un Fork del proyecto.
2. Crea una rama para tu feature (`git checkout -b feature/AmazingFeature`).
3. Haz commit de tus cambios (`git commit -m 'feat: Add some AmazingFeature'`).
4. Haz push a la rama (`git push origin feature/AmazingFeature`).
5. Abre un Pull Request.

---

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo `LICENSE` para más detalles.

Copyright © 2026 Julio.

---

## 👤 Autor

**Julio** - *Senior Software Engineer*
- GitHub: [@julio](https://github.com/julio)
- Proyecto: [PoshBuddy](https://github.com/julio/poshbuddy)

---

## 🙏 Agradecimientos

- A [JanDeDobbeleer](https://github.com/JanDeDobbeleer) por Oh My Posh.
- A la comunidad de [Ratatui](https://ratatui.rs/) por el increíble framework TUI.
- A [Nerd Fonts](https://www.nerdfonts.com/) por los glifos que hacen que nuestras terminales cobren vida.
