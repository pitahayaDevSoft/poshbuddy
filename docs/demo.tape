# Output path for the animated GIF
Output docs/demo.gif

# Visual Settings
Set FontSize 10
Set FontFamily "ZedMono NFM"
Set LineHeight 1.0
Set Width 1000
Set Height 600
Set Theme "Builtin Dark"
Set Padding 10

# Command to launch the application (hidden from the viewer)
Hide
Type "./target/release/poshbuddy"
Enter
Sleep 3s
Show

# --- Welcome Screen ---
Sleep 2.5s
Enter
Sleep 2.5s

# --- Themes View (Tab 1) ---
Down
Sleep 1.5s
Down
Sleep 1.5s
Down
Sleep 1.5s

# Switch to Fonts View (Tab 2)
Tab
Sleep 2s

# --- Fonts View (Tab 2) ---
Down
Sleep 1s
Down
Sleep 1s
Down
Sleep 1s

# Install the selected font
Enter
Sleep 6s

# Dismiss the success modal
Space
Sleep 1.5s

# Switch to Segments View (Tab 3)
Tab
Sleep 2s

# --- Segments View (Tab 3) ---
Down
Sleep 1s
Down
Sleep 1s

# Toggle the selected segment
Enter
Sleep 3s

# Dismiss the success modal
Space
Sleep 1.5s

# Switch back to Themes View to complete the tour
Tab
Sleep 2s

# Quit the application
Type "q"
Sleep 1.5s
