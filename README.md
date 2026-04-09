<div align="center">
  <img src="https://raw.githubusercontent.com/julesklord/poshbuddy/main/assets/poshbuddy_logo.png" alt="PoshBuddy Logo" width="200" onerror="this.src='https://placehold.co/200x200/222222/00d2ff?text=PoshBuddy'"/>
  
# PoshBuddy
  
  **El gestor TUI definitivo para Oh My Posh en entornos Windows y PowerShell**

  [![Rust](https://img.shields.io/badge/Rust-1.76+-orange.svg)](https://www.rust-lang.org)
  [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
  [![Platform](https://img.shields.io/badge/Platform-Windows-lightgray.svg)]()
  [![State](https://img.shields.io/badge/State-Beta-green.svg)]()
</div>

<br>

**PoshBuddy** es una interfaz de terminal (TUI) ligera, rápida y hermosamente diseñada en Rust, creada para simplificar la instalación, gestión y personalización de temas y fuentes para **Oh My Posh**. Adiós a la configuración manual de perfiles y a lidiar con JSONs; PoshBuddy hace el trabajo pesado por ti con una experiencia visual inmersiva.

---

## ✨ Características Principales

- **🎨 Previsualización Real ANSI**: Visualiza los temas de Oh My Posh directamente en la terminal con renderizado de colores reales y glifos gracias a `ansi-to-tui`, aislando el entorno para una previsualización fidedigna.
- **🚀 Instalación Automática (Dependency Manager)**: PoshBuddy detecta automáticamente si `oh-my-posh` está instalado en tu sistema. Si no lo está, te ofrece un instalador transparente y en tiempo real usando `winget`.
- **⚕️ Diagnóstico Dinámico (Onboarding)**: Evalúa dinámicamente tu entorno al arrancar. Te alerta si te falta una Nerd Font, si estás usando un PowerShell antiguo (5.1 vs 7+) o una consola obsoleta frente a modernos emuladores como Windows Terminal.
- **🔗 Soporte Multi-Perfil Dinámico**: Aplica tu tema elegido instantáneamente y de forma simultánea en todas tus instalaciones de PowerShell detectadas (Windows PowerShell clásico y PowerShell 7 Core), independientemente de la unidad en la que residan tus documentos.
- **🔤 Gestor de Fuentes (Nerd Fonts)**: Explora, descarga e instala las famosas *Nerd Fonts* necesarias para ver los característicos iconos de desarrollo sin salir de la TUI.

## 🛠️ Requisitos Previos

Para la experiencia óptima (Golden Standard), recomendamos:

- **[Windows Terminal](https://github.com/microsoft/terminal)**
- **[PowerShell 7+](https://github.com/PowerShell/PowerShell)**
- Una **Nerd Font** configurada en tu emulador de terminal elegida.

*(Nota: PoshBuddy funciona en consolas clásicas, pero la visualización y experiencia serán abismalmente superiores cumpliendo los requisitos superiores, el programa te guiará durante el onboarding).*

## 📦 Instalación

Asegúrate de tener instalado [Rust y Cargo](https://rustup.rs/).

```powershell
# Clonar el repositorio
git clone https://github.com/julesklord/poshbuddy.git
cd poshbuddy

# Compilar y ejecutar
cargo run --release
```

## 🎮 Uso

1. Lanza el ejecutable o `cargo run`.
2. Revisa la pantalla de **Diagnóstico del Sistema** y pulsa `[ENTER]` para empezar.
3. Utiliza las flechas `[ARRIBA]` y `[ABAJO]` para navegar entre los temas de la lista.
4. Navega entre paneles con `[TAB]` o usa atajos rápidos (`[1]` para Temas, `[2]` para Fuentes).
5. Selecciona tu tema o fuente y pulsa `[ENTER]` para aplicarlo/instalarlo.
6. **Disfruta de tu nuevo prompt**. PoshBuddy te indicará cuándo debes recargar la terminal o aplicar los cambios en sus nuevas pantallas de *Feedback de Éxito*.

## 🗺️ Roadmap y Futuro

PoshBuddy se encuentra en constante y activa evolución. Estamos trabajando en allanar el camino para las siguientes características clave (V0.3.0+):

- [ ] **🔌 Soporte para Instalación de Plugins**: Un gestor gráfico unificado para añadir bloques de segmentos extra, scripts auxiliares y módulos de prompt.
- [ ] **🌐 Soporte Multilenguaje (i18n)**: Empezando el soporte nativo multi-idioma (Inglés, Español de manera primaria) para abrir PoshBuddy a toda la comunidad global.
- [ ] **📦 Lanzamientos en Binario**: Distribución vía WinGet y Scoop para una instalación sin necesitar un runtime de Rust.

## 🤝 Contribuir

¡Las contribuciones son bienvenidas! Si deseas ayudar a expandir PoshBuddy (incluyendo los milestones de multilenguaje y plugins):

1. Haz un *Fork* del proyecto.
2. Crea una rama para tu característica (`git checkout -b feature/NuevaCaracteristica`).
3. Haz *Commit* a tus cambios (`git commit -m 'feat(scope): añade NuevaCaracteristica'`).
4. Haz *Push* de la rama (`git push origin feature/NuevaCaracteristica`).
5. Abre un **Pull Request**.

Por favor, lee [CONTRIBUTING.md](CONTRIBUTING.md) para más detalles sobre el código de conducta y convenciones de la base de código.

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo [LICENSE](LICENSE) para más detalles.

---

<div align="center">
  <i>Construido con ❤️ para la comunidad de desarrolladores en Windows.</i>
</div>
