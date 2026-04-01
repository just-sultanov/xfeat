use clap::{Parser, Subcommand};

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
}
