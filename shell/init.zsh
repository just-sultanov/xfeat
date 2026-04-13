xf() {
  if [[ $# -eq 0 ]]; then
    xfeat
    return
  fi

  local cmd="$1"
  shift

  case "$cmd" in
    new)
      xfeat new "$@"
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
    add)
      local feature="$1"
      shift
      xfeat add "$feature" "$@"
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

  commands=("new" "list" "remove" "sync" "add")

  if [ $CURRENT -eq 2 ]; then
    _describe 'command' commands
  elif [ $CURRENT -gt 2 ]; then
    local cmd="${words[2]}"
    case "$cmd" in
      remove|sync)
        if [[ -d "$features_dir" ]]; then
          features=("${(@f)$(ls -1d "$features_dir"/*/ 2>/dev/null | xargs -n1 basename)}")
          if (( ${#features} > 0 )); then
            _describe 'feature' features
          fi
        fi
        ;;
      add)
        if [ $CURRENT -eq 3 ]; then
          if [[ -d "$features_dir" ]]; then
            features=("${(@f)$(ls -1d "$features_dir"/*/ 2>/dev/null | xargs -n1 basename)}")
            if (( ${#features} > 0 )); then
              _describe 'feature' features
            fi
          fi
        else
          if [[ -d "$repos_dir" ]]; then
            repos=("${(@f)$(find "$repos_dir" -name .git -exec dirname {} \; 2>/dev/null | sed "s|$repos_dir/||")}")
            local feature="${words[3]}"
            local feature_path="${features_dir%/}/$feature"
            if [[ -d "$feature_path" ]]; then
              local used
              used=$(find "$feature_path" -name .git -exec dirname {} \; 2>/dev/null | sed "s|$feature_path/||")
              repos=("${(@)repos:|used}")
            fi
            if (( ${#repos} > 0 )); then
              _describe 'repository' repos
            fi
          fi
        fi
        ;;
    esac
  fi
}

compdef _xfeat_complete xf