## 2024-04-11 - TUI List Search Allocation Bottleneck
**Learning:** In Ratatui applications with live-filtering lists, using `.to_lowercase().contains(...)` inside `iter().filter()` causes excessive memory allocations and CPU spikes per frame when typing search queries, due to creating a new lowercase `String` for every list item during the hot render loop.
**Action:** Use zero-allocation byte-level sliding windows (`.as_bytes().windows(len).any(|w| w.eq_ignore_ascii_case(needle))`) for case-insensitive substring matching in any hot filtering or rendering loops.
