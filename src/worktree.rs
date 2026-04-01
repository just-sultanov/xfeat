use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

pub fn create_worktree(source_repo: &Path, worktree_path: &Path, branch: &str) -> Result<()> {
    if worktree_path.exists() {
        return Err(Error::WorktreeExists(worktree_path.to_path_buf()));
    }

    let output = Command::new("git")
        .args([
            "-C",
            source_repo.to_str().unwrap(),
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            branch,
        ])
        .output()
        .map_err(|e| Error::GitCommand(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitCommand(stderr.trim().to_string()));
    }

    Ok(())
}
