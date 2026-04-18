## 2024-04-13 - Zero-allocation Case-Insensitive String Matching
**Learning:** Using `.to_lowercase().contains(...)` inside loops allocates new `String` objects, which creates a significant performance bottleneck during list filtering.
**Action:** For performant, zero-allocation case-insensitive ASCII substring matching in Rust loops, use byte-level sliding windows (e.g., `line.as_bytes().windows(len).any(|w| w.eq_ignore_ascii_case(needle))`) instead of `.to_lowercase().contains(...)`.
## 2023-10-27 - Zero-allocation substring matching for UI list filters
**Learning:** Using `.to_lowercase().contains()` inside loops for filtering lists (like TUI lists) allocates memory on every iteration, causing unnecessary CPU overhead and garbage collection pressure.
**Action:** Use a sliding window approach with bytes `haystack.as_bytes().windows(needle_bytes.len()).any(|w| w.eq_ignore_ascii_case(needle_bytes))` to perform zero-allocation, case-insensitive substring matching for ASCII strings, which is significantly faster and more memory efficient.
## 2026-04-15 - Graceful Background Task Termination
**Learning:** Background tokio tasks using `mpsc` channels for UI updates continue to consume resources (CPU/Memory) if the UI receiver drops during application shutdown and the sender's error is ignored.
**Action:** When using `tokio::sync::mpsc` channels, explicitly handle `tx.send().await` errors (e.g., `if tx.send(...).await.is_err() { return; }`) to gracefully terminate the task when the channel is closed.
## 2026-04-15 - Graceful Background Task Termination
**Learning:** Background tokio tasks using `mpsc` channels for UI updates continue to consume resources (CPU/Memory) if the UI receiver drops during application shutdown and the sender's error is ignored.
**Action:** When using `tokio::sync::mpsc` channels, explicitly handle `tx.send().await` errors (e.g., `if tx.send(...).await.is_err() { return; }`) to gracefully terminate the task when the channel is closed. Avoid this pattern with `try_send()`, as it errors on full channels (`TrySendError::Full`), which can unintentionally abort tasks during traffic spikes.
## 2024-05-19 - Zero-allocation list length tracking in render loops
**Learning:** Calling `filtered_items().len()` during the UI render loop allocates memory for a new Vector and clones all filtered string elements simply to return an integer length. This causes severe memory pressure on every frame in TUI applications.
**Action:** Implement and use dedicated `filtered_items_count()` methods that utilize iterators (e.g., `items.iter().filter(...).count()`) to calculate list lengths directly without performing any memory allocations or data cloning.
