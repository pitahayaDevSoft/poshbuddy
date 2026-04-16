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
## 2024-04-16 - Zero-allocation counts for TUI tabs
**Learning:** Calling methods that construct and return filtered `Vec`s (like `.filtered_themes()`, which internally clones assets) just to get `.len()` for UI tabs causes significant performance degradation due to unnecessary memory allocations on every UI render cycle.
**Action:** Always provide and use dedicated `*_count()` methods that utilize iterators and `.count()` directly on collections without cloning elements or allocating intermediate `Vec`s when only the size of a filtered subset is needed.
