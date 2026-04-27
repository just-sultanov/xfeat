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

    let _ = Command::new("git")
        .args(["-C", source_repo.to_str().unwrap(), "worktree", "prune"])
        .output();

    let mut args: Vec<String> = vec![
        "-C".into(),
        source_repo.to_str().unwrap().into(),
        "worktree".into(),
        "add".into(),
        worktree_path.to_str().unwrap().into(),
    ];

    if let Some(from) = from_branch {
        if branch_exists(source_repo, from) {
            args.push(from.into());
        } else {
            args.push(format!("origin/{from}"));
        }
    }

    let branch_exists_locally = branch_exists(source_repo, branch_name);

    if !branch_exists_locally {
        args.push("-b".into());
    }
    args.push(branch_name.into());

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
        .args([
            "-C",
            repo.to_str().unwrap(),
            "fetch",
            "origin",
            "--prune",
            "--prune-tags",
        ])
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    struct TestRepo {
        tmp: PathBuf,
        repo_path: PathBuf,
    }

    impl TestRepo {
        fn new() -> Self {
            let unique = format!(
                "xfeat-wt-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            let tmp = std::env::temp_dir().join(unique);
            let repo_path = tmp.join("source-repo");

            fs::create_dir_all(&repo_path).unwrap();

            Command::new("git")
                .args(["init", repo_path.to_str().unwrap()])
                .status()
                .expect("failed to init git repo");

            fs::write(repo_path.join("README.md"), "initial").unwrap();

            Command::new("git")
                .args(["-C", repo_path.to_str().unwrap(), "add", "."])
                .status()
                .expect("failed to add files");

            Command::new("git")
                .args([
                    "-C",
                    repo_path.to_str().unwrap(),
                    "commit",
                    "-m",
                    "initial commit",
                ])
                .status()
                .expect("failed to commit");

            Self { tmp, repo_path }
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.tmp);
        }
    }

    #[test]
    fn test_create_worktree_after_stale_worktree() {
        let test_repo = TestRepo::new();
        let worktree_path = test_repo.tmp.join("features").join("stale-test");

        fs::create_dir_all(worktree_path.parent().unwrap()).unwrap();

        let status = Command::new("git")
            .args([
                "-C",
                test_repo.repo_path.to_str().unwrap(),
                "worktree",
                "add",
                worktree_path.to_str().unwrap(),
                "-b",
                "stale-test",
            ])
            .status()
            .expect("failed to create initial worktree");
        assert!(status.success(), "initial worktree creation failed");

        fs::remove_dir_all(&worktree_path).unwrap();

        let worktree_list = Command::new("git")
            .args([
                "-C",
                test_repo.repo_path.to_str().unwrap(),
                "worktree",
                "list",
                "--porcelain",
            ])
            .output()
            .expect("failed to list worktrees");
        let output = String::from_utf8_lossy(&worktree_list.stdout);
        assert!(
            output.contains("stale-test"),
            "worktree should be registered as stale before prune"
        );

        let result = create_worktree(&test_repo.repo_path, &worktree_path, None, "stale-test");
        assert!(
            result.is_ok(),
            "create_worktree should succeed after stale worktree, got: {:?}",
            result.err()
        );
        assert!(worktree_path.exists(), "worktree directory should exist");
    }
}
