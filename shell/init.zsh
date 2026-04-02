xf() {
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
      xfeat new "$feature" "$@"
      ;;
    remove)
      local feature="$1"
      shift
      local current_dir="$(pwd)"
      local features_dir="${XF_FEATURES_DIR/#\~/$HOME}"
      local target_dir="${features_dir%/}/$feature"
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
  local repos_dir="${XF_REPOS_DIR/#\~/$HOME}"
  local features_dir="${XF_FEATURES_DIR/#\~/$HOME}"

  commands=("new:create a new feature" "list:list all features" "remove:remove a feature" "sync:sync a feature with main")

  if [ $CURRENT -eq 2 ]; then
    _describe 'command' commands
  elif [ $CURRENT -gt 2 ]; then
    local cmd="${words[2]}"
    case "$cmd" in
      new)
        if [[ -d "$repos_dir" ]]; then
          repos=("${(@f)$(command ls -1 "$repos_dir" 2>/dev/null)}")
          if (( ${#repos} > 0 )); then
            _describe 'repository' repos
          fi
        fi
        ;;
      remove|sync)
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
