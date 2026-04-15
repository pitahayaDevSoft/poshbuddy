## 2024-10-24 - Explicit State Rendering in TUIs
**Learning:** In Terminal User Interfaces (TUIs) like PoshBuddy (built with Ratatui), implicit states such as active search queries or filters must be explicitly rendered in the UI (e.g., dynamically updated block titles). If hidden, users become confused about why list content is restricted or missing.
**Action:** When working on TUI components, always ensure that background filtering or view states are visibly reflected in headers, block titles, or status bars.
## 2026-04-13 - Explícitos mensajes de estado vacío en listas filtradas
**Learning:** Dejar las listas vacías cuando se aplica un filtro puede confundir al usuario, haciéndole creer que hay un error de carga en lugar de una búsqueda sin resultados.
**Action:** Siempre proporcionar estados vacíos útiles que le informen al usuario por qué la lista está vacía (e.g., 'No hay temas que coincidan con XYZ').
## 2024-05-18 - Intercepting Global Shortcuts to Clear Local State
**Learning:** In terminal applications, users intuitively press `Esc` to clear local states like active search filters before they expect to be navigated away from the current view. If global shortcuts (like 'Back to Dashboard') supersede this, users experience frustrating, accidental context loss.
**Action:** Always intercept cancellation keys (like `Esc` or `Backspace`) to clear local states (e.g. search filters, selections) first, before falling back to global navigation actions.
