# xfeat

CLI utility for managing git worktrees across multiple repositories.

## Overview

`xfeat` is designed for developers working on multiple projects simultaneously. Each project has its own workspace with `repos` (source repositories) and `features` (worktree branches) directories. Environment variables `XF_REPOS_DIR` and `XF_FEATURES_DIR` are scoped per-project, allowing isolated feature development across multiple repositories within a single project context.

By leveraging git worktrees, `xfeat` enables parallel development on multiple features without the overhead of cloning repositories or switching branches. Each feature gets its own isolated workspace, making it ideal for AI-assisted development where multiple coding agents can work on different features simultaneously without conflicts.

## Installation

```bash
# via mise
mise install github:just-sultanov/xfeat

# via cargo
cargo install xfeat  # soon
```

## Quick Start

Set up your project workspace and start developing features in parallel:

```bash
cd ~/projects/store
export XF_REPOS_DIR=~/projects/store/repos
export XF_FEATURES_DIR=~/projects/store/features
```

```
~/projects/store/
├── repos/
│   ├── payment-service/
│   ├── checkout-api/
│   └── frontend/
└── features/           # empty
```

**1. Create a new feature and add worktrees:**

```bash
xf new checkout-v2
xf add checkout-v2 payment-service checkout-api
```

```
~/projects/store/
├── repos/
│   ├── payment-service/
│   ├── checkout-api/
│   └── frontend/
└── features/
    └── checkout-v2/
        ├── payment-service/  # worktree on branch checkout-v2
        └── checkout-api/     # worktree on branch checkout-v2
```

**2. Work on your feature** — each worktree is a fully independent git checkout:

```bash
xf switch checkout-v2
cd payment-service
# make changes, commit, push — no stashing, no branch switching
```

**3. Stay in sync with main:**

```bash
xf sync checkout-v2
```

**4. List all active features:**

```bash
xf list
```

```
├── checkout-v2
│   ├── payment-service (checkout-v2)
│   └── checkout-api (checkout-v2)
└── payment-refactor (empty)
```

**5. Done? Clean up:**

```bash
xf remove checkout-v2
```

Each feature gets its own git worktrees, so you can switch between projects and features instantly without stashing or switching branches.

## Workflows

### AI-assisted development

Run multiple AI coding agents (Claude Code, Codex, Cursor, etc.) on different features simultaneously — each agent gets its own isolated workspace with no risk of conflicts:

```bash
# Start two features in parallel
xf new ai-payment-fix
xf add ai-payment-fix payment-service --from develop

xf new ai-checkout-v3
xf add ai-checkout-v3 checkout-api frontend --from develop
```

Now launch AI agents in separate terminals:

```bash
# Terminal 1 — Claude Code working on payment fix
xf switch ai-payment-fix
cd payment-service
claude

# Terminal 2 — Codex working on checkout redesign
xf switch ai-checkout-v3
cd checkout-api
codex
```

Both agents work independently on their own branches. Before merging, sync each feature with the latest `main`:

```bash
xf sync ai-payment-fix
xf sync ai-checkout-v3
```

### Multiple projects & features simultaneously

Switch between projects and juggle multiple features without losing context:

```bash
# Project A — e-commerce
cd ~/projects/store
export XF_REPOS_DIR=~/projects/store/repos
export XF_FEATURES_DIR=~/projects/store/features

xf new checkout-v2
xf add checkout-v2 payment-service checkout-api

# Project B — analytics dashboard
cd ~/projects/analytics
export XF_REPOS_DIR=~/projects/analytics/repos
export XF_FEATURES_DIR=~/projects/analytics/features

xf new dashboard-redesign
xf add dashboard-redesign frontend backend
```

Use `xf switch` to jump between features instantly:

```bash
xf switch checkout-v2        # cd to features/checkout-v2
# work on checkout...

xf switch dashboard-redesign # cd to features/dashboard-redesign (in analytics project)
# work on dashboard...
```

View everything at a glance:

```bash
xf list
```

```
├── checkout-v2
│   ├── payment-service (checkout-v2)
│   └── checkout-api (checkout-v2)
├── payment-refactor
│   └── payment-service (payment-refactor)
└── dashboard-redesign
    ├── frontend (dashboard-redesign)
    └── backend (dashboard-redesign)
```

> **Tip:** Use `direnv` or `mise env` to automatically set `XF_REPOS_DIR` and `XF_FEATURES_DIR` when entering a project directory. Add an `.envrc` file:
>
> ```bash
> export XF_REPOS_DIR=~/projects/store/repos
> export XF_FEATURES_DIR=~/projects/store/features
> ```

## Commands

### `xfeat new`

Create a new empty feature directory:

```bash
xf new <feature-name>
```

**Example:**

```bash
xf new JIRA-123-fix-issue
```

This creates an empty directory. Add worktrees with `xf add`:

```bash
xf add JIRA-123-fix-issue service-1 service-2 lib-1
```

### `xfeat add`

Add worktrees for repositories to an existing feature:

```bash
xf add <feature-name> <repos...>
xf add <feature-name> <repos...> --from <branch>
xf add <feature-name> <repos...> --branch <branch-name>
xf add <feature-name> <repos...> --from <branch> --branch <branch-name>
```

**Examples:**

```bash
# Add repos — branches named after the feature
xf add JIRA-123 payment-service checkout-api

# Add repos, branching from a specific source branch
xf add JIRA-123 payment-service --from develop

# Add repos with a custom branch name
xf add JIRA-123 payment-service --branch bugfix/JIRA-123

# Combine: branch from 'develop' with a custom name
xf add JIRA-123 payment-service --from develop --branch bugfix/JIRA-123
```

Skips repositories that already have worktrees in the feature.

### `xfeat list`

List all features with their worktrees and current branches:

```bash
xf list
```

**Example output:**

```
├── JIRA-123
│   ├── service-1 (JIRA-123)
│   └── service-2 (JIRA-123)
├── JIRA-456
│   └── service-1 (JIRA-456)
└── JIRA-789 (empty)
```

Empty features (created with `xf new` but without worktrees yet) are shown with `(empty)`.

### `xfeat sync`

Sync a feature with the latest main branch from source repos:

```bash
xf sync <feature-name>
```

For each worktree in the feature:

1. Fetches latest changes from remote
2. Rebases the feature branch onto `origin/main` (auto-detected)
3. Stops on first conflict with an error message

**Example:**

```bash
xf sync JIRA-123-fix-issue
```

**Typical workflow before merging:**

```bash
xf sync JIRA-123-fix-issue   # rebase onto latest main
# resolve any conflicts if needed
xf sync JIRA-123-fix-issue   # verify clean sync
# merge or create PR
```

### `xfeat remove`

Remove a feature and its worktrees. Prompts for confirmation by default:

```bash
xf remove <feature-name>
xf remove <feature-name> --yes   # skip confirmation (for scripts)
```

**Example output:**

```
Feature 'JIRA-123' contains:
  - service-1 (JIRA-123)
  - service-2 (JIRA-123) ⚠ has uncommitted changes

Remove feature 'JIRA-123'? [y/N] y
Feature 'JIRA-123' removed.
```

### `xfeat init`

Generate shell initialization code with autocompletion and `xf` wrapper function:

```bash
eval "$(xfeat init zsh)"
```

**Supported shells:** `zsh`

The `xf` wrapper:

- `xf new <feature>` — creates an empty feature directory
- `xf add <feature> <repos...>` — adds worktrees to a feature
- `xf switch <feature>` — `cd` into a feature directory
- `xf remove <feature>` — removes feature (with confirmation) and `cd`s out if needed
- `xf sync <feature>` — syncs feature with main
- `xf list` and other commands — proxied to `xfeat`
- Tab completion for repository names (`xf add <TAB>`), feature names (`xf new <TAB>`, `xf remove <TAB>`, `xf sync <TAB>`, `xf switch <TAB>`)

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

## License

MIT — see [LICENSE](LICENSE) for details.
