use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "poshbuddy")]
#[command(about = "The Professional Management Engine for Oh My Posh Configurations", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Set configuration
    Set {
        #[command(subcommand)]
        target: SetTarget,
    },
    /// Install resources
    Install {
        #[command(subcommand)]
        target: InstallTarget,
    },
    /// List resources
    List {
        #[command(subcommand)]
        target: ListTarget,
    },
}

#[derive(Subcommand)]
pub enum SetTarget {
    /// Set the active theme
    Theme { name: String },
}

#[derive(Subcommand)]
pub enum InstallTarget {
    /// Install a Nerd Font
    Font { name: String },
}

#[derive(Subcommand)]
pub enum ListTarget {
    /// List available themes
    Themes {
        #[arg(short, long)]
        local: bool,
        #[arg(short, long)]
        remote: bool,
    },
    /// List available fonts
    Fonts,
}
