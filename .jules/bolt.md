## 2023-10-27 - Zero-allocation substring matching for UI list filters
**Learning:** Using `.to_lowercase().contains()` inside loops for filtering lists (like TUI lists) allocates memory on every iteration, causing unnecessary CPU overhead and garbage collection pressure.
**Action:** Use a sliding window approach with bytes `haystack.as_bytes().windows(needle_bytes.len()).any(|w| w.eq_ignore_ascii_case(needle_bytes))` to perform zero-allocation, case-insensitive substring matching for ASCII strings, which is significantly faster and more memory efficient.
