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
            rollback(&created);
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

fn rollback(created: &[std::path::PathBuf]) {
    for path in created.iter().rev() {
        let _ = fs::remove_dir_all(path);
    }
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

        fn setup_repo(&self) -> String {
            let repo_name = "xfeat".to_string();
            let repo_path = self.config.repos_dir.join(&repo_name);

            let status = Command::new("git")
                .args([
                    "clone",
                    "https://github.com/just-sultanov/xfeat",
                    repo_path.to_str().unwrap(),
                ])
                .status()
                .expect("failed to clone repo");

            assert!(status.success(), "git clone failed");
            repo_name
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
        let repo_name = env.setup_repo();

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
        let repo_name = env.setup_repo();

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
}
