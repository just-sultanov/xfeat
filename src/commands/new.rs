use std::fs;

use crate::config::Config;
use crate::worktree;

pub fn run(feature_name: &str, repos: &[String], config: &Config) -> anyhow::Result<()> {
    let feature_dir = config.features_dir.join(feature_name);

    if feature_dir.exists() {
        anyhow::bail!(
            "feature directory '{}' already exists",
            feature_dir.display()
        );
    }

    for repo_name in repos {
        let repo_path = config.repos_dir.join(repo_name);
        if !repo_path.exists() || !worktree::is_git_repo(&repo_path) {
            anyhow::bail!("repository '{repo_name}' not found or is not a git repository");
        }
    }

    fs::create_dir_all(&feature_dir)?;

    let mut created = Vec::new();

    for repo_name in repos {
        let repo_path = config.repos_dir.join(repo_name);
        let worktree_path = feature_dir.join(repo_name);

        if let Err(e) = worktree::create_worktree(&repo_path, &worktree_path, feature_name) {
            rollback(&created, &feature_dir);
            return Err(anyhow::anyhow!(e));
        }

        created.push(worktree_path);
    }

    println!(
        "Feature '{feature_name}' created at: {}",
        feature_dir.display()
    );

    Ok(())
}

fn rollback(created: &[std::path::PathBuf], feature_dir: &std::path::Path) {
    for path in created.iter().rev() {
        let _ = fs::remove_dir_all(path);
    }
    let _ = fs::remove_dir_all(feature_dir);
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
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.tmp);
        }
    }

    #[test]
    fn test_new_command_creates_worktree_and_branch() {
        let env = TestEnv::new();
        let repo_name = env.setup_repo("repo-1");

        let feature_name = "new-test";
        let result = run(feature_name, &[repo_name.clone()], &env.config);

        assert!(result.is_ok(), "run failed: {:?}", result.err());

        let feature_dir = env.config.features_dir.join(feature_name);
        let worktree_path = feature_dir.join(&repo_name);

        assert!(worktree_path.exists(), "worktree directory not created");
        assert!(
            worktree_path.join(".git").exists(),
            "worktree .git not found"
        );

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
        assert_eq!(branch, feature_name, "branch name mismatch");
    }

    #[test]
    fn test_new_command_fails_for_existing_feature() {
        let env = TestEnv::new();
        let repo_name = env.setup_repo("repo-2");

        let feature_name = "existing-test";
        let feature_dir = env.config.features_dir.join(feature_name);
        fs::create_dir_all(&feature_dir).unwrap();

        let result = run(feature_name, &[repo_name], &env.config);

        assert!(result.is_err(), "expected error for existing feature");
        assert!(
            result.unwrap_err().to_string().contains("already exists"),
            "error message should mention 'already exists'"
        );
    }

    #[test]
    fn test_new_command_fails_for_missing_repo() {
        let env = TestEnv::new();

        let result = run(
            "test-feature",
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
    fn test_new_command_creates_feature_directory() {
        let env = TestEnv::new();
        let repo_name = env.setup_repo("repo-3");

        let feature_name = "feature-dir-test";
        run(feature_name, &[repo_name.clone()], &env.config).unwrap();

        let feature_dir = env.config.features_dir.join(feature_name);
        assert!(feature_dir.is_dir(), "feature directory should exist");
        assert!(
            feature_dir.join(&repo_name).is_dir(),
            "worktree subdirectory should exist"
        );
    }

    #[test]
    fn test_new_command_worktree_linked_to_source() {
        let env = TestEnv::new();
        let repo_name = env.setup_repo("repo-4");
        let repo_path = env.config.repos_dir.join(&repo_name);

        let feature_name = "linked-test";
        run(feature_name, &[repo_name.clone()], &env.config).unwrap();

        let worktree_path = env.config.features_dir.join(feature_name).join(&repo_name);

        let output = Command::new("git")
            .args(["-C", repo_path.to_str().unwrap(), "worktree", "list"])
            .output()
            .expect("failed to list worktrees");

        let worktree_list = String::from_utf8_lossy(&output.stdout);
        assert!(
            worktree_list.contains(worktree_path.to_str().unwrap()),
            "worktree should be registered in source repo"
        );
    }

    #[test]
    fn test_new_command_multiple_repos_all_have_correct_branch() {
        let env = TestEnv::new();
        let repo1 = env.setup_repo("repo-5");
        let repo2 = env.setup_repo("repo-6");

        let feature_name = "multi-branch-test";
        run(feature_name, &[repo1.clone(), repo2.clone()], &env.config).unwrap();

        let feature_dir = env.config.features_dir.join(feature_name);

        for repo_name in [&repo1, &repo2] {
            let worktree_path = feature_dir.join(repo_name);
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
            assert_eq!(branch, feature_name, "branch should match feature name");
        }
    }

    #[test]
    fn test_new_command_worktree_is_valid_git_repo() {
        let env = TestEnv::new();
        let repo_name = env.setup_repo("repo-7");

        let feature_name = "valid-git-test";
        run(feature_name, &[repo_name.clone()], &env.config).unwrap();

        let worktree_path = env.config.features_dir.join(feature_name).join(&repo_name);

        let status = Command::new("git")
            .args(["-C", worktree_path.to_str().unwrap(), "status"])
            .status()
            .expect("git status should work in worktree");

        assert!(
            status.success(),
            "worktree should be a valid git repository"
        );
    }

    #[test]
    fn test_rollback_cleans_up_created_worktrees() {
        let env = TestEnv::new();
        let repo1 = env.setup_repo("repo-8");
        let repo2 = env.setup_repo("repo-9");

        let feature_name = "rollback-test";
        let feature_dir = env.config.features_dir.join(feature_name);
        fs::create_dir_all(&feature_dir).unwrap();

        // Simulate partial success: create worktree for repo1
        let repo1_path = env.config.repos_dir.join(&repo1);
        let worktree1 = feature_dir.join(&repo1);
        let worktree2 = feature_dir.join(&repo2);

        Command::new("git")
            .args([
                "-C",
                repo1_path.to_str().unwrap(),
                "worktree",
                "add",
                worktree1.to_str().unwrap(),
                "-b",
                feature_name,
            ])
            .status()
            .expect("failed to create first worktree");

        assert!(worktree1.exists(), "worktree1 should exist before rollback");

        // Simulate rollback (as if worktree2 creation failed)
        rollback(&[worktree1.clone(), worktree2.clone()], &feature_dir);

        assert!(!worktree1.exists(), "worktree1 should be cleaned up");
        assert!(!worktree2.exists(), "worktree2 should not exist");
        assert!(
            !feature_dir.exists(),
            "feature directory should be cleaned up"
        );
    }
}
