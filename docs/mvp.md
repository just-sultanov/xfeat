# MVP Plan: xfeat

CLI utility for managing git worktrees across multiple repositories.

## Overview

xfeat is designed for developers working on multiple products simultaneously. Each product has its own workspace with `repos` (source repositories) and `features` (worktree branches) directories. Environment variables `XF_REPOS_DIR` and `XF_FEATURES_DIR` are scoped per-project, allowing isolated feature development across multiple repositories within a single product context.

By leveraging git worktrees, xfeat enables parallel development on multiple features without the overhead of cloning repositories or switching branches. Each feature gets its own isolated workspace, making it ideal for AI-assisted development where multiple coding agents can work on different features simultaneously without conflicts.

Example project structure:

```
~/projects/project-x/
├── repos/        # Source git repositories
└── features/     # Git worktrees for active features
```

## Implemented Commands

### `xfeat new <feature-name>`

Creates an empty feature directory. Use `xfeat add` to add worktrees.

### `xfeat add <feature-name> <repos...>`

Adds worktrees for specified repositories to an existing feature. Supports `--from <branch>` to specify the base branch and `--branch <name>` for a custom branch name. Skips repos that already have worktrees in the feature.

### `xfeat list`

Lists all features with their worktrees and current branch names in a tree-like format. Branch names are always shown. Empty features (no worktrees) are shown with an `(empty)` marker. Supports `--path` flag to also show worktree paths.

### `xfeat remove <feature-name>`

Removes a feature directory and its worktrees. Prompts for confirmation by default. Warns about uncommitted changes. Supports `--yes` / `-y` flag to skip confirmation.

### `xfeat sync <feature-name>`

Syncs a feature with the latest main branch from source repos. For each worktree:
1. Fetches latest changes from remote
2. Rebases the feature branch onto `origin/main` (auto-detected via `refs/remotes/origin/HEAD`, fallback `main`)
3. Stops on first conflict with an error message

### `xfeat init <shell>`

Generates shell initialization code for `eval`. Currently supports `zsh`.

The generated code provides:
- `xf` wrapper function that proxies to `xfeat`
- Tab completion for repository names (`xf new <TAB>`, `xf add <TAB>`) with automatic filtering of already-specified repos
- Tab completion for feature names (`xf add <TAB>` first arg, `xf remove <TAB>`, `xf sync <TAB>`)
- Autocomplete for `xf add` shows features for the first arg, repos (excluding already-added ones) for subsequent args
- Reads `XF_REPOS_DIR` / `XF_FEATURES_DIR` directly from the environment on each invocation (compatible with `direnv`)
- Automatically expands `~` in path variables

## Project Structure

```
src/
├── main.rs           # Entry point, arg parsing, command dispatch
├── cli.rs            # CLI definition (clap)
├── config.rs         # Configuration (env vars with defaults)
├── error.rs          # Custom error types
├── init.rs           # Shell initialization code (embeds shell/ scripts)
├── worktree.rs       # Git worktree operations
└── commands/
    ├── mod.rs
    ├── new.rs        # Implementation of `new` command
    ├── add.rs        # Implementation of `add` command
    ├── list.rs       # Implementation of `list` command
    ├── remove.rs     # Implementation of `remove` command
    └── sync.rs       # Implementation of `sync` command
shell/
└── init.zsh          # Zsh initialization with completions (embedded at compile time)
bin/
└── ...               # Mise task scripts (clean, lint, test, build-*, dist-*, etc.)
```

## Dependencies

| Crate           | Purpose                          |
| --------------- | -------------------------------- |
| `clap` (derive) | CLI argument parsing             |
| `thiserror`     | Custom error types               |
| `anyhow`        | Application-level error handling |
| `shellexpand`   | Expand `~` and env vars in paths |
| `include_dir`   | Embed shell/ directory at compile time |

## Configuration

- `Config` struct with `repos_dir` and `features_dir` fields
- Read from env vars `XF_REPOS_DIR` / `XF_FEATURES_DIR`
- Defaults: `~/workspace/repos`, `~/workspace/features`
- Supports absolute (`/tmp/repos`), relative (`./repos`), and tilde (`~/repos`) paths
- Expand `~` via `shellexpand`
- Env var names defined as constants in `config.rs`

## Error Types

- `Error::RepoNotFound` — repository not found in repos_dir
- `Error::WorktreeExists` — worktree already exists
- `Error::GitCommand` — git command execution failed
- `Error::RebaseConflict` — rebase conflict during sync
- `Error::Io` — filesystem error

## Git Worktree Logic

- `fn create_worktree(source_repo: &Path, worktree_path: &Path, branch: &str) -> Result<()>`
- Execute `git worktree add <path> -b <branch>` via `std::process::Command`
- Paths are resolved to absolute before calling git (fixes relative path issues)
- Validate that source is a git repository

## Testing

- Tests use `TestEnv` fixture struct with `Drop` for automatic cleanup
- Tests verify: directory creation, worktree links, branch names, error cases, skip-existing behavior

## Implemented Features

### Shell Wrapper `xf`

- Zsh script stored in `shell/init.zsh`, embedded into the binary at compile time:
  - `xf new <feature>` — creates an empty feature directory
  - `xf add <feature> <repos...>` — adds worktrees to a feature (supports `--from` and `--branch`)
  - `xf remove` — `cd` out if currently in the feature directory (with confirmation prompt)
  - `xf sync` — syncs feature with latest main
  - Other commands — proxy to `xfeat`
  - Autocompletion for repository names (`xf new <TAB>`, `xf add <TAB>`) with automatic filtering of already-specified repos
  - Autocomplete for `xf add` shows features for the first arg, repos (excluding already-added ones) for subsequent args
  - Autocompletion for feature names (`xf remove <TAB>`, `xf sync <TAB>`)
  - Reads `XF_REPOS_DIR` / `XF_FEATURES_DIR` directly from the environment on each invocation (compatible with `direnv`)
  - Automatically expands `~` in path variables

### Version Display

- `--version` flag shows version from `Cargo.toml` (e.g., `0.1.0`)
- `--help` shows full version string with git SHA and build date (e.g., `v0.1.0@abc1234 (2026-04-03)`)
- Version information generated at compile time via `build.rs`

### Release Process

- GitHub Actions workflows for CI and automated releases
- CI runs on push to `main`: lint → test → build for all platforms
- Release triggers on `v*` tags: lint → test → build → draft release
- Linux binary is statically linked (musl) for maximum compatibility
- macOS binaries built for both aarch64 and x86_64 architectures
- Draft releases allow review before publishing

### Development Tooling

- mise tasks with scripts in `bin/` directory for clean separation of concerns
- Tasks for version management (`bump-version`, `pre-publish`, `publish`)
- Cross-platform build and packaging tasks (`dist-linux`, `dist-macos-arm`, `dist-macos-x86`, `dist-windows.ps1`)
- Installable via `mise install github:just-sultanov/xfeat`

### Future Commands

- `xfeat status` — show git status across all worktrees in a feature
