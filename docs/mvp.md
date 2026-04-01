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

## Project Structure

```
src/
├── main.rs           # Entry point, arg parsing, command dispatch
├── cli.rs            # CLI definition (clap)
├── config.rs         # Configuration (env vars with defaults)
├── error.rs          # Custom error types
├── worktree.rs       # Git worktree operations
└── commands/
    ├── mod.rs
    ├── new.rs        # Implementation of `new` command
    └── list.rs       # Implementation of `list` command
```

## Dependencies

| Crate           | Purpose                          |
| --------------- | -------------------------------- |
| `clap` (derive) | CLI argument parsing             |
| `thiserror`     | Custom error types               |
| `anyhow`        | Application-level error handling |
| `shellexpand`   | Expand `~` and env vars in paths |

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
- `Error::Io` — filesystem error

## Git Worktree Logic

- `fn create_worktree(source_repo: &Path, worktree_path: &Path, branch: &str) -> Result<()>`
- Execute `git worktree add <path> -b <branch>` via `std::process::Command`
- Paths are resolved to absolute before calling git (fixes relative path issues)
- Validate that source is a git repository

## Rollback Strategy

If worktree creation fails for any repository — remove all already-created worktrees and the feature directory to avoid leaving a partial state.

## Testing

- 24 tests total (8 for `new`, 12 for `list`, 4 for `config`)
- `TestEnv` fixture struct with `Drop` for automatic cleanup
- Tests verify: directory creation, worktree links, branch names, error cases, rollback

## Planned Features

### Shell Wrapper `xf`

- Bash/zsh/fish script that:
  - Calls `xfeat new "$@"` to create
  - On success — `cd` into feature directory
  - On `delete` — `cd` out if currently in the feature directory
  - For other commands — proxy to `xfeat`
- Autocompletion for repository names (directory listing from `XF_REPOS_DIR`)
- Implemented via `xfeat init <shell>` command that outputs shell code for `eval`

### Future Commands

- `xfeat delete <feature-name>` — remove feature worktrees
- `xfeat status` — show git status across all worktrees in a feature
