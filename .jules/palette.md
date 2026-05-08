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
## 2024-05-19 - Maintain Context During Progress Modals
**Learning:** In TUI applications, intercepting progress or loading states at the top-level UI render loop replaces the main view, causing a jarring visual context switch (leaving the user with a floating modal on a blank screen).
**Action:** Allow the main UI to render as the background and overlay the progress modal within the main view's rendering function to prevent context loss.
## 2024-05-20 - Visual Progress over Plain Text
**Learning:** For long-running operations in TUI applications, representing progress as raw text percentages (e.g. "Progress: 50%") provides inferior feedback compared to visual representations.
**Action:** When working on Ratatui TUIs, use visual widgets like `ratatui::widgets::Gauge` for progress states to improve user perception of speed and completion, making sure to render them as overlays.
## 2024-11-20 - Actionable Empty States
**Learning:** For TUI lists with empty fallback states (e.g. after searching/filtering), users might not realize a filter is active and think the application failed to load data. The empty state message needs to be helpful.
**Action:** When creating empty states, provide explicit inline guidance on how to recover or clear the current state (e.g., '(Press Esc to clear search)').
## 2024-05-21 - Contextual Empty States in Detail Panels
**Learning:** When a master-detail view has an active filter that yields zero results, the detail panel should not prompt the user to "Select an item to continue". This is confusing because there are no items to select.
**Action:** Always update detail/preview panels to contextually acknowledge the empty state of the master list (e.g., "No results match your search. Press Esc to clear filter.").
## 2024-11-23 - Synchronize Master-Detail Views During Search
**Learning:** In master-detail TUI layouts, changing the search filter updates the master list's content and selection index. If the detail/preview panel is not explicitly synchronized during this live search input, it will display orphaned or stale data from the previous state, confusing the user.
**Action:** When handling keystrokes (e.g., `Backspace`, character inputs) that modify active filters in a master-detail view, always re-trigger the detail loading logic (e.g., `load_theme_preview`) for the newly highlighted item to maintain UI consistency.

## 2024-11-25 - Explicit Text for Disabled Menu Items
**Learning:** While dimming the color of disabled menu items provides a visual cue, it doesn't explain *why* the item is disabled. Users might think it's a bug or that they need to enable a prerequisite setting, rather than realizing the feature is simply "Coming Soon".
**Action:** When rendering disabled items in a TUI menu, always append an explicit textual explanation (like `[Coming Soon]`) to the label to manage user expectations.
