use std::path::PathBuf;

pub const ENV_REPOS_DIR: &str = "XF_REPOS_DIR";
pub const ENV_FEATURES_DIR: &str = "XF_FEATURES_DIR";
pub const DEFAULT_REPOS_DIR: &str = "~/workspace/repos";
pub const DEFAULT_FEATURES_DIR: &str = "~/workspace/features";

pub struct Config {
    pub repos_dir: PathBuf,
    pub features_dir: PathBuf,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let repos_dir =
            shellexpand::env(&var_or_default(ENV_REPOS_DIR, DEFAULT_REPOS_DIR))?.into_owned();
        let repos_dir = shellexpand::tilde(&repos_dir).into_owned();

        let features_dir =
            shellexpand::env(&var_or_default(ENV_FEATURES_DIR, DEFAULT_FEATURES_DIR))?.into_owned();
        let features_dir = shellexpand::tilde(&features_dir).into_owned();

        let repos_dir = make_absolute(PathBuf::from(repos_dir))?;
        let features_dir = make_absolute(PathBuf::from(features_dir))?;

        Ok(Self {
            repos_dir,
            features_dir,
        })
    }
}

fn make_absolute(path: PathBuf) -> anyhow::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path);
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(&path))
        .map_err(|e| anyhow::anyhow!("cannot resolve path '{}': {}", path.display(), e))
}

fn var_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_with_relative_paths() {
        unsafe {
            std::env::set_var(ENV_REPOS_DIR, "./test-repos");
            std::env::set_var(ENV_FEATURES_DIR, "./test-features");
        }

        let config = Config::load().unwrap();
        let cwd = std::env::current_dir().unwrap();

        assert_eq!(config.repos_dir, cwd.join("test-repos"));
        assert_eq!(config.features_dir, cwd.join("test-features"));

        unsafe {
            std::env::remove_var(ENV_REPOS_DIR);
            std::env::remove_var(ENV_FEATURES_DIR);
        }
    }

    #[test]
    fn test_config_load_with_absolute_paths() {
        unsafe {
            std::env::set_var(ENV_REPOS_DIR, "/tmp/test-repos");
            std::env::set_var(ENV_FEATURES_DIR, "/tmp/test-features");
        }

        let config = Config::load().unwrap();

        assert_eq!(config.repos_dir, PathBuf::from("/tmp/test-repos"));
        assert_eq!(config.features_dir, PathBuf::from("/tmp/test-features"));

        unsafe {
            std::env::remove_var(ENV_REPOS_DIR);
            std::env::remove_var(ENV_FEATURES_DIR);
        }
    }

    #[test]
    fn test_config_load_with_tilde_paths() {
        unsafe {
            std::env::set_var(ENV_REPOS_DIR, "~/my-repos");
            std::env::set_var(ENV_FEATURES_DIR, "~/my-features");
        }

        let config = Config::load().unwrap();
        let home = shellexpand::tilde("~").into_owned();

        assert_eq!(config.repos_dir, PathBuf::from(&home).join("my-repos"));
        assert_eq!(
            config.features_dir,
            PathBuf::from(&home).join("my-features")
        );

        unsafe {
            std::env::remove_var(ENV_REPOS_DIR);
            std::env::remove_var(ENV_FEATURES_DIR);
        }
    }

    #[test]
    fn test_config_load_defaults() {
        unsafe {
            std::env::remove_var(ENV_REPOS_DIR);
            std::env::remove_var(ENV_FEATURES_DIR);
        }

        let config = Config::load().unwrap();
        let home = shellexpand::tilde("~").into_owned();

        assert_eq!(
            config.repos_dir,
            PathBuf::from(&home).join("workspace/repos")
        );
        assert_eq!(
            config.features_dir,
            PathBuf::from(&home).join("workspace/features")
        );
    }
}
