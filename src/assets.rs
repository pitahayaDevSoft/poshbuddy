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
            category: "Version Control".to_string(),
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
            name: "Python info".to_string(),
            segment_type: "python".to_string(),
            description: "Shows active Python version/virtualenv.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Rust info".to_string(),
            segment_type: "rust".to_string(),
            description: "Shows active Rust compiler version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Go info".to_string(),
            segment_type: "go".to_string(),
            description: "Shows active Go version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: ".NET info".to_string(),
            segment_type: "dotnet".to_string(),
            description: "Shows active .NET SDK version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "PHP version".to_string(),
            segment_type: "php".to_string(),
            description: "Shows active PHP version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Ruby version".to_string(),
            segment_type: "ruby".to_string(),
            description: "Shows active Ruby version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Java version".to_string(),
            segment_type: "java".to_string(),
            description: "Shows active Java JDK version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Flutter info".to_string(),
            segment_type: "flutter".to_string(),
            description: "Shows active Flutter SDK version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Zig version".to_string(),
            segment_type: "zig".to_string(),
            description: "Shows active Zig version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Package version".to_string(),
            segment_type: "package".to_string(),
            description: "Shows current project package version (npm/Cargo).".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Docker".to_string(),
            segment_type: "docker".to_string(),
            description: "Shows current Docker status and context.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "AWS info".to_string(),
            segment_type: "aws".to_string(),
            description: "Shows active AWS profile and region.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Azure info".to_string(),
            segment_type: "az".to_string(),
            description: "Shows active Azure subscription.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "GCP info".to_string(),
            segment_type: "gcp".to_string(),
            description: "Shows active GCP project.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Kubernetes context".to_string(),
            segment_type: "kubectl".to_string(),
            description: "Shows active kubectl context and namespace.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Terraform workspace".to_string(),
            segment_type: "terraform".to_string(),
            description: "Shows active Terraform workspace.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Cloudflare".to_string(),
            segment_type: "cf".to_string(),
            description: "Shows active Cloudflare context.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Helm status".to_string(),
            segment_type: "helm".to_string(),
            description: "Shows active Helm chart version.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "System Time".to_string(),
            segment_type: "time".to_string(),
            description: "Shows current system time.".to_string(),
            category: "Time".to_string(),
        },
        SegmentAsset {
            name: "Operating System".to_string(),
            segment_type: "os".to_string(),
            description: "Shows operating system logo.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Shell version".to_string(),
            segment_type: "shell".to_string(),
            description: "Shows active shell name and version.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Spotify track".to_string(),
            segment_type: "spotify".to_string(),
            description: "Shows currently playing Spotify track.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Weather info".to_string(),
            segment_type: "weather".to_string(),
            description: "Shows current local weather.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "System Memory".to_string(),
            segment_type: "sysinfo".to_string(),
            description: "Shows system CPU or memory usage.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "WakaTime".to_string(),
            segment_type: "wakatime".to_string(),
            description: "Shows active WakaTime coding stats.".to_string(),
            category: "System".to_string(),
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
                    category: "Version Control".to_string(),
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
                    name: "Python info".to_string(),
                    segment_type: "python".to_string(),
                    description: "Shows active Python version/virtualenv.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Rust info".to_string(),
                    segment_type: "rust".to_string(),
                    description: "Shows active Rust compiler version.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Go info".to_string(),
                    segment_type: "go".to_string(),
                    description: "Shows active Go version.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: ".NET info".to_string(),
                    segment_type: "dotnet".to_string(),
                    description: "Shows active .NET SDK version.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "PHP version".to_string(),
                    segment_type: "php".to_string(),
                    description: "Shows active PHP version.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Ruby version".to_string(),
                    segment_type: "ruby".to_string(),
                    description: "Shows active Ruby version.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Java version".to_string(),
                    segment_type: "java".to_string(),
                    description: "Shows active Java JDK version.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Flutter info".to_string(),
                    segment_type: "flutter".to_string(),
                    description: "Shows active Flutter SDK version.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Zig version".to_string(),
                    segment_type: "zig".to_string(),
                    description: "Shows active Zig version.".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Package version".to_string(),
                    segment_type: "package".to_string(),
                    description: "Shows current project package version (npm/Cargo).".to_string(),
                    category: "Development".to_string(),
                },
                SegmentAsset {
                    name: "Docker".to_string(),
                    segment_type: "docker".to_string(),
                    description: "Shows current Docker status and context.".to_string(),
                    category: "Cloud".to_string(),
                },
                SegmentAsset {
                    name: "AWS info".to_string(),
                    segment_type: "aws".to_string(),
                    description: "Shows active AWS profile and region.".to_string(),
                    category: "Cloud".to_string(),
                },
                SegmentAsset {
                    name: "Azure info".to_string(),
                    segment_type: "az".to_string(),
                    description: "Shows active Azure subscription.".to_string(),
                    category: "Cloud".to_string(),
                },
                SegmentAsset {
                    name: "GCP info".to_string(),
                    segment_type: "gcp".to_string(),
                    description: "Shows active GCP project.".to_string(),
                    category: "Cloud".to_string(),
                },
                SegmentAsset {
                    name: "Kubernetes context".to_string(),
                    segment_type: "kubectl".to_string(),
                    description: "Shows active kubectl context and namespace.".to_string(),
                    category: "Cloud".to_string(),
                },
                SegmentAsset {
                    name: "Terraform workspace".to_string(),
                    segment_type: "terraform".to_string(),
                    description: "Shows active Terraform workspace.".to_string(),
                    category: "Cloud".to_string(),
                },
                SegmentAsset {
                    name: "Cloudflare".to_string(),
                    segment_type: "cf".to_string(),
                    description: "Shows active Cloudflare context.".to_string(),
                    category: "Cloud".to_string(),
                },
                SegmentAsset {
                    name: "Helm status".to_string(),
                    segment_type: "helm".to_string(),
                    description: "Shows active Helm chart version.".to_string(),
                    category: "Cloud".to_string(),
                },
                SegmentAsset {
                    name: "System Time".to_string(),
                    segment_type: "time".to_string(),
                    description: "Shows current system time.".to_string(),
                    category: "Time".to_string(),
                },
                SegmentAsset {
                    name: "Operating System".to_string(),
                    segment_type: "os".to_string(),
                    description: "Shows operating system logo.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Shell version".to_string(),
                    segment_type: "shell".to_string(),
                    description: "Shows active shell name and version.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Spotify track".to_string(),
                    segment_type: "spotify".to_string(),
                    description: "Shows currently playing Spotify track.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "Weather info".to_string(),
                    segment_type: "weather".to_string(),
                    description: "Shows current local weather.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "System Memory".to_string(),
                    segment_type: "sysinfo".to_string(),
                    description: "Shows system CPU or memory usage.".to_string(),
                    category: "System".to_string(),
                },
                SegmentAsset {
                    name: "WakaTime".to_string(),
                    segment_type: "wakatime".to_string(),
                    description: "Shows active WakaTime coding stats.".to_string(),
                    category: "System".to_string(),
                },
            ]
        );
    }
}
