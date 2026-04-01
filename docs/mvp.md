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

## Project Structure

```
src/
├── main.rs           # Entry point, arg parsing, command dispatch
├── cli.rs            # CLI definition (clap)
├── config.rs         # Configuration (env vars with defaults)
├── error.rs          # Custom error types
├── worktree.rs       # Git worktree operations
└── commands/
    └── new.rs        # Implementation of `new` command
```

## Dependencies

| Crate             | Purpose                          |
| ----------------- | -------------------------------- |
| `clap` (derive)   | CLI argument parsing             |
| `thiserror`       | Custom error types               |
| `anyhow`          | Application-level error handling |
| `shellexpand`     | Expand `~` in paths              |
| `path-absolutize` | Absolute path handling           |

## CLI Interface

```
xfeat new <feature-name> <repos...>
```

## Implementation Steps

### 1. Project Setup

- Add dependencies to `Cargo.toml`
- Create module structure

### 2. `config.rs` — Configuration

- `Config` struct with `repos_dir` and `features_dir` fields
- Read from env vars `XF_REPOS_DIR` / `XF_FEATURES_DIR`
- Defaults: `~/workspace/repos`, `~/workspace/features`
- Expand `~` via `shellexpand`

### 3. `error.rs` — Error Types

- `Error::RepoNotFound` — repository not found in repos_dir
- `Error::WorktreeExists` — worktree already exists
- `Error::GitCommand` — git command execution failed
- `Error::Io` — filesystem error

### 4. `cli.rs` — CLI Definition

- `Cli` struct with `#[command(subcommand)]`
- `Commands::New { feature_name: String, repos: Vec<String> }`

### 5. `worktree.rs` — Git Worktree Logic

- `fn create_worktree(source_repo: &Path, worktree_path: &Path, branch: &str) -> Result<()>`
- Execute `git worktree add <path> -b <branch>` via `std::process::Command`
- Validate that source is a git repository

### 6. `commands/new.rs` — `new` Command

- Validate: all specified repos exist in `repos_dir`
- Check: feature directory does not already exist
- Create feature directory
- For each repo: create worktree with new branch
- On error: rollback (remove created worktrees)

### 7. `main.rs` — Entry Point

- Initialize clap
- Load config
- Dispatch to command
- Print errors and exit code

### 8. Shell Wrapper `xf`

- Bash/zsh/fish script that:
  - Calls `xfeat new "$@"` to create
  - On success — `cd` into feature directory
  - Prints final path
- Autocompletion for repository names (directory listing from `XF_REPOS_DIR`):
  - **Bash**: `complete -F _xfeat_complete xf` with `compgen -W "$(ls "$XF_REPOS_DIR")"`
  - **Zsh**: `_describe 'repos' repos` via `#compdef xf`
  - **Fish**: `complete -c xf -a "(ls $XF_REPOS_DIR 2>/dev/null || ls ~/workspace/repos)"`

## Git Worktree Example

```bash
git -C ~/workspace/repos/service-1 worktree add ~/workspace/features/JIRA-123-fix-issue/service-1 -b JIRA-123-fix-issue
```

## Rollback Strategy

If worktree creation fails for any repository — remove all already-created worktrees and the feature directory to avoid leaving a partial state.
