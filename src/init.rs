use crate::cli::Shell;
use crate::config::{DEFAULT_FEATURES_DIR, DEFAULT_REPOS_DIR, ENV_FEATURES_DIR, ENV_REPOS_DIR};

#[allow(clippy::literal_string_with_formatting_args)]
fn generate_zsh_init() -> String {
    let code = r#"xf() {
  if [[ $# -eq 0 ]]; then
    xfeat
    return
  fi

  local cmd="$1"
  shift

  case "$cmd" in
    new)
      local feature="$1"
      shift
      xfeat new "$feature" "$@" || return
      local features_dir="${__ENV_FEATURES_DIR__:-__DEFAULT_FEATURES_DIR__}"
      cd "$features_dir/$feature" || return
      ;;
    remove)
      local feature="$1"
      shift
      local current_dir="$(pwd)"
      local features_dir="${__ENV_FEATURES_DIR__:-__DEFAULT_FEATURES_DIR__}"
      local target_dir="$features_dir/$feature"
      xfeat remove "$feature" "$@" || return
      if [ "$current_dir" = "$target_dir" ]; then
        cd "$features_dir"
      fi
      ;;
    *)
      xfeat "$cmd" "$@"
      ;;
  esac
}

_xfeat_complete() {
  local -a commands repos features
  local repos_dir="${__ENV_REPOS_DIR__:-__DEFAULT_REPOS_DIR__}"
  local features_dir="${__ENV_FEATURES_DIR__:-__DEFAULT_FEATURES_DIR__}"

  commands=("new:create a new feature" "list:list all features" "remove:remove a feature")

  if (( CURRENT == 2 )); then
    _describe 'command' commands
  elif (( CURRENT > 2 )); then
    case "$words[2]" in
      new)
        repos_dir="${(e)repos_dir}"
        repos_dir="${~repos_dir}"
        if [[ -d "$repos_dir" ]]; then
          repos=("${(@f)$(command ls -1 "$repos_dir" 2>/dev/null)}")
          if (( ${#repos} > 0 )); then
            _describe 'repository' repos
          fi
        fi
        ;;
      remove)
        features_dir="${(e)features_dir}"
        features_dir="${~features_dir}"
        if [[ -d "$features_dir" ]]; then
          features=("${(@f)$(command ls -1 "$features_dir" 2>/dev/null)}")
          if (( ${#features} > 0 )); then
            _describe 'feature' features
          fi
        fi
        ;;
    esac
  fi
}

compdef _xfeat_complete xf
"#;

    code.replace("__ENV_REPOS_DIR__", ENV_REPOS_DIR)
        .replace("__ENV_FEATURES_DIR__", ENV_FEATURES_DIR)
        .replace("__DEFAULT_REPOS_DIR__", DEFAULT_REPOS_DIR)
        .replace("__DEFAULT_FEATURES_DIR__", DEFAULT_FEATURES_DIR)
}

/// Generates shell initialization code for the specified shell.
///
/// The output should be evaluated in the user's shell configuration
/// (e.g., `eval "$(xfeat init zsh)"` in `~/.zshrc`).
pub fn run(shell: &Shell) {
    match shell {
        Shell::Zsh => print!("{}", generate_zsh_init()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zsh_init() -> String {
        generate_zsh_init()
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
    fn test_init_zsh_uses_env_constants() {
        let output = zsh_init();
        assert!(
            output.contains(ENV_REPOS_DIR),
            "zsh init code should use ENV_REPOS_DIR constant"
        );
        assert!(
            output.contains(ENV_FEATURES_DIR),
            "zsh init code should use ENV_FEATURES_DIR constant"
        );
    }

    #[test]
    fn test_init_zsh_uses_default_constants() {
        let output = zsh_init();
        assert!(
            output.contains(DEFAULT_REPOS_DIR),
            "zsh init code should use DEFAULT_REPOS_DIR constant"
        );
        assert!(
            output.contains(DEFAULT_FEATURES_DIR),
            "zsh init code should use DEFAULT_FEATURES_DIR constant"
        );
    }
}
