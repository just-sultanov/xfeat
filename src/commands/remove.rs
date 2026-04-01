use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;

/// Removes a feature directory and its worktrees.
///
/// Prompts for confirmation unless `skip_confirm` is true.
/// Warns about uncommitted changes in worktrees before removal.
pub fn run(feature_name: &str, skip_confirm: bool, config: &Config) -> anyhow::Result<()> {
    let feature_dir = config.features_dir.join(feature_name);

    if !feature_dir.exists() {
        anyhow::bail!("feature '{feature_name}' not found");
    }

    let worktrees = list_worktrees(&feature_dir);

    if worktrees.is_empty() {
        anyhow::bail!("feature '{feature_name}' exists but contains no worktrees");
    }

    let has_uncommitted = check_uncommitted_changes(&feature_dir);

    println!("Feature '{feature_name}' contains:");
    for (repo, branch) in &worktrees {
        let warning = if has_uncommitted.contains(repo) {
            " ⚠ has uncommitted changes"
        } else {
            ""
        };
        println!("  - {repo} ({branch}){warning}");
    }
    println!();

    if !skip_confirm {
        print!("Remove feature '{feature_name}'? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            println!("Aborted.");
            return Ok(());
        }
    }

    remove_worktrees(&feature_dir, &worktrees);

    if let Err(e) = fs::remove_dir_all(&feature_dir) {
        anyhow::bail!("failed to remove feature directory: {e}");
    }

    println!("Feature '{feature_name}' removed.");
    Ok(())
}

fn list_worktrees(feature_dir: &PathBuf) -> Vec<(String, String)> {
    let mut worktrees = Vec::new();

    if let Ok(entries) = fs::read_dir(feature_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let branch = get_worktree_branch(&path);
                worktrees.push((name, branch));
            }
        }
    }

    worktrees.sort_by(|a, b| a.0.cmp(&b.0));
    worktrees
}

fn check_uncommitted_changes(feature_dir: &PathBuf) -> Vec<String> {
    let mut uncommitted = Vec::new();

    if let Ok(entries) = fs::read_dir(feature_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                if has_uncommitted(&path) {
                    uncommitted.push(name);
                }
            }
        }
    }

    uncommitted
}

fn has_uncommitted(worktree_path: &std::path::Path) -> bool {
    let output = Command::new("git")
        .args([
            "-C",
            worktree_path.to_str().unwrap(),
            "status",
            "--porcelain",
        ])
        .output();

    match output {
        Ok(output) => !String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        Err(_) => false,
    }
}

fn remove_worktrees(feature_dir: &std::path::Path, worktrees: &[(String, String)]) {
    for (repo, _) in worktrees {
        let worktree_path = feature_dir.join(repo);

        let _ = Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .output();

        let _ = fs::remove_dir_all(&worktree_path);
    }
}

fn get_worktree_branch(worktree_path: &std::path::Path) -> String {
    let output = Command::new("git")
        .args([
            "-C",
            worktree_path.to_str().unwrap(),
            "branch",
            "--show-current",
        ])
        .output();

    match output {
        Ok(output) => {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if branch.is_empty() {
                "detached".to_string()
            } else {
                branch
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    struct TestEnv {
        config: Config,
        tmp: PathBuf,
    }

    impl TestEnv {
        fn new() -> Self {
            let unique = format!(
                "xfeat-remove-test-{}-{}",
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

        fn setup_repo(&self, name: &str) -> PathBuf {
            let repo_path = self.config.repos_dir.join(name);
            fs::create_dir_all(&repo_path).unwrap();

            Command::new("git")
                .args(["init", repo_path.to_str().unwrap()])
                .status()
                .expect("failed to init git repo");

            repo_path
        }

        fn create_worktree(&self, feature_name: &str, repo_name: &str, repo_path: &PathBuf) {
            let worktree_path = self.config.features_dir.join(feature_name).join(repo_name);
            fs::create_dir_all(&worktree_path).unwrap();

            Command::new("git")
                .args([
                    "worktree",
                    "add",
                    worktree_path.to_str().unwrap(),
                    "-b",
                    feature_name,
                ])
                .current_dir(repo_path)
                .status()
                .expect("failed to create worktree");
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.tmp);
        }
    }

    #[test]
    fn test_remove_feature_not_found() {
        let env = TestEnv::new();

        let result = run("nonexistent-feature", true, &env.config);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("not found"),
            "error should mention 'not found'"
        );
    }

    #[test]
    fn test_remove_feature_success() {
        let env = TestEnv::new();
        let repo_path = env.setup_repo("service-1");
        env.create_worktree("JIRA-123", "service-1", &repo_path);

        let feature_dir = env.config.features_dir.join("JIRA-123");
        assert!(feature_dir.exists());

        run("JIRA-123", true, &env.config).unwrap();

        assert!(!feature_dir.exists(), "feature directory should be removed");
    }

    #[test]
    fn test_remove_feature_cleans_worktrees() {
        let env = TestEnv::new();
        let repo_path = env.setup_repo("service-1");
        env.create_worktree("JIRA-456", "service-1", &repo_path);

        let feature_dir = env.config.features_dir.join("JIRA-456");
        let worktree_path = feature_dir.join("service-1");

        assert!(worktree_path.exists());

        // Verify worktree is registered before removal
        let worktree_list_before = Command::new("git")
            .args(["-C", repo_path.to_str().unwrap(), "worktree", "list"])
            .output()
            .expect("failed to list worktrees");

        let output_before = String::from_utf8_lossy(&worktree_list_before.stdout);
        assert!(
            output_before.contains(worktree_path.to_str().unwrap()),
            "worktree should be registered before removal"
        );

        run("JIRA-456", true, &env.config).unwrap();

        assert!(!worktree_path.exists(), "worktree should be removed");
        assert!(!feature_dir.exists(), "feature directory should be removed");
    }

    #[test]
    fn test_remove_feature_warns_on_uncommitted() {
        let env = TestEnv::new();
        let repo_path = env.setup_repo("service-1");
        env.create_worktree("JIRA-789", "service-1", &repo_path);

        let worktree_path = env.config.features_dir.join("JIRA-789").join("service-1");

        fs::write(worktree_path.join("uncommitted.txt"), "changes").unwrap();

        let uncommitted = check_uncommitted_changes(&env.config.features_dir.join("JIRA-789"));
        assert!(
            uncommitted.contains(&"service-1".to_string()),
            "should detect uncommitted changes"
        );

        run("JIRA-789", true, &env.config).unwrap();
    }
}
