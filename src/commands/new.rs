use std::fs;

use crate::config::Config;

pub fn run(feature_name: &str, config: &Config) -> anyhow::Result<()> {
    let feature_dir = config.features_dir.join(feature_name);

    if feature_dir.exists() {
        anyhow::bail!(
            "feature directory '{}' already exists",
            feature_dir.display()
        );
    }

    fs::create_dir_all(&feature_dir)?;

    println!(
        "Feature '{feature_name}' created at: {}",
        feature_dir.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.tmp);
        }
    }

    #[test]
    fn test_new_creates_empty_directory() {
        let env = TestEnv::new();

        run("new-test", &env.config).unwrap();

        let feature_dir = env.config.features_dir.join("new-test");
        assert!(feature_dir.is_dir(), "feature directory should exist");
        assert!(
            feature_dir.read_dir().unwrap().next().is_none(),
            "feature directory should be empty"
        );
    }

    #[test]
    fn test_new_fails_if_feature_exists() {
        let env = TestEnv::new();

        let feature_dir = env.config.features_dir.join("existing-test");
        fs::create_dir_all(&feature_dir).unwrap();

        let result = run("existing-test", &env.config);

        assert!(result.is_err(), "expected error for existing feature");
        assert!(
            result.unwrap_err().to_string().contains("already exists"),
            "error message should mention 'already exists'"
        );
    }
}
