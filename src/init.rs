use include_dir::{Dir, include_dir};

use crate::cli::Shell;

static SHELL_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/shell");

fn get_shell_init(shell: &Shell) -> &'static str {
    let filename = match shell {
        Shell::Zsh => "init.zsh",
    };

    SHELL_DIR
        .get_file(filename)
        .unwrap_or_else(|| panic!("shell init file not found: {filename}"))
        .contents_utf8()
        .unwrap_or_else(|| panic!("shell init file is not valid UTF-8: {filename}"))
}

/// Generates shell initialization code for the specified shell.
///
/// The output should be evaluated in the user's shell configuration
/// (e.g., `eval "$(xfeat init zsh)"` in `~/.zshrc`).
pub fn run(shell: &Shell) {
    print!("{}", get_shell_init(shell));
}

#[cfg(test)]
mod tests {
    use super::get_shell_init;
    use crate::cli::Shell;

    fn zsh_init() -> &'static str {
        get_shell_init(&Shell::Zsh)
    }

    #[test]
    fn test_init_zsh_outputs_code() {
        let output = zsh_init();
        assert!(!output.is_empty(), "zsh init code should not be empty");
    }

    #[test]
    fn test_init_zsh_contains_xf_function() {
        assert!(
            zsh_init().contains("xf()"),
            "zsh init code should define xf function"
        );
    }

    #[test]
    fn test_init_zsh_contains_completion() {
        assert!(
            zsh_init().contains("compdef _xfeat_complete xf"),
            "zsh init code should register completion"
        );
    }

    #[test]
    fn test_init_zsh_handles_new_command() {
        assert!(
            zsh_init().contains("xfeat new"),
            "zsh init code should handle new command"
        );
    }

    #[test]
    fn test_init_zsh_handles_remove_command() {
        assert!(
            zsh_init().contains("xfeat remove"),
            "zsh init code should handle remove command"
        );
    }

    #[test]
    fn test_init_zsh_remove_does_not_force_yes() {
        assert!(
            !zsh_init().contains("--yes"),
            "zsh init code should not pass --yes flag for remove (user should confirm)"
        );
    }

    #[test]
    fn test_init_zsh_uses_env_variables() {
        let output = zsh_init();
        assert!(
            output.contains("XF_REPOS_DIR"),
            "zsh init code should use XF_REPOS_DIR variable"
        );
        assert!(
            output.contains("XF_FEATURES_DIR"),
            "zsh init code should use XF_FEATURES_DIR variable"
        );
    }
}
