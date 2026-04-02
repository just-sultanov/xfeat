# xfeat

CLI utility for managing git worktrees across multiple repositories.

## Overview

`xfeat` is designed for developers working on multiple products simultaneously. Each product has its own workspace with `repos` (source repositories) and `features` (worktree branches) directories. Environment variables `XF_REPOS_DIR` and `XF_FEATURES_DIR` are scoped per-project, allowing isolated feature development across multiple repositories within a single product context.

By leveraging git worktrees, `xfeat` enables parallel development on multiple features without the overhead of cloning repositories or switching branches. Each feature gets its own isolated workspace, making it ideal for AI-assisted development where multiple coding agents can work on different features simultaneously without conflicts.

## Installation

```bash
cargo install --path .
```

## Commands

### `xfeat new`

Create a new feature with worktrees for specified repositories:

```bash
xfeat new <feature-name> <repos...>
```

**Example:**

```bash
xfeat new JIRA-123-fix-issue service-1 service-2 lib-1
```

This creates:

```
~/workspace/features/JIRA-123-fix-issue/
├── service-1   # worktree on branch JIRA-123-fix-issue
├── service-2   # worktree on branch JIRA-123-fix-issue
└── lib-1       # worktree on branch JIRA-123-fix-issue
```

### `xfeat list`

List all features with their worktrees and current branches:

```bash
xfeat list
```

**Example output:**

```
├── JIRA-123
│   ├── service-1 (JIRA-123)
│   └── service-2 (JIRA-123)
└── JIRA-456
    └── service-1 (JIRA-456)
```

### `xfeat remove`

Remove a feature and its worktrees. Prompts for confirmation by default:

```bash
xfeat remove <feature-name>
xfeat remove <feature-name> --yes   # skip confirmation
```

**Example output:**

```
Feature 'JIRA-123' contains:
  - service-1 (JIRA-123)
  - service-2 (JIRA-123) ⚠ has uncommitted changes

Remove feature 'JIRA-123'? [y/N] y
Feature 'JIRA-123' removed.
```

### `xfeat sync`

Sync a feature with the latest main branch from source repos:

```bash
xfeat sync <feature-name>
```

For each worktree in the feature:
1. Fetches latest changes from remote
2. Rebases the feature branch onto `origin/main` (auto-detected)
3. Stops on first conflict with an error message

**Example:**

```bash
xfeat sync JIRA-123-fix-issue
```

### `xfeat init`

Generate shell initialization code with autocompletion and `xf` wrapper function:

```bash
eval "$(xfeat init zsh)"
```

**Supported shells:** `zsh`

The `xf` wrapper:
- `xf new <feature> <repos...>` — creates feature
- `xf switch <feature>` — `cd` into a feature directory
- `xf remove <feature>` — removes feature (with confirmation) and `cd`s out if needed
- `xf sync <feature>` — syncs feature with main
- `xf list` and other commands — proxied to `xfeat`
- Tab completion for repository names (`xf new <TAB>`), feature names (`xf remove <TAB>`, `xf sync <TAB>`, `xf switch <TAB>`)

Shell scripts are stored in `shell/` and embedded into the binary at compile time. They read `XF_REPOS_DIR` and `XF_FEATURES_DIR` from the environment on each invocation, making them compatible with tools like `direnv`. Tilde (`~`) in paths is expanded automatically.

## Configuration

Set environment variables per-project:

```bash
export XF_REPOS_DIR=~/projects/project-x/repos
export XF_FEATURES_DIR=~/projects/project-x/features
```

| Variable          | Description                                   | Default                |
| ----------------- | --------------------------------------------- | ---------------------- |
| `XF_REPOS_DIR`    | Directory containing source git repositories  | `~/workspace/repos`    |
| `XF_FEATURES_DIR` | Directory where feature worktrees are created | `~/workspace/features` |

Paths can be absolute (`/tmp/repos`), relative (`./repos`), or tilde-based (`~/repos`). All are resolved correctly.

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

## License

MIT — see [LICENSE](LICENSE) for details.
