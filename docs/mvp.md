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

### `xfeat new <feature-name> <repos...>`

Creates a feature directory with git worktrees for each specified repository.

### `xfeat list`

Lists all features with their worktrees and current branch names in a tree-like format.

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
- `xf switch <feature>` — `cd` into a feature directory
- Tab completion for repository names (`xf new <TAB>`) and feature names (`xf remove <TAB>`, `xf sync <TAB>`, `xf switch <TAB>`)
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
    ├── list.rs       # Implementation of `list` command
    ├── remove.rs     # Implementation of `remove` command
    └── sync.rs       # Implementation of `sync` command
shell/
└── init.zsh          # Zsh initialization with completions (embedded at compile time)
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

## Rollback Strategy

If worktree creation fails for any repository — remove all already-created worktrees and the feature directory to avoid leaving a partial state.

## Testing

- Tests use `TestEnv` fixture struct with `Drop` for automatic cleanup
- Tests verify: directory creation, worktree links, branch names, error cases, rollback

## Implemented Features

### Shell Wrapper `xf`

- Zsh script stored in `shell/init.zsh`, embedded into the binary at compile time:
  - `xf new` — calls `xfeat new` to create a feature
  - `xf switch <feature>` — `cd` into the feature directory (errors if not found)
  - `xf remove` — `cd` out if currently in the feature directory (with confirmation prompt)
  - `xf sync` — syncs feature with latest main
  - Other commands — proxy to `xfeat`
  - Autocompletion for repository names (`xf new <TAB>`), feature names (`xf remove <TAB>`, `xf sync <TAB>`, `xf switch <TAB>`)
  - Reads `XF_REPOS_DIR` / `XF_FEATURES_DIR` directly from the environment on each invocation (compatible with `direnv`)
  - Automatically expands `~` in path variables

### Future Commands

- `xfeat status` — show git status across all worktrees in a feature
