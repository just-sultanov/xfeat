use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

pub fn create_worktree(
    source_repo: &Path,
    worktree_path: &Path,
    from_branch: Option<&str>,
    branch_name: &str,
) -> Result<()> {
    if worktree_path.exists() {
        return Err(Error::WorktreeExists(worktree_path.to_path_buf()));
    }

    let mut args = vec![
        "-C",
        source_repo.to_str().unwrap(),
        "worktree",
        "add",
        worktree_path.to_str().unwrap(),
    ];

    if let Some(from) = from_branch {
        args.push(from);
    }

    args.push("-b");
    args.push(branch_name);

    let output = Command::new("git")
        .args(&args)
        .output()
        .map_err(|e| Error::GitCommand(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitCommand(stderr.trim().to_string()));
    }

    Ok(())
}

pub fn fetch_repo(repo: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["-C", repo.to_str().unwrap(), "fetch", "origin"])
        .output()
        .map_err(|e| Error::GitCommand(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitCommand(stderr.trim().to_string()));
    }

    Ok(())
}

pub fn branch_exists(repo: &Path, branch: &str) -> bool {
    let local = Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "rev-parse",
            "--verify",
            &format!("refs/heads/{branch}"),
        ])
        .output();

    if local.is_ok_and(|o| o.status.success()) {
        return true;
    }

    let remote = Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "rev-parse",
            "--verify",
            &format!("refs/remotes/{branch}"),
        ])
        .output();

    remote.is_ok_and(|o| o.status.success())
}

pub fn is_branch_in_use(repo: &Path, branch: &str) -> Option<std::path::PathBuf> {
    let output = Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "worktree",
            "list",
            "--porcelain",
        ])
        .output()
        .ok()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut current_worktree: Option<&str> = None;

    for line in output_str.lines() {
        if line.is_empty() {
            continue;
        }
        if let Some(path) = line.strip_prefix("worktree ") {
            current_worktree = Some(path);
        } else if let Some(b) = line.strip_prefix("branch ") {
            let branch_name = b
                .strip_prefix("refs/heads/")
                .or_else(|| b.strip_prefix("refs/remotes/"))
                .unwrap_or(b);
            if branch_name == branch
                && let Some(wt) = current_worktree
            {
                return Some(std::path::PathBuf::from(wt));
            }
        }
    }

    None
}

/// Determines the default branch of a git repository.
/// Tries `git symbolic-ref refs/remotes/origin/HEAD`, falls back to "main".
pub fn get_default_branch(repo: &Path) -> String {
    let output = Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "symbolic-ref",
            "--short",
            "refs/remotes/origin/HEAD",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if branch.is_empty() {
                "main".to_string()
            } else {
                branch
                    .strip_prefix("origin/")
                    .unwrap_or(&branch)
                    .to_string()
            }
        }
        _ => "main".to_string(),
    }
}

/// Fetches latest changes from the remote in the worktree.
pub fn fetch_worktree(worktree_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["-C", worktree_path.to_str().unwrap(), "fetch", "origin"])
        .output()
        .map_err(|e| Error::GitCommand(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitCommand(stderr.trim().to_string()));
    }

    Ok(())
}

/// Rebases the worktree onto the latest main branch.
pub fn rebase_worktree(worktree_path: &Path, main_branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args([
            "-C",
            worktree_path.to_str().unwrap(),
            "rebase",
            &format!("origin/{main_branch}"),
        ])
        .output()
        .map_err(|e| Error::GitCommand(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr_str = stderr.trim().to_string();

        if stderr_str.contains("conflict") || output.status.code() == Some(1) {
            return Err(Error::RebaseConflict(
                worktree_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                stderr_str,
            ));
        }

        return Err(Error::GitCommand(stderr_str));
    }

    Ok(())
}
