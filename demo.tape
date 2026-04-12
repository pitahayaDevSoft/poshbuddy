# Output configuration
Output demo.gif

# Terminal settings
Set FontSize 18
Set Width 1200
Set Height 850
Set WindowBar Colorful
Set BorderRadius 10
Set Padding 20
Set Theme "Catppuccin Mocha"
Set FontFamily "CaskaydiaCove Nerd Font"

# --- Start of Demo ---

# 1. Run the application (Hidden from the GIF)
Hide
Type "cargo run"
Enter
# Wait for compilation and startup
Sleep 6s
Show

# 2. NEW: Welcome Screen / Quick Actions
# Navigate down through the new actions
Down 1
Sleep 800ms
Down 1
Sleep 800ms
Down 1
Sleep 800ms
Down 1
Sleep 800ms

# 3. Jump to Themes Explorer (Pressing 't')
Type "t"
Sleep 1.5s

# 4. Navigate Themes with Live Previews
Down 1
Sleep 2.5s
Down 1
Sleep 2.5s
Down 1
Sleep 2.5s

# 5. Show Dynamic Filtering
Type "bubbles"
Sleep 2s
# Clear filter
Backspace 7
Sleep 1s

# 6. Switch to Fonts View (Pressing '2')
Type "2"
Sleep 2s
Down 2
Sleep 1s

# 7. Back to Themes and Apply one
Type "1"
Sleep 500ms
Down 3
Sleep 1s
Enter

# 8. Success Screen
Sleep 4s

# Exit
Type "q"
Sleep 1s
