use crate::app::{PluginAsset, SegmentAsset};

/// Returns the predefined list of legacy PowerShell plugins/modules supported by PoshBuddy.
pub fn get_default_plugins() -> Vec<PluginAsset> {
    vec![
        PluginAsset {
            name: "Terminal-Icons".to_string(),
            description: "Adds file and folder icons to your terminal outputs (ls, dir).".to_string(),
            module_name: "Terminal-Icons".to_string(),
            init_script: None,
        },
        PluginAsset {
            name: "zoxide (z Explorer)".to_string(),
            description: "A smarter cd command. It remembers which directories you use most often.".to_string(),
            module_name: "zoxide".to_string(),
            init_script: Some("if (Get-Command zoxide -ErrorAction SilentlyContinue) { zoxide init powershell --hook pwd | Out-String | Invoke-Expression }".to_string()),
        },
        PluginAsset {
            name: "PSReadLine Mastery".to_string(),
            description: "Enables Predictive IntelliSense (fish-like) and syntax highlighting.".to_string(),
            module_name: "PSReadLine".to_string(),
            init_script: Some("Set-PSReadLineOption -PredictionSource History\nSet-PSReadLineOption -PredictionViewStyle ListView".to_string()),
        },
    ]
}

/// Returns the standard predefined Oh My Posh segments for surgical manipulation.
pub fn get_default_segments() -> Vec<SegmentAsset> {
    vec![
        SegmentAsset {
            name: "Git Status".to_string(),
            segment_type: "git".to_string(),
            description: "Shows current branch and Git file status.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Path".to_string(),
            segment_type: "path".to_string(),
            description: "Shows current location in the file system.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Session (User)".to_string(),
            segment_type: "session".to_string(),
            description: "Shows current user and host.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Battery".to_string(),
            segment_type: "battery".to_string(),
            description: "Displays battery percentage and charging status.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Execution Time".to_string(),
            segment_type: "executiontime".to_string(),
            description: "Shows duration of the last command executed.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Node.js info".to_string(),
            segment_type: "node".to_string(),
            description: "Shows active Node version in the directory.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Docker".to_string(),
            segment_type: "docker".to_string(),
            description: "Shows current Docker status and context.".to_string(),
            category: "Cloud".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_plugins() {
        let plugins = get_default_plugins();
        assert_eq!(
            plugins,
            vec![
                PluginAsset {
                    name: "Terminal-Icons".to_string(),
                    description: "Adds file and folder icons to your terminal outputs (ls, dir).".to_string(),
                    module_name: "Terminal-Icons".to_string(),
                    init_script: None,
                },
                PluginAsset {
                    name: "zoxide (z Explorer)".to_string(),
                    description: "A smarter cd command. It remembers which directories you use most often.".to_string(),
                    module_name: "zoxide".to_string(),
                    init_script: Some("if (Get-Command zoxide -ErrorAction SilentlyContinue) { zoxide init powershell --hook pwd | Out-String | Invoke-Expression }".to_string()),
                },
                PluginAsset {
                    name: "PSReadLine Mastery".to_string(),
                    description: "Enables Predictive IntelliSense (fish-like) and syntax highlighting.".to_string(),
                    module_name: "PSReadLine".to_string(),
                    init_script: Some("Set-PSReadLineOption -PredictionSource History\nSet-PSReadLineOption -PredictionViewStyle ListView".to_string()),
                },
            ]
        );
    }

    #[test]
    fn test_get_default_segments() {
        let segments = get_default_segments();
        assert_eq!(
            segments,
            vec![
                SegmentAsset {
                    name: "Git Status".to_string(),
                    segment_type: "git".to_string(),
                    description: "Shows current branch and Git file status.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Path".to_string(),
                    segment_type: "path".to_string(),
                    description: "Shows current location in the file system.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Session (User)".to_string(),
                    segment_type: "session".to_string(),
                    description: "Shows current user and host.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Battery".to_string(),
                    segment_type: "battery".to_string(),
                    description: "Displays battery percentage and charging status.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Execution Time".to_string(),
                    segment_type: "executiontime".to_string(),
                    description: "Shows duration of the last command executed.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Node.js info".to_string(),
                    segment_type: "node".to_string(),
                    description: "Shows active Node version in the directory.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Docker".to_string(),
                    segment_type: "docker".to_string(),
                    description: "Shows current Docker status and context.".to_string(),
                    category: "Cloud".to_string(),
                },
            ]
        );
    }
}
