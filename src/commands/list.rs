use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Config;

pub fn run(config: &Config, show_path: bool) -> anyhow::Result<()> {
    if !config.features_dir.exists() {
        println!("No features found");
        return Ok(());
    }

    let mut feature_map: HashMap<String, Vec<WorktreeEntry>> = HashMap::new();

    for entry in fs::read_dir(&config.features_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let feature_name = path.file_name().unwrap().to_string_lossy().to_string();
        let worktrees = collect_worktrees_recursive(&path, &feature_name);
        feature_map.insert(feature_name, worktrees);
    }

    let mut features: Vec<_> = feature_map.into_iter().collect();
    features.sort_by(|a, b| a.0.cmp(&b.0));

    if features.is_empty() {
        println!("No features found");
        return Ok(());
    }

    print_features(&features, config, show_path);

    Ok(())
}

#[derive(Debug, Clone)]
struct WorktreeEntry {
    #[allow(dead_code)]
    path: PathBuf,
    rel_path: String,
    branch: String,
}

fn collect_worktrees_recursive(feature_dir: &Path, _prefix: &str) -> Vec<WorktreeEntry> {
    let mut worktrees = Vec::new();
    scan_recursive(feature_dir, feature_dir, &mut worktrees);
    worktrees
}

fn scan_recursive(dir: &Path, base: &Path, worktrees: &mut Vec<WorktreeEntry>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.join(".git").exists() {
                    let rel_path = path.strip_prefix(base).unwrap_or(&path);
                    let rel_str = rel_path.to_string_lossy().to_string();
                    let branch = get_worktree_branch(&path);
                    worktrees.push(WorktreeEntry {
                        path,
                        rel_path: rel_str,
                        branch,
                    });
                } else {
                    scan_recursive(&path, base, worktrees);
                }
            }
        }
    }
}

fn get_worktree_branch(worktree_path: &Path) -> String {
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

fn print_features(features: &[(String, Vec<WorktreeEntry>)], _config: &Config, show_path: bool) {
    for (feature_name, worktrees) in features {
        if worktrees.is_empty() {
            println!("{feature_name} (empty)");
        } else {
            println!("{feature_name}");
            for wt in worktrees {
                let branch = &wt.branch;
                if show_path {
                    println!(
                        "  {} ({branch}) -> {}/{}",
                        wt.rel_path, feature_name, wt.rel_path
                    );
                } else {
                    println!("  {} ({branch})", wt.rel_path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
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
                "xfeat-list-test-{}-{}",
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
    fn test_list_features_empty() {
        let env = TestEnv::new();
        run(&env.config, false).unwrap();
    }

    #[test]
    fn test_list_features_single() {
        let env = TestEnv::new();
        let repo_path = env.setup_repo("service-1");
        env.create_worktree("JIRA-123", "service-1", &repo_path);

        run(&env.config, false).unwrap();

        let feature_dir = env.config.features_dir.join("JIRA-123");
        assert!(feature_dir.exists());
        assert!(feature_dir.join("service-1").exists());

        let branch_output = Command::new("git")
            .args([
                "-C",
                feature_dir.join("service-1").to_str().unwrap(),
                "branch",
                "--show-current",
            ])
            .output()
            .expect("failed to get branch");

        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();
        assert_eq!(branch, "JIRA-123");
    }

    #[test]
    fn test_list_features_multiple() {
        let env = TestEnv::new();
        let repo1 = env.setup_repo("service-1");
        let repo2 = env.setup_repo("service-2");

        env.create_worktree("JIRA-123", "service-1", &repo1);
        env.create_worktree("JIRA-123", "service-2", &repo2);
        env.create_worktree("JIRA-456", "service-1", &repo1);

        run(&env.config, false).unwrap();

        let feature_123 = env.config.features_dir.join("JIRA-123");
        let feature_456 = env.config.features_dir.join("JIRA-456");

        assert!(feature_123.exists());
        assert!(feature_123.join("service-1").exists());
        assert!(feature_123.join("service-2").exists());
        assert!(feature_456.exists());
        assert!(feature_456.join("service-1").exists());
    }

    #[test]
    fn test_list_features_sorted() {
        let env = TestEnv::new();
        let repo = env.setup_repo("lib-1");

        env.create_worktree("zzz-feature", "lib-1", &repo);
        env.create_worktree("aaa-feature", "lib-1", &repo);
        env.create_worktree("mmm-feature", "lib-1", &repo);

        run(&env.config, false).unwrap();

        let entries: Vec<_> = fs::read_dir(&env.config.features_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();

        assert!(entries.contains(&"aaa-feature".to_string()));
        assert!(entries.contains(&"mmm-feature".to_string()));
        assert!(entries.contains(&"zzz-feature".to_string()));
    }

    #[test]
    fn test_list_features_dir_does_not_exist() {
        let unique = format!(
            "xfeat-list-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let tmp = std::env::temp_dir().join(unique);
        let config = Config {
            repos_dir: tmp.join("repos"),
            features_dir: tmp.join("nonexistent-features"),
        };

        run(&config, false).unwrap();
    }

    #[test]
    fn test_list_features_empty_feature_dir() {
        let env = TestEnv::new();

        let empty_feature_dir = env.config.features_dir.join("empty-feature");
        fs::create_dir_all(&empty_feature_dir).unwrap();

        run(&env.config, false).unwrap();

        let feature_dir = env.config.features_dir.join("empty-feature");
        assert!(feature_dir.exists());
        assert!(feature_dir.read_dir().unwrap().next().is_none());
    }

    #[test]
    fn test_list_features_mixed_empty_and_with_worktrees() {
        let env = TestEnv::new();
        let repo = env.setup_repo("service-1");

        env.create_worktree("JIRA-123", "service-1", &repo);
        fs::create_dir_all(env.config.features_dir.join("empty-feature")).unwrap();
        env.create_worktree("JIRA-456", "service-1", &repo);

        run(&env.config, false).unwrap();

        assert!(env.config.features_dir.join("JIRA-123").exists());
        assert!(env.config.features_dir.join("empty-feature").exists());
        assert!(env.config.features_dir.join("JIRA-456").exists());
    }

    #[test]
    fn test_list_features_ignores_non_git_dirs() {
        let env = TestEnv::new();

        let not_a_worktree = env.config.features_dir.join("not-a-feature");
        fs::create_dir_all(&not_a_worktree).unwrap();
        fs::create_dir_all(not_a_worktree.join("some-file.txt")).unwrap();

        run(&env.config, false).unwrap();
    }

    #[test]
    fn test_list_features_many_worktrees() {
        let env = TestEnv::new();
        let repo = env.setup_repo("lib-1");

        env.create_worktree("big-feature", "service-1", &repo);
        env.create_worktree("big-feature", "service-2", &repo);
        env.create_worktree("big-feature", "service-3", &repo);
        env.create_worktree("big-feature", "service-4", &repo);
        env.create_worktree("big-feature", "lib-1", &repo);

        run(&env.config, false).unwrap();

        let feature_dir = env.config.features_dir.join("big-feature");
        assert!(feature_dir.exists());
        assert!(feature_dir.join("service-1").exists());
        assert!(feature_dir.join("service-2").exists());
        assert!(feature_dir.join("service-3").exists());
        assert!(feature_dir.join("service-4").exists());
        assert!(feature_dir.join("lib-1").exists());

        for repo_name in ["service-1", "service-2", "service-3", "service-4", "lib-1"] {
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
            assert_eq!(branch, "big-feature");
        }
    }

    #[test]
    fn test_list_features_special_characters() {
        let env = TestEnv::new();
        let repo = env.setup_repo("lib-1");

        env.create_worktree("feature/with-slashes", "lib-1", &repo);

        run(&env.config, false).unwrap();
    }

    #[test]
    fn test_list_features_files_in_features_dir_ignored() {
        let env = TestEnv::new();
        let repo = env.setup_repo("lib-1");

        fs::write(env.config.features_dir.join("some-file.txt"), "not a dir").unwrap();

        env.create_worktree("valid-feature", "lib-1", &repo);

        run(&env.config, false).unwrap();

        let feature_dir = env.config.features_dir.join("valid-feature");
        assert!(feature_dir.exists());
    }

    #[test]
    fn test_list_features_worktree_has_correct_branch() {
        let env = TestEnv::new();
        let repo_path = env.setup_repo("service-1");
        env.create_worktree("JIRA-123", "service-1", &repo_path);

        let worktree_path = env.config.features_dir.join("JIRA-123").join("service-1");
        assert!(worktree_path.exists());
        assert!(worktree_path.join(".git").exists());

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
        assert_eq!(branch, "JIRA-123");
    }

    #[test]
    fn test_list_features_worktree_is_linked_to_source() {
        let env = TestEnv::new();
        let repo_path = env.setup_repo("service-1");
        env.create_worktree("JIRA-456", "service-1", &repo_path);

        let worktree_path = env.config.features_dir.join("JIRA-456").join("service-1");

        let worktree_list = Command::new("git")
            .args(["-C", repo_path.to_str().unwrap(), "worktree", "list"])
            .output()
            .expect("failed to list worktrees");

        let output = String::from_utf8_lossy(&worktree_list.stdout);
        assert!(
            output.contains(worktree_path.to_str().unwrap()),
            "worktree should be linked to source repo"
        );
    }

    #[test]
    fn test_list_features_with_path_flag() {
        let env = TestEnv::new();
        let repo_path = env.setup_repo("service-1");
        env.create_worktree("JIRA-123", "service-1", &repo_path);

        run(&env.config, true).unwrap();
    }

    #[test]
    fn test_list_nested_worktrees() {
        let env = TestEnv::new();
        let repo1 = env.setup_repo("payment-service");
        let repo2 = env.setup_repo("checkout-service");

        env.create_worktree("story-123", "services/payment-service", &repo1);
        env.create_worktree("story-123", "services/checkout-service", &repo2);

        run(&env.config, false).unwrap();

        let feature_dir = env.config.features_dir.join("story-123");
        assert!(feature_dir.join("services/payment-service").exists());
        assert!(feature_dir.join("services/checkout-service").exists());
    }
}
