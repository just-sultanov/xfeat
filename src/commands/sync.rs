use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::worktree;

pub fn run(feature_name: &str, config: &Config, from_branch: Option<&str>) -> anyhow::Result<()> {
    let feature_dir = config.features_dir.join(feature_name);

    if !feature_dir.exists() {
        anyhow::bail!("feature '{feature_name}' not found");
    }

    let worktrees = collect_worktrees(&feature_dir);
    if worktrees.is_empty() {
        anyhow::bail!("no worktrees found in feature '{feature_name}'");
    }

    for (repo_name, worktree_path) in &worktrees {
        let branch = if let Some(from) = from_branch {
            from.to_string()
        } else {
            let source_repo = find_source_repo(worktree_path)?;
            worktree::get_default_branch(&source_repo)
        };

        println!("syncing {repo_name}...");
        worktree::fetch_worktree(worktree_path)?;
        worktree::rebase_worktree(worktree_path, &branch)?;
        println!("  {repo_name} synced");
    }

    println!("feature '{feature_name}' synced successfully");

    Ok(())
}

fn collect_worktrees(feature_dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    let mut worktrees = Vec::new();
    scan_recursive(feature_dir, feature_dir, &mut worktrees);
    worktrees
}

fn scan_recursive(dir: &Path, base: &Path, worktrees: &mut Vec<(String, std::path::PathBuf)>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.join(".git").exists() {
                    let rel_path = path.strip_prefix(base).unwrap_or(&path);
                    let name = rel_path.to_string_lossy().to_string();
                    worktrees.push((name, path));
                } else {
                    scan_recursive(&path, base, worktrees);
                }
            }
        }
    }
}

fn find_source_repo(worktree_path: &Path) -> anyhow::Result<std::path::PathBuf> {
    let output = std::process::Command::new("git")
        .args([
            "-C",
            worktree_path.to_str().unwrap(),
            "rev-parse",
            "--git-common-dir",
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;

    if !output.status.success() {
        anyhow::bail!("not a valid git worktree: {}", worktree_path.display());
    }

    let common_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let common_path = std::path::PathBuf::from(&common_dir);

    let source_repo = if common_path.is_absolute() {
        common_path.parent().unwrap_or(&common_path).to_path_buf()
    } else {
        worktree_path
            .join(&common_path)
            .parent()
            .unwrap()
            .to_path_buf()
    };

    Ok(source_repo)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command;

    use super::*;

    struct TestEnv {
        config: Config,
        tmp: PathBuf,
    }

    impl TestEnv {
        fn new() -> Self {
            let unique = format!(
                "xfeat-sync-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            let tmp = std::env::temp_dir().join(unique);
            let repos_dir = tmp.join("repos");
            let features_dir = tmp.join("features");

            fs::create_dir_all(&repos_dir).unwrap();
            fs::create_dir_all(&features_dir).unwrap();

            Self {
                config: Config {
                    repos_dir,
                    features_dir,
                },
                tmp,
            }
        }

        fn setup_bare_repo(&self, name: &str) -> PathBuf {
            let repo_path = self.config.repos_dir.join(name);
            fs::create_dir_all(&repo_path).unwrap();

            Command::new("git")
                .args(["init", "--bare", repo_path.to_str().unwrap()])
                .status()
                .expect("failed to init bare repo");

            repo_path
        }

        fn setup_source_repo(&self, name: &str, bare_path: &Path) -> PathBuf {
            let work_path = self.tmp.join(format!("source-{name}"));
            fs::create_dir_all(&work_path).unwrap();

            Command::new("git")
                .args([
                    "clone",
                    bare_path.to_str().unwrap(),
                    work_path.to_str().unwrap(),
                ])
                .status()
                .expect("failed to clone");

            let file_path = work_path.join("README.md");
            fs::write(&file_path, "initial").unwrap();

            Command::new("git")
                .args(["-C", work_path.to_str().unwrap(), "add", "."])
                .status()
                .expect("failed to add");

            Command::new("git")
                .args([
                    "-C",
                    work_path.to_str().unwrap(),
                    "commit",
                    "-m",
                    "initial commit",
                ])
                .status()
                .expect("failed to commit");

            Command::new("git")
                .args([
                    "-C",
                    work_path.to_str().unwrap(),
                    "push",
                    "-u",
                    "origin",
                    "main",
                ])
                .status()
                .expect("failed to push");

            work_path
        }

        fn setup_bare_repo_with_master(&self, name: &str) -> PathBuf {
            let repo_path = self.config.repos_dir.join(name);
            fs::create_dir_all(&repo_path).unwrap();

            Command::new("git")
                .args(["init", "--bare", repo_path.to_str().unwrap()])
                .status()
                .expect("failed to init bare repo");

            Command::new("git")
                .args([
                    "-C",
                    repo_path.to_str().unwrap(),
                    "symbolic-ref",
                    "HEAD",
                    "refs/heads/master",
                ])
                .status()
                .expect("failed to set HEAD to master");

            let clone_path = self.tmp.join(format!("temp-clone-{name}"));
            Command::new("git")
                .args([
                    "clone",
                    repo_path.to_str().unwrap(),
                    clone_path.to_str().unwrap(),
                ])
                .status()
                .expect("failed to clone");

            Command::new("git")
                .args(["-C", clone_path.to_str().unwrap(), "branch", "-m", "master"])
                .status()
                .expect("failed to rename to master");

            fs::write(clone_path.join("README.md"), "initial").unwrap();
            Command::new("git")
                .args(["-C", clone_path.to_str().unwrap(), "add", "."])
                .status()
                .expect("failed to add");
            Command::new("git")
                .args([
                    "-C",
                    clone_path.to_str().unwrap(),
                    "commit",
                    "-m",
                    "initial commit",
                ])
                .status()
                .expect("failed to commit");

            Command::new("git")
                .args([
                    "-C",
                    clone_path.to_str().unwrap(),
                    "push",
                    "-u",
                    "origin",
                    "master",
                ])
                .status()
                .expect("failed to push");

            Command::new("git")
                .args([
                    "-C",
                    clone_path.to_str().unwrap(),
                    "remote",
                    "set-head",
                    "origin",
                    "master",
                ])
                .status()
                .expect("failed to set origin/head");

            let _ = fs::remove_dir_all(clone_path);

            repo_path
        }

        fn setup_source_repo_with_master(&self, name: &str, bare_path: &Path) -> PathBuf {
            let work_path = self.tmp.join(format!("source-master-{name}"));
            fs::create_dir_all(&work_path).unwrap();

            Command::new("git")
                .args([
                    "clone",
                    bare_path.to_str().unwrap(),
                    work_path.to_str().unwrap(),
                ])
                .status()
                .expect("failed to clone");

            Command::new("git")
                .args(["-C", work_path.to_str().unwrap(), "checkout", "master"])
                .status()
                .expect("failed to checkout master");

            let file_path = work_path.join("README.md");
            fs::write(&file_path, "initial").unwrap();

            Command::new("git")
                .args(["-C", work_path.to_str().unwrap(), "add", "."])
                .status()
                .expect("failed to add");

            Command::new("git")
                .args([
                    "-C",
                    work_path.to_str().unwrap(),
                    "commit",
                    "-m",
                    "initial commit on master",
                ])
                .status()
                .expect("failed to commit");

            Command::new("git")
                .args([
                    "-C",
                    work_path.to_str().unwrap(),
                    "push",
                    "-u",
                    "origin",
                    "master",
                ])
                .status()
                .expect("failed to push");

            work_path
        }

        fn create_worktree(&self, feature_name: &str, repo_name: &str, source_path: &Path) {
            let worktree_path = self.config.features_dir.join(feature_name).join(repo_name);

            Command::new("git")
                .args([
                    "-C",
                    source_path.to_str().unwrap(),
                    "worktree",
                    "add",
                    worktree_path.to_str().unwrap(),
                    "-b",
                    feature_name,
                ])
                .status()
                .expect("failed to create worktree");
        }

        fn add_commit_to_main(source_path: &Path) {
            let file_path = source_path.join("update.txt");
            fs::write(&file_path, "updated").unwrap();

            Command::new("git")
                .args(["-C", source_path.to_str().unwrap(), "add", "."])
                .status()
                .expect("failed to add");

            Command::new("git")
                .args([
                    "-C",
                    source_path.to_str().unwrap(),
                    "commit",
                    "-m",
                    "update on main",
                ])
                .status()
                .expect("failed to commit");

            Command::new("git")
                .args(["-C", source_path.to_str().unwrap(), "push"])
                .status()
                .expect("failed to push");
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.tmp);
        }
    }

    #[test]
    fn test_sync_feature_not_found() {
        let env = TestEnv::new();
        let result = run("nonexistent", &env.config, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_sync_no_worktrees() {
        let env = TestEnv::new();
        let feature_dir = env.config.features_dir.join("empty-feature");
        fs::create_dir_all(&feature_dir).unwrap();

        let result = run("empty-feature", &env.config, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no worktrees"));
    }

    #[test]
    fn test_sync_single_worktree() {
        let env = TestEnv::new();
        let bare = env.setup_bare_repo("repo-1");
        let source = env.setup_source_repo("repo-1", &bare);

        env.create_worktree("sync-test", "repo-1", &source);
        TestEnv::add_commit_to_main(&source);

        let result = run("sync-test", &env.config, None);
        assert!(result.is_ok(), "sync failed: {:?}", result.err());
    }

    #[test]
    fn test_sync_multiple_worktrees() {
        let env = TestEnv::new();

        let bare1 = env.setup_bare_repo("repo-a");
        let source1 = env.setup_source_repo("repo-a", &bare1);

        let bare2 = env.setup_bare_repo("repo-b");
        let source2 = env.setup_source_repo("repo-b", &bare2);

        env.create_worktree("multi-sync", "repo-a", &source1);
        env.create_worktree("multi-sync", "repo-b", &source2);

        TestEnv::add_commit_to_main(&source1);
        TestEnv::add_commit_to_main(&source2);

        let result = run("multi-sync", &env.config, None);
        assert!(result.is_ok(), "sync failed: {:?}", result.err());
    }

    #[test]
    fn test_sync_worktree_is_up_to_date_after() {
        let env = TestEnv::new();
        let bare = env.setup_bare_repo("repo-2");
        let source = env.setup_source_repo("repo-2", &bare);

        env.create_worktree("up-to-date", "repo-2", &source);
        TestEnv::add_commit_to_main(&source);

        run("up-to-date", &env.config, None).unwrap();

        let worktree_path = env.config.features_dir.join("up-to-date").join("repo-2");

        let log = Command::new("git")
            .args([
                "-C",
                worktree_path.to_str().unwrap(),
                "log",
                "--oneline",
                "origin/main",
            ])
            .output()
            .expect("failed to get log");

        let log_str = String::from_utf8_lossy(&log.stdout);
        assert!(
            log_str.contains("update on main"),
            "worktree should have the main commit after sync"
        );
    }

    #[test]
    fn test_sync_with_different_default_branches() {
        let env = TestEnv::new();

        let bare1 = env.setup_bare_repo("repo-main");
        let source1 = env.setup_source_repo("repo-main", &bare1);

        let bare2 = env.setup_bare_repo_with_master("repo-master");
        let source2 = env.setup_source_repo_with_master("repo-master", &bare2);

        env.create_worktree("mixed-branches", "repo-main", &source1);
        env.create_worktree("mixed-branches", "repo-master", &source2);

        let file_path1 = source1.join("update.txt");
        fs::write(&file_path1, "updated-main").unwrap();
        Command::new("git")
            .args(["-C", source1.to_str().unwrap(), "add", "."])
            .status()
            .expect("failed to add");
        Command::new("git")
            .args([
                "-C",
                source1.to_str().unwrap(),
                "commit",
                "-m",
                "update on main",
            ])
            .status()
            .expect("failed to commit");
        Command::new("git")
            .args(["-C", source1.to_str().unwrap(), "push"])
            .status()
            .expect("failed to push");

        let file_path2 = source2.join("update.txt");
        fs::write(&file_path2, "updated-master").unwrap();
        Command::new("git")
            .args(["-C", source2.to_str().unwrap(), "add", "."])
            .status()
            .expect("failed to add");
        Command::new("git")
            .args([
                "-C",
                source2.to_str().unwrap(),
                "commit",
                "-m",
                "update on master",
            ])
            .status()
            .expect("failed to commit");
        Command::new("git")
            .args(["-C", source2.to_str().unwrap(), "push"])
            .status()
            .expect("failed to push");

        let result = run("mixed-branches", &env.config, None);
        assert!(
            result.is_ok(),
            "sync should succeed with different default branches: {:?}",
            result.err()
        );

        let wt2_path = env
            .config
            .features_dir
            .join("mixed-branches")
            .join("repo-master");

        let log2 = Command::new("git")
            .args(["-C", wt2_path.to_str().unwrap(), "log", "--oneline", "-1"])
            .output()
            .expect("failed to get log");
        let log2_str = String::from_utf8_lossy(&log2.stdout);
        assert!(
            log2_str.contains("update on master") || log2_str.contains("update on main"),
            "repo-master should have the update commit, got: {log2_str}"
        );
    }

    #[test]
    fn test_sync_with_explicit_from_branch() {
        let env = TestEnv::new();
        let bare = env.setup_bare_repo("repo-test");
        let source = env.setup_source_repo("repo-test", &bare);

        env.create_worktree("explicit-from", "repo-test", &source);

        let file_path = source.join("update.txt");
        fs::write(&file_path, "updated").unwrap();
        Command::new("git")
            .args(["-C", source.to_str().unwrap(), "add", "."])
            .status()
            .expect("failed to add");
        Command::new("git")
            .args(["-C", source.to_str().unwrap(), "commit", "-m", "update"])
            .status()
            .expect("failed to commit");
        Command::new("git")
            .args(["-C", source.to_str().unwrap(), "push"])
            .status()
            .expect("failed to push");

        let result = run("explicit-from", &env.config, Some("main"));
        assert!(
            result.is_ok(),
            "sync with explicit --from should succeed: {:?}",
            result.err()
        );
    }
}
