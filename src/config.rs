use std::path::PathBuf;

pub struct Config {
    pub repos_dir: PathBuf,
    pub features_dir: PathBuf,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let repos_dir =
            shellexpand::env(&var_or_default("XF_REPOS_DIR", "~/workspace/repos"))?.into_owned();

        let features_dir =
            shellexpand::env(&var_or_default("XF_FEATURES_DIR", "~/workspace/features"))?
                .into_owned();

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
