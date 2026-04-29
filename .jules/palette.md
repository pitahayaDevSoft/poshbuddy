## 2024-10-24 - Explicit State Rendering in TUIs
**Learning:** In Terminal User Interfaces (TUIs) like PoshBuddy (built with Ratatui), implicit states such as active search queries or filters must be explicitly rendered in the UI (e.g., dynamically updated block titles). If hidden, users become confused about why list content is restricted or missing.
**Action:** When working on TUI components, always ensure that background filtering or view states are visibly reflected in headers, block titles, or status bars.
## 2026-04-13 - Explícitos mensajes de estado vacío en listas filtradas
**Learning:** Dejar las listas vacías cuando se aplica un filtro puede confundir al usuario, haciéndole creer que hay un error de carga en lugar de una búsqueda sin resultados.
**Action:** Siempre proporcionar estados vacíos útiles que le informen al usuario por qué la lista está vacía (e.g., 'No hay temas que coincidan con XYZ').
## 2024-05-18 - Intercepting Global Shortcuts to Clear Local State
**Learning:** In terminal applications, users intuitively press `Esc` to clear local states like active search filters before they expect to be navigated away from the current view. If global shortcuts (like 'Back to Dashboard') supersede this, users experience frustrating, accidental context loss.
**Action:** Always intercept cancellation keys (like `Esc` or `Backspace`) to clear local states (e.g. search filters, selections) first, before falling back to global navigation actions.
## 2024-04-18 - Estilos de foco para elementos deshabilitados en TUIs
**Learning:** En las aplicaciones de terminal (TUI), los elementos de menú que están deshabilitados pero pueden recibir el foco del teclado (como opciones futuras o funciones en desarrollo) deben indicar visualmente que están seleccionados. Si un elemento deshabilitado no cambia su estilo cuando el usuario navega hacia él, se pierde el rastro del cursor y la navegación por teclado se vuelve confusa.
**Action:** Al diseñar estilos para listas o menús en Ratatui, siempre proporcionar una combinación de fondo tenue (e.g., `Color::DarkGray`) para el estado `is_disabled && is_selected`, asegurando así que el usuario sepa dónde está el cursor sin sugerir que la acción está disponible.

## 2026-04-25 - Explicit Empty States and Disabled Highlights
**Learning:** In TUI applications built with Ratatui, active filters yielding zero results should have explicit empty states rather than blank views, and empty state list items shouldn't be highlighted as if they are selectable.
**Action:** When rendering lists with an empty fallback message, I will conditionally disable the list's `highlight_style` and `highlight_symbol` (e.g. `if !items.is_empty()`) and explicitly update block titles to reflect active filters.
## 2024-04-28 - Hide Dismiss Hints on Non-Interactive States
**Learning:** Progress dialogs or loading states should not prompt users with misleading dismiss hints like "Press please wait to dismiss" when the action is non-interactive. It increases cognitive load and causes confusion.
**Action:** Always make dismiss hints optional in shared modal components. Only show interactive keyboard hints (like "Press any key to dismiss") when the user actually can interact with the dialog.
