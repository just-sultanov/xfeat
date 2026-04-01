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
├── worktree.rs       # Git worktree operations
└── commands/
    ├── mod.rs
    ├── new.rs        # Implementation of `new` command
    └── list.rs       # Implementation of `list` command
```

## License

MIT — see [LICENSE](LICENSE) for details.
