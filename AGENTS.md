# xfeat — Agent Guidelines

## Project overview

Rust CLI (2024 edition) for managing git worktrees across multiple repositories. Enables parallel feature development without branch switching or stashing. Each feature gets its own isolated workspace via git worktrees.

- **Binary name:** `xfeat`
- **Shell alias:** `xf`
- **Repository:** https://github.com/just-sultanov/xfeat

## Architecture

```
src/
├── main.rs          # Entry point, CLI dispatch
├── cli.rs           # clap derive CLI definition (new, list, remove, sync, add, init)
├── config.rs        # Env config (XF_REPOS_DIR, XF_FEATURES_DIR) with tilde/env expansion
├── worktree.rs      # Git operations (create, fetch, rebase, branch detection)
├── error.rs         # Typed errors via thiserror
├── init.rs          # Shell init generation (embeds shell/ via include_dir!)
└── commands/        # Per-command implementations
    ├── add.rs
    ├── list.rs
    ├── new.rs
    ├── remove.rs
    └── sync.rs
```

- `build.rs` — injects version/git-sha/built-at via cargo env vars
- `shell/init.zsh` — `xf` wrapper function + zsh tab completions
- `bin/` — development scripts (test, lint, build, dist, publish, install)
- `install.sh` / `install.ps1` — standalone install scripts for end users
- `mise.toml` — tool versions and task definitions

## Dependencies

| Crate           | Purpose                             |
| --------------- | ----------------------------------- |
| `anyhow`        | Error handling in main              |
| `clap` (derive) | CLI argument parsing                |
| `thiserror`     | Typed domain errors                 |
| `shellexpand`   | Tilde and env var path expansion    |
| `include_dir`   | Embed shell scripts at compile time |

## Development commands

When `mise` is available, use `mise run <task>`. Otherwise, run `bin/<script>` directly.

| Action              | Command                                             |
| ------------------- | --------------------------------------------------- |
| Test                | `mise run test` or `bin/test`                       |
| Lint (fmt + clippy) | `mise run lint` or `bin/lint`                       |
| Check (lint + test) | `mise run check` or `bin/check`                     |
| Clean               | `mise run clean` or `bin/clean`                     |
| Build (macOS arm)   | `mise run build-macos-arm` or `bin/build-macos-arm` |
| Build (macOS x86)   | `mise run build-macos-x86` or `bin/build-macos-x86` |
| Build (Linux)       | `mise run build-linux` or `bin/build-linux`         |
| Build (Windows)     | `mise run build-windows` or `bin/build-windows`     |
| Setup tools         | `mise run setup-tools` or `bin/setup-tools`         |
| Publish to crates.io| `mise run publish-crates` or `bin/publish-crates`   |

Always run `mise run check` before committing changes.

## Code conventions

- Rust 2024 edition
- No comments unless explicitly asked
- Use `anyhow::Result` for main flow, `thiserror` for typed domain errors (`src/error.rs`)
- Use clap derive macros for CLI definitions
- Follow existing patterns in `commands/` when adding new commands
- Shell scripts in `bin/` use `#!/usr/bin/env bash` with `set -euo pipefail`
- Paths support absolute, relative, and tilde (`~`) — resolved via `shellexpand`

## Error handling

- `src/error.rs` defines the `Error` enum with variants: `RepoNotFound`, `WorktreeExists`, `GitCommand`, `RebaseConflict`, `Io`
- Commands return `anyhow::Result<()>` and convert `Error` via `?`
- Git commands are executed via `std::process::Command` with `-C` flag for repo path

## Configuration

Two environment variables control behavior:

| Variable          | Description                                   | Default                |
| ----------------- | --------------------------------------------- | ---------------------- |
| `XF_REPOS_DIR`    | Directory containing source git repositories  | `~/workspace/repos`    |
| `XF_FEATURES_DIR` | Directory where feature worktrees are created | `~/workspace/features` |

Paths are resolved: env vars expanded → tilde expanded → relative made absolute via cwd.

---

## Using xfeat (for users)

### Quick start

```bash
# Set up project context
export XF_REPOS_DIR=~/projects/store/repos
export XF_FEATURES_DIR=~/projects/store/features

# Initialize shell integration (add to ~/.zshrc)
eval "$(xfeat init zsh)"
```

### Typical workflow

```bash
# 1. Create a feature and add worktrees
xf new checkout-v2
xf add checkout-v2 payment-service checkout-api

# 2. Switch to the feature and work
xf switch checkout-v2
cd payment-service
# make changes, commit, push

# 3. Stay in sync with main before merging
xf sync checkout-v2

# 4. Clean up when done
xf remove checkout-v2
```

### Commands

| Command                                       | Description                                          |
| --------------------------------------------- | ---------------------------------------------------- |
| `xf new <feature>`                            | Create empty feature directory                       |
| `xf add <feature> <repos...>`                 | Add git worktrees to a feature                       |
| `xf add <feature> <repos...> --from <branch>` | Branch from specific source branch                   |
| `xf add <feature> <repos...> --branch <name>` | Use custom branch name                               |
| `xf list`                                     | List all features with worktrees and branches        |
| `xf sync <feature>`                           | Rebase feature onto latest origin/main               |
| `xf remove <feature>`                         | Remove feature and its worktrees (with confirmation) |
| `xf remove <feature> --yes`                   | Remove without confirmation                          |
| `xf switch <feature>`                         | cd into feature directory (shell wrapper)            |

### AI-assisted development

Run multiple AI coding agents on different features simultaneously — each gets its own isolated workspace:

```bash
# Terminal 1
xf new ai-payment-fix
xf add ai-payment-fix payment-service --from develop
xf switch ai-payment-fix

# Terminal 2
xf new ai-checkout-v3
xf add ai-checkout-v3 checkout-api frontend --from develop
xf switch ai-checkout-v3
```

Both agents work independently. Sync before merging:

```bash
xf sync ai-payment-fix
xf sync ai-checkout-v3
```

### Important notes

- Feature directories contain git worktrees, not clones
- Worktrees are named after source repos in `$XF_REPOS_DIR`
- Branch names default to the feature name unless `--branch` is specified
- `--from` specifies the source branch to create the feature branch from
- Always run `xf sync` before merging to ensure the feature is up to date
- The `xf` shell wrapper (from `xfeat init zsh`) provides `xf switch` and tab completion
