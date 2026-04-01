use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "xfeat",
    about = "Manage git worktrees across multiple repositories"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new feature with worktrees for specified repositories
    New {
        /// Name of the feature (also used as branch name)
        feature_name: String,

        /// Repositories to include in the feature
        repos: Vec<String>,
    },

    /// List all features with their worktrees
    List,

    /// Remove a feature and its worktrees
    Remove {
        /// Name of the feature to remove
        feature_name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Sync a feature with the latest main branch from source repos
    Sync {
        /// Name of the feature to sync
        feature_name: String,
    },

    /// Generate shell initialization code
    Init {
        /// Shell to generate init code for
        shell: Shell,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Shell {
    Zsh,
}
