use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::worktree;

pub fn run(feature_name: &str, config: &Config) -> anyhow::Result<()> {
    let feature_dir = config.features_dir.join(feature_name);

    if !feature_dir.exists() {
        anyhow::bail!("feature '{feature_name}' not found");
    }

    let worktrees = collect_worktrees(&feature_dir);
    if worktrees.is_empty() {
        anyhow::bail!("no worktrees found in feature '{feature_name}'");
    }

    let mut main_branch = None;

    for (repo_name, worktree_path) in &worktrees {
        let source_repo = find_source_repo(worktree_path)?;
        let branch = worktree::get_default_branch(&source_repo);

        if let Some(existing) = &main_branch {
            if *existing != branch {
                anyhow::bail!("inconsistent default branches detected: '{existing}' vs '{branch}'");
            }
        } else {
            main_branch = Some(branch);
        }

        println!("syncing {repo_name}...");
        worktree::fetch_worktree(worktree_path)?;
        worktree::rebase_worktree(worktree_path, main_branch.as_ref().unwrap())?;
        println!("  {repo_name} synced");
    }

    println!("feature '{feature_name}' synced successfully");

    Ok(())
}

fn collect_worktrees(feature_dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    let mut worktrees = Vec::new();

    if let Ok(entries) = fs::read_dir(feature_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                worktrees.push((name, path));
            }
        }
    }

    worktrees
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
        let result = run("nonexistent", &env.config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_sync_no_worktrees() {
        let env = TestEnv::new();
        let feature_dir = env.config.features_dir.join("empty-feature");
        fs::create_dir_all(&feature_dir).unwrap();

        let result = run("empty-feature", &env.config);
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

        let result = run("sync-test", &env.config);
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

        let result = run("multi-sync", &env.config);
        assert!(result.is_ok(), "sync failed: {:?}", result.err());
    }

    #[test]
    fn test_sync_worktree_is_up_to_date_after() {
        let env = TestEnv::new();
        let bare = env.setup_bare_repo("repo-2");
        let source = env.setup_source_repo("repo-2", &bare);

        env.create_worktree("up-to-date", "repo-2", &source);
        TestEnv::add_commit_to_main(&source);

        run("up-to-date", &env.config).unwrap();

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
}
