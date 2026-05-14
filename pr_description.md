⚡ Bolt: Implement theme preview debouncing and caching

💡 What:
Added a debounce delay (150ms) before loading theme previews, avoiding the immediate execution of `oh-my-posh` for each keystroke during search. Also introduced a `theme_preview_cache` in the `App` state to avoid redundantly regenerating previews for previously selected or previewed themes.

🎯 Why:
To prevent spamming the system with `oh-my-posh` child processes when rapidly typing or deleting characters in the search bar, which was causing significant performance hits. Caching ensures we instantly update the preview if we already calculated it before, improving responsiveness.

📊 Impact:
Significantly reduces the number of spawned processes and I/O operations while navigating and filtering themes. This creates a much smoother UI experience, particularly on slower systems where spawning processes can block event loops.

🔬 Measurement:
While a formal, automated benchmark is impractical to attach due to the nature of TUI event loops, the number of child processes created when rapidly typing "abc" drops from 3 to 1.
