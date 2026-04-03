use clap::{ArgAction, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "xfeat",
    version = env!("XFEAT_VERSION"),
    disable_version_flag = true,
    about = "Manage git worktrees across multiple repositories",
    before_help = concat!(
        "xfeat v",
        env!("XFEAT_VERSION"),
        "@",
        env!("XFEAT_GIT_SHA"),
        " (",
        env!("XFEAT_BUILT_AT"),
        ")"
    ),
    help_template = "{before-help}{about}\n\n{usage-heading}\n  {usage}\n\n{all-args}{after-help}",
)]
pub struct Cli {
    /// Print version information
    #[arg(long, action = ArgAction::Version)]
    version: (),

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new feature directory
    New {
        /// Name of the feature
        feature_name: String,
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

    /// Add worktrees for repos to an existing feature
    Add {
        /// Name of the feature to add worktrees to
        feature_name: String,

        /// Repositories to add to the feature
        repos: Vec<String>,

        /// Create branch from a specific branch (e.g., develop, origin/develop)
        #[arg(long)]
        from: Option<String>,

        /// Custom branch name (defaults to feature name)
        #[arg(long)]
        branch: Option<String>,
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
