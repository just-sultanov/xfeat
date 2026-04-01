use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("repository '{0}' not found in repos directory")]
    #[allow(dead_code)]
    RepoNotFound(String),

    #[error("worktree already exists at '{0}'")]
    WorktreeExists(PathBuf),

    #[error("git command failed: {0}")]
    GitCommand(String),

    #[error("rebase conflict in '{0}': {1}")]
    RebaseConflict(String, String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
