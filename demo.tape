# Output configuration
Output demo.gif

# Terminal settings
Set FontSize 18
Set Width 1200
Set Height 800
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
# Wait for compilation and startup before showing the terminal
Sleep 5s
# Skip onboarding screen if it appears
Enter
Sleep 1s
Show

# 2. Navigate Themes (Show preview for 5 themes with longer pauses)
Down 1
Sleep 2s
Down 1
Sleep 2s
Down 1
Sleep 2s
Down 1
Sleep 2s
Down 1
Sleep 2s
Up 4
Sleep 1s

# 3. Filter themes (Show search functionality)
Type "bubbles"
Sleep 3s
Backspace 7
Sleep 1s

# 4. Switch Tabs (Fonts & Plugins)
# Press 2 for Fonts
Type "2"
Sleep 2.5s
Down 2
Sleep 1.5s

# Press 3 for Plugins
Type "3"
Sleep 2.5s

# Back to Themes
Type "1"
Sleep 1s

# 5. Exit
Type "q"
Sleep 1s
