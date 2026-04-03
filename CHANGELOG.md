# Changelog

All notable changes to this project will be documented in this file.

## 0.1.3 - 2026-04-03

### Added

- `xf add` — add worktrees for repos to an existing feature
- `xf add --from <branch>` — create worktree branch from a specific branch
- `xf add --branch <name>` — use a custom branch name instead of the feature name
- Empty features are now shown in `xf list` with `(empty)` marker

### Changed

- `xf new` now only creates an empty feature directory (use `xf add` to add worktrees)
- `xf new` autocomplete now shows feature names instead of repos
- `xf add` autocomplete excludes repos already added to the feature
- `xf add` autocomplete shows features for the first arg, repos for subsequent args

## 0.1.2 - 2026-04-03

### Changed

- Linux binary is now statically linked (musl), compatible with all Linux distributions

## 0.1.1 - 2026-04-03

### Initial Release

Initial release of `xfeat` CLI for managing git worktrees across multiple repositories

## Features

- `xfeat new` — create a feature with worktrees for specified repositories
- `xfeat list` — list all features with their worktrees and branches
- `xfeat remove` — remove a feature with confirmation and uncommitted change warnings
- `xfeat sync` — sync a feature with the latest main branch (fetch + rebase)
- `xfeat init` — generate shell initialization code with autocompletion
- `xf` shell wrapper (currently `zsh` only)
