use crate::config::Config;
use crate::error::Error;
use crate::worktree;

pub fn run(feature_name: &str, repos: &[String], config: &Config) -> anyhow::Result<()> {
    let feature_dir = config.features_dir.join(feature_name);

    if !feature_dir.exists() {
        anyhow::bail!("feature '{feature_name}' not found. Use `xf new` to create it first.",);
    }

    println!("Feature '{feature_name}' already exists.");

    let mut added_count = 0;

    for repo_name in repos {
        let repo_path = config.repos_dir.join(repo_name);
        if !repo_path.exists() || !worktree::is_git_repo(&repo_path) {
            anyhow::bail!("repository '{repo_name}' not found or is not a git repository");
        }

        let worktree_path = feature_dir.join(repo_name);

        if worktree_path.exists() {
            println!("  Skipping: {repo_name} (worktree exists)");
            continue;
        }

        println!("  Creating: {repo_name}");
        worktree::create_worktree(&repo_path, &worktree_path, feature_name).map_err(
            |e| match e {
                Error::WorktreeExists(_) => {
                    anyhow::anyhow!("worktree already exists at '{}'", worktree_path.display())
                }
                other => anyhow::anyhow!(other),
            },
        )?;
        added_count += 1;
    }

    if added_count == 0 {
        println!("No new worktrees added.");
    } else {
        println!("Feature '{feature_name}' updated.");
    }

    Ok(())
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
                "xfeat-test-{}-{}",
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

        fn setup_repo(&self, name: &str) -> String {
            let repo_path = self.config.repos_dir.join(name);

            let status = Command::new("git")
                .args([
                    "clone",
                    "https://github.com/just-sultanov/xfeat",
                    repo_path.to_str().unwrap(),
                ])
                .status()
                .expect("failed to clone repo");

            assert!(status.success(), "git clone failed");
            name.to_string()
        }

        fn setup_feature_with_worktree(&self, feature_name: &str, repo_name: &str) {
            let feature_dir = self.config.features_dir.join(feature_name);
            fs::create_dir_all(&feature_dir).unwrap();

            let repo_path = self.config.repos_dir.join(repo_name);
            let worktree_path = feature_dir.join(repo_name);

            let status = Command::new("git")
                .args([
                    "-C",
                    repo_path.to_str().unwrap(),
                    "worktree",
                    "add",
                    worktree_path.to_str().unwrap(),
                    "-b",
                    feature_name,
                ])
                .status()
                .expect("failed to create worktree");

            assert!(status.success(), "worktree creation failed");
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.tmp);
        }
    }

    #[test]
    fn test_add_repo_to_existing_feature() {
        let env = TestEnv::new();
        let repo1 = env.setup_repo("repo-a");
        let repo2 = env.setup_repo("repo-b");

        env.setup_feature_with_worktree("add-test", &repo1);

        let result = run("add-test", &[repo2.clone()], &env.config);

        assert!(result.is_ok(), "add failed: {:?}", result.err());

        let worktree_path = env.config.features_dir.join("add-test").join(&repo2);
        assert!(worktree_path.exists(), "new worktree should exist");

        let branch_output = Command::new("git")
            .args([
                "-C",
                worktree_path.to_str().unwrap(),
                "branch",
                "--show-current",
            ])
            .output()
            .expect("failed to get branch");

        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();
        assert_eq!(branch, "add-test", "branch name should match feature name");
    }

    #[test]
    fn test_add_existing_worktree_skips() {
        let env = TestEnv::new();
        let repo = env.setup_repo("repo-c");

        env.setup_feature_with_worktree("skip-test", &repo);

        let result = run("skip-test", &[repo], &env.config);

        assert!(result.is_ok(), "add should succeed even when skipping");
    }

    #[test]
    fn test_add_feature_not_found() {
        let env = TestEnv::new();

        let result = run(
            "nonexistent-feature",
            &["some-repo".to_string()],
            &env.config,
        );

        assert!(result.is_err(), "expected error for missing feature");
        assert!(
            result.unwrap_err().to_string().contains("not found"),
            "error message should mention 'not found'"
        );
    }

    #[test]
    fn test_add_repo_not_found() {
        let env = TestEnv::new();

        let feature_dir = env.config.features_dir.join("empty-feature");
        fs::create_dir_all(&feature_dir).unwrap();

        let result = run(
            "empty-feature",
            &["nonexistent-repo".to_string()],
            &env.config,
        );

        assert!(result.is_err(), "expected error for missing repo");
        assert!(
            result.unwrap_err().to_string().contains("not found"),
            "error message should mention 'not found'"
        );
    }

    #[test]
    fn test_add_mixed_new_and_existing() {
        let env = TestEnv::new();
        let repo1 = env.setup_repo("repo-mix-1");
        let repo2 = env.setup_repo("repo-mix-2");
        let repo3 = env.setup_repo("repo-mix-3");

        env.setup_feature_with_worktree("mixed-test", &repo1);

        let result = run(
            "mixed-test",
            &[repo1.clone(), repo2.clone(), repo3.clone()],
            &env.config,
        );

        assert!(result.is_ok(), "add should succeed");

        let worktree2 = env.config.features_dir.join("mixed-test").join(&repo2);
        let worktree3 = env.config.features_dir.join("mixed-test").join(&repo3);

        assert!(worktree2.exists(), "new worktree for repo2 should exist");
        assert!(worktree3.exists(), "new worktree for repo3 should exist");
    }
}
