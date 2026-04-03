# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - YYYY-MM-DD

### Initial Release

Initial release of `xfeat` CLI for managing git worktrees across multiple repositories

## Features

- `xfeat new` — create a feature with worktrees for specified repositories
- `xfeat list` — list all features with their worktrees and branches
- `xfeat remove` — remove a feature with confirmation and uncommitted change warnings
- `xfeat sync` — sync a feature with the latest main branch (fetch + rebase)
- `xfeat init` — generate shell initialization code with autocompletion
- `xf` shell wrapper (currently `zsh` only)
