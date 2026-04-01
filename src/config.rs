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

        Ok(Self {
            repos_dir: PathBuf::from(repos_dir),
            features_dir: PathBuf::from(features_dir),
        })
    }
}

fn var_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
