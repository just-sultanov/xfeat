use std::path::PathBuf;

use crate::config::Config;
use crate::error::Error;
use crate::worktree;

pub fn run(
    feature_name: &str,
    repos: &[String],
    from_branch: Option<&str>,
    branch_name: Option<&str>,
    config: &Config,
) -> anyhow::Result<()> {
    let feature_dir = config.features_dir.join(feature_name);

    if !feature_dir.exists() {
        anyhow::bail!("feature '{feature_name}' not found. Use `xf new` to create it first.",);
    }

    let target_branch = branch_name.unwrap_or(feature_name);

    if let Some(from) = from_branch {
        for repo_name in repos {
            let repo_path = find_repo_path(repo_name, config);
            if !repo_path.exists() || !worktree::is_git_repo(&repo_path) {
                anyhow::bail!("repository '{repo_name}' not found or is not a git repository");
            }
            worktree::fetch_repo(&repo_path)
                .map_err(|e| anyhow::anyhow!("failed to fetch repository '{repo_name}': {e}"))?;
            if !worktree::branch_exists(&repo_path, from) {
                anyhow::bail!("branch '{from}' does not exist in repository '{repo_name}'");
            }
        }
    }

    if branch_name.is_some() {
        for repo_name in repos {
            let repo_path = find_repo_path(repo_name, config);
            if !repo_path.exists() || !worktree::is_git_repo(&repo_path) {
                anyhow::bail!("repository '{repo_name}' not found or is not a git repository");
            }
            if let Some(worktree_path) = worktree::is_branch_in_use(&repo_path, target_branch) {
                anyhow::bail!(
                    "branch '{target_branch}' is already checked out at '{}'",
                    worktree_path.display()
                );
            }
        }
    }

    println!("Feature '{feature_name}' already exists.");

    let mut added_count = 0;

    for repo_name in repos {
        let repo_path = find_repo_path(repo_name, config);
        if !repo_path.exists() || !worktree::is_git_repo(&repo_path) {
            anyhow::bail!("repository '{repo_name}' not found or is not a git repository");
        }

        let worktree_path = feature_dir.join(repo_name);

        if worktree_path.exists() {
            println!("  Skipping: {repo_name} (worktree exists)");
            continue;
        }

        println!("  Creating: {repo_name}");
        worktree::create_worktree(&repo_path, &worktree_path, from_branch, target_branch).map_err(
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

fn find_repo_path(repo_name: &str, config: &Config) -> PathBuf {
    config.repos_dir.join(repo_name)
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

            std::fs::create_dir_all(&repos_dir).unwrap();
            std::fs::create_dir_all(&features_dir).unwrap();

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
            std::fs::create_dir_all(&feature_dir).unwrap();

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
            let _ = std::fs::remove_dir_all(&self.tmp);
        }
    }

    #[test]
    fn test_add_repo_to_existing_feature() {
        let env = TestEnv::new();
        let repo1 = env.setup_repo("repo-a");
        let repo2 = env.setup_repo("repo-b");

        env.setup_feature_with_worktree("add-test", &repo1);

        let result = run(
            "add-test",
            std::slice::from_ref(&repo2),
            None,
            None,
            &env.config,
        );

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

        let result = run("skip-test", &[repo], None, None, &env.config);

        assert!(result.is_ok(), "add should succeed even when skipping");
    }

    #[test]
    fn test_add_feature_not_found() {
        let env = TestEnv::new();

        let result = run(
            "nonexistent-feature",
            &["some-repo".to_string()],
            None,
            None,
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
        std::fs::create_dir_all(&feature_dir).unwrap();

        let result = run(
            "empty-feature",
            &["nonexistent-repo".to_string()],
            None,
            None,
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
            &[repo1, repo2.clone(), repo3.clone()],
            None,
            None,
            &env.config,
        );

        assert!(result.is_ok(), "add should succeed");

        let worktree2 = env.config.features_dir.join("mixed-test").join(&repo2);
        let worktree3 = env.config.features_dir.join("mixed-test").join(&repo3);

        assert!(worktree2.exists(), "new worktree for repo2 should exist");
        assert!(worktree3.exists(), "new worktree for repo3 should exist");
    }

    #[test]
    fn test_add_with_from_branch() {
        let env = TestEnv::new();
        let repo1 = env.setup_repo("repo-from-1");
        let repo2 = env.setup_repo("repo-from-2");

        env.setup_feature_with_worktree("from-test", &repo1);

        let result = run(
            "from-test",
            std::slice::from_ref(&repo2),
            Some("main"),
            None,
            &env.config,
        );

        assert!(result.is_ok(), "add with --from failed: {:?}", result.err());

        let worktree_path = env.config.features_dir.join("from-test").join(&repo2);
        assert!(worktree_path.exists(), "worktree should exist");

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
        assert_eq!(branch, "from-test", "branch name should match feature name");
    }

    #[test]
    fn test_add_with_from_branch_not_found() {
        let env = TestEnv::new();
        let repo = env.setup_repo("repo-from-missing");

        let feature_dir = env.config.features_dir.join("from-missing-test");
        std::fs::create_dir_all(&feature_dir).unwrap();

        let result = run(
            "from-missing-test",
            &[repo],
            Some("nonexistent-branch"),
            None,
            &env.config,
        );

        assert!(result.is_err(), "expected error for missing branch");
        assert!(
            result.unwrap_err().to_string().contains("does not exist"),
            "error message should mention 'does not exist'"
        );
    }

    #[test]
    fn test_add_with_custom_branch() {
        let env = TestEnv::new();
        let repo1 = env.setup_repo("repo-custom-1");
        let repo2 = env.setup_repo("repo-custom-2");

        env.setup_feature_with_worktree("custom-test", &repo1);

        let result = run(
            "custom-test",
            std::slice::from_ref(&repo2),
            None,
            Some("bugfix/JIRA-123"),
            &env.config,
        );

        assert!(
            result.is_ok(),
            "add with --branch failed: {:?}",
            result.err()
        );

        let worktree_path = env.config.features_dir.join("custom-test").join(&repo2);
        assert!(worktree_path.exists(), "worktree should exist");

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
        assert_eq!(
            branch, "bugfix/JIRA-123",
            "branch name should match custom branch"
        );
    }

    #[test]
    fn test_add_with_branch_already_in_use() {
        let env = TestEnv::new();
        let repo = env.setup_repo("repo-inuse-1");

        let feature_dir = env.config.features_dir.join("inuse-test");
        std::fs::create_dir_all(&feature_dir).unwrap();

        let repo_path = env.config.repos_dir.join(&repo);
        // Create a worktree in a DIFFERENT feature dir with the target branch name
        let other_feature_dir = env.config.features_dir.join("other-feature");
        std::fs::create_dir_all(&other_feature_dir).unwrap();
        let worktree1 = other_feature_dir.join(&repo);

        Command::new("git")
            .args([
                "-C",
                repo_path.to_str().unwrap(),
                "worktree",
                "add",
                worktree1.to_str().unwrap(),
                "-b",
                "bugfix/already-in-use",
            ])
            .status()
            .expect("failed to create worktree");

        // Now try to add the same repo to inuse-test with the same branch name
        let result = run(
            "inuse-test",
            std::slice::from_ref(&repo),
            None,
            Some("bugfix/already-in-use"),
            &env.config,
        );

        assert!(result.is_err(), "expected error for branch already in use");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("already checked out"),
            "error message should mention 'already checked out'"
        );
    }

    #[test]
    fn test_add_with_from_and_branch() {
        let env = TestEnv::new();
        let repo1 = env.setup_repo("repo-both-1");
        let repo2 = env.setup_repo("repo-both-2");

        env.setup_feature_with_worktree("both-test", &repo1);

        let result = run(
            "both-test",
            std::slice::from_ref(&repo2),
            Some("main"),
            Some("feature/custom-branch"),
            &env.config,
        );

        assert!(
            result.is_ok(),
            "add with --from and --branch failed: {:?}",
            result.err()
        );

        let worktree_path = env.config.features_dir.join("both-test").join(&repo2);
        assert!(worktree_path.exists(), "worktree should exist");

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
        assert_eq!(
            branch, "feature/custom-branch",
            "branch name should match custom branch"
        );
    }
}
