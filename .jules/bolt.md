## 2024-04-13 - Zero-allocation Case-Insensitive String Matching
**Learning:** Using `.to_lowercase().contains(...)` inside loops allocates new `String` objects, which creates a significant performance bottleneck during list filtering.
**Action:** For performant, zero-allocation case-insensitive ASCII substring matching in Rust loops, use byte-level sliding windows (e.g., `line.as_bytes().windows(len).any(|w| w.eq_ignore_ascii_case(needle))`) instead of `.to_lowercase().contains(...)`.
## 2023-10-27 - Zero-allocation substring matching for UI list filters
**Learning:** Using `.to_lowercase().contains()` inside loops for filtering lists (like TUI lists) allocates memory on every iteration, causing unnecessary CPU overhead and garbage collection pressure.
**Action:** Use a sliding window approach with bytes `haystack.as_bytes().windows(needle_bytes.len()).any(|w| w.eq_ignore_ascii_case(needle_bytes))` to perform zero-allocation, case-insensitive substring matching for ASCII strings, which is significantly faster and more memory efficient.
## 2026-04-15 - Graceful Background Task Termination
**Learning:** Background tokio tasks using `mpsc` channels for UI updates continue to consume resources (CPU/Memory) if the UI receiver drops during application shutdown and the sender's error is ignored.
**Action:** When using `tokio::sync::mpsc` channels, explicitly handle `tx.send().await` errors (e.g., `if tx.send(...).await.is_err() { return; }`) to gracefully terminate the task when the channel is closed. Avoid this pattern with `try_send()`, as it errors on full channels (`TrySendError::Full`), which can unintentionally abort tasks during traffic spikes.
## 2024-05-19 - Zero-allocation list length counting
**Learning:** In Ratatui-based TUIs, determining list item counts using methods that allocate and clone elements into new `Vec`s (e.g. `app.filtered_items().len()`) causes massive memory allocation overhead and Garbage Collection pressure during the frequent render loop.
**Action:** Implement and use iterator-based `_count()` methods (e.g., `.filter(...).count()`) instead of `.len()` on collected `Vec`s to perform zero-allocation counting directly inside rendering loops.
## 2024-05-19 - Cache local sets for O(N+M) TUI filtering
**Learning:** In Ratatui-based TUIs, comparing remote items against local items during the render loop using nested `O(N*M)` iterator scans (e.g. !self.local_items.iter().any(...)) creates massive frame latency as collections grow, and dynamically allocating a `HashSet` inside the method adds memory overhead per frame.
**Action:** To optimize O(N*M) lookups in TUI render loops (e.g., matching remote themes against local themes), cache a pre-computed `HashSet` of identifiers directly on the application state (`App` struct) and update it via message handlers. Do not dynamically allocate the `HashSet` inside the frequently called render/filter methods to avoid unnecessary heap allocations.
## 2024-04-27 - Unnecessary Allocation in TUI Navigation
**Learning:** In Ratatui-based TUI event handlers within this codebase, full list allocation methods (e.g., `filtered_fonts()`) were being called purely to check `.len()` for keyboard navigation boundaries, discarding the heavy `Vec` immediately after.
**Action:** Always prefer `_count()` methods (e.g., `filtered_fonts_count()`) for list boundary calculations during navigation to avoid O(N) allocation and deep copying on every arrow key press.
## 2024-05-24 - Zero-allocation List Element Lookups
**Learning:** In Rust TUI applications, fetching a single element from a filtered list using methods that allocate a full `Vec` (e.g. `app.filtered_themes().first()`) creates O(N) allocation overhead per keystroke during keyboard navigation or event handling.
**Action:** Implement zero-allocation lazy-iteration lookups (e.g., `filtered_theme_at(&self, index: usize)`) that lazily scan and return the Nth element, avoiding full collection allocations and deep cloning just to check a single item's state.
## 2024-05-30 - O(log N) deduplication over HashSet for small/medium collections
**Learning:** For application states maintaining collections like `themes`, using a separate `HashSet` for quick duplicate lookups doubles the memory overhead and forces string allocations during insertion.
**Action:** Instead of pairing a `Vec` and a `HashSet`, maintain the primary `Vec` sorted via `sort_by(...)` upon insertion, and use `binary_search_by(...)` for fast `O(log N)` lookups. This achieves near-instantaneous `O(log N)` lookups for hundreds of items with strictly zero extra heap allocation.
## 2025-01-20 - Avoid Vector Allocations in TUI Render Loops
**Learning:** Calling methods that map, filter, and allocate intermediate `Vec` collections (e.g. `app.filtered_themes()`) before mapping to Ratatui `ListItem` objects causes unnecessary string cloning and memory pressure during frequent render cycles.
**Action:** Iterate directly over the source application state collections, apply filters lazily, and construct `ListItem`s directly to prevent creating an intermediate, discarded collection during every frame.
## 2025-02-12 - Conditional Empty State Iterators for Ratatui Lists
**Learning:** To conditionally render an empty state message in a Ratatui `List` without breaking iterator chains or allocating a `Vec`, you can chain an `Option` mapped to `.into_iter()`. If the main list filtering relies on iterators, calling `.count()` on the source iterator might consume it.
**Action:** In Ratatui, `List::new()` accepts an iterator directly. To conditionally render an empty state message without allocating a `Vec` of `ListItem`s, use `.chain()` to append an optional 'empty message' iterator, relying on a separate `_count()` check to determine if the collection is empty.
## 2024-05-31 - Zero-allocation ListItem strings via Line/Span composition
**Learning:** Using the `format!` macro within iterative `.map()` closures to construct text for Ratatui `ListItem` objects causes unnecessary, severe heap allocation per item, per frame during the main TUI render loop.
**Action:** In Ratatui TUI render loops, avoid using the `format!` macro to construct strings for `ListItem` widgets. Instead, compose zero-allocation UI elements using `Line::from` and `Span::raw` by passing string slices (`&str`).

## 2024-05-18 - Optimized `contains_ignore_ascii_case`
**Learning:** `haystack.as_bytes().windows().any(...)` is a high overhead O(N*M) check that can be significantly improved by implementing a fast path that manually iterates and checks if the first byte matches the needle's uppercase or lowercase equivalent before invoking `eq_ignore_ascii_case`.
**Action:** When implementing custom string search operations, prioritize fast-path rejection on the first character to avoid the overhead of full slice equality checks and slice creations.
