🎯 **Qué:** Se solucionó una vulnerabilidad de inyección de comandos en las funciones que ejecutan comandos de PowerShell (`install_module`, `check_module_installed`, `uninstall_module`, `get_module_info` en `src/plugin_installer.rs` y `install_plugin` en `src/app/services.rs`).

⚠️ **Riesgo:** Un atacante que pudiera controlar el valor de `module_name` podría inyectar comandos de PowerShell arbitrarios que se ejecutarían con los privilegios del usuario actual, lo que podría llevar a la ejecución de código remoto o a la toma de control del sistema.

🛡️ **Solución:** Se reemplazó la interpolación de cadenas de Rust por variables de entorno de `tokio::process::Command` y `std::process::Command` (`.env("MODULE_NAME", module_name)`). Luego, estas variables se referencian de forma segura desde PowerShell usando la sintaxis `$env:MODULE_NAME`, previniendo así la evaluación de código y asegurando que las variables se traten estrictamente como datos en el subproceso.
