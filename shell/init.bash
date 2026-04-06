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
    local current_dir
    current_dir="$(pwd)"
    local features_dir="${XF_FEATURES_DIR/#\~/$HOME}"
    local target_dir="${features_dir%/}/$feature"
    xfeat remove "$feature" "$@" || return
    if [ "$current_dir" = "$target_dir" ]; then
      cd "$features_dir" || return
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
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local repos_dir="${XF_REPOS_DIR/#\~/$HOME}"
  local features_dir="${XF_FEATURES_DIR/#\~/$HOME}"

  local commands="new list remove sync add"

  if [[ $COMP_CWORD -eq 1 ]]; then
    mapfile -t COMPREPLY < <(compgen -W "$commands" -- "$cur")
    return
  fi

  local cmd="${COMP_WORDS[1]}"

  case "$cmd" in
  remove | sync)
    if [[ -d "$features_dir" ]]; then
      local features
      features=$(command ls -1 "$features_dir" 2>/dev/null)
      mapfile -t COMPREPLY < <(compgen -W "$features" -- "$cur")
    fi
    ;;
  add)
    if [[ $COMP_CWORD -eq 2 ]]; then
      if [[ -d "$features_dir" ]]; then
        local features
        features=$(command ls -1 "$features_dir" 2>/dev/null)
        mapfile -t COMPREPLY < <(compgen -W "$features" -- "$cur")
      fi
    else
      if [[ -d "$repos_dir" ]]; then
        local repos
        repos=$(command ls -1 "$repos_dir" 2>/dev/null)
        local feature="${COMP_WORDS[2]}"
        local feature_path="${features_dir%/}/$feature"
        if [[ -d "$feature_path" ]]; then
          local used
          used=$(command ls -1 "$feature_path" 2>/dev/null)
          for u in $used; do
            repos="${repos/$u/}"
          done
        fi
        mapfile -t COMPREPLY < <(compgen -W "$repos" -- "$cur")
      fi
    fi
    ;;
  *)
    COMPREPLY=()
    ;;
  esac
}

complete -F _xfeat_complete xf
