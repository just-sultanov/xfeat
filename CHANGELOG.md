# Changelog

All notable changes to this project will be documented in this file.

## 0.3.1 - 2026-04-07

### Added

- `xfeat init bash` — bash shell support with `xf` wrapper function and tab completions
- `shell/init.bash` — bash initialization script with completion for all commands

### Changed

- Updated documentation (README.md, AGENTS.md, SKILL.md) to reflect bash shell support
- `xf` shell wrapper now available for both `zsh` and `bash`

## 0.3.0 - 2026-04-07

### Added

- `xf list --path` — show worktree paths alongside branch information
- `xf add --from <branch> --branch <name>` — combine source branch with custom branch name
- Alpha status warning in README

### Changed

- `xf list` now always shows branch information in expanded tree format (no longer compact `repo (branch)` format)
- Removed `xf switch` command — users should use `cd "$XF_FEATURES_DIR/<feature>"` instead
- Repository naming convention in documentation updated to story-based examples (e.g., `STORY-123-add-payment`)
- Repository names in examples updated to realistic names (`payment-service`, `checkout-service`, `frontend`)

### Fixed

- Stale worktrees are now automatically pruned before creating new ones, preventing "missing but already registered worktree" errors
- Tree structure lines in `xf list` output are now properly aligned for nested features

### Removed

- `xf switch` command and its autocomplete entry (use `cd` directly)

## 0.2.0 - 2026-04-03

### Added

- Windows build support — works in both Command Prompt and PowerShell

## 0.1.3 - 2026-04-03

### Added

- `xf add` — add worktrees for repos to an existing feature
- `xf add --from <branch>` — create worktree branch from a specific branch
- `xf add --branch <name>` — use a custom branch name instead of the feature name
- Empty features are now shown in `xf list` with `(empty)` marker

### Changed

- `xf new` now only creates an empty feature directory (use `xf add` to add worktrees)
- `xf new` autocomplete excludes repos already specified in the command
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
