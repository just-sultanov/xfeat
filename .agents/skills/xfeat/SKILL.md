---
name: xfeat
description: Use the xfeat CLI to manage git worktrees across multiple repositories — create features, add worktrees, and sync with main
license: MIT
metadata:
  audience: developers
  workflow: git-worktree
---

## What I do

I teach you how to use `xfeat` (aliased as `xf`) — a CLI utility for managing git worktrees across multiple repositories. Each feature gets its own isolated workspace via git worktrees, enabling parallel development without branch switching or stashing.

## Prerequisites

### Installation

- **Using cargo:** `cargo install xfeat --locked`
- **Using mise:** `mise install github:just-sultanov/xfeat`
- **Using curl (macOS/Linux):** `curl -fsSL https://raw.githubusercontent.com/just-sultanov/xfeat/main/install.sh | bash`
- **Using PowerShell (Windows):** `irm https://raw.githubusercontent.com/just-sultanov/xfeat/main/install.ps1 | iex`
- **Pre-built binaries:** [GitHub Releases](https://github.com/just-sultanov/xfeat/releases)

### Environment

- `xfeat` must be available as `xf` in PATH
- Environment variables must be set:
  - `XF_REPOS_DIR` — directory containing source git repositories
  - `XF_FEATURES_DIR` — directory where feature worktrees are created

## Commands

### `xf new <feature-name>`

Create a new empty feature directory:

```bash
xf new STORY-123-add-payment
```

This creates an empty directory under `$XF_FEATURES_DIR/<feature-name>/`. Add worktrees with `xf add`.

### `xf add <feature-name> <repos...>`

Add git worktrees for repositories to an existing feature:

```bash
# Add repos — branches named after the feature
xf add STORY-123-add-payment payment-service checkout-service

# Add repos, branching from a specific source branch
xf add STORY-123-add-payment payment-service --from develop

# Add repos with a custom branch name
xf add STORY-123-add-payment payment-service --branch feature/TASK-123-add-payment

# Combine: branch from 'develop' with a custom name
xf add STORY-123-add-payment payment-service --from develop --branch feature/TASK-123-add-payment
```

Skips repositories that already have worktrees in the feature.

### `xf list`

List all features with their worktrees and current branches:

```bash
xf list
xf list --path
```

Example output:

```
├── STORY-123-add-payment
│   ├── payment-service
│   │   branch: STORY-123-add-payment
│   └── checkout-service
│       branch: STORY-123-add-payment
├── STORY-456-redesign-checkout
│   └── frontend
        branch: STORY-456-redesign-checkout
└── STORY-789-empty (empty)
```

With `--path`, worktree paths are also shown:

```
├── STORY-123-add-payment
│   └── payment-service
│       branch: STORY-123-add-payment
│       path: ~/workspace/features/STORY-123-add-payment/payment-service
└── STORY-456-redesign-checkout
    └── frontend
        branch: feature/TASK-456-redesign-checkout
        path: ~/workspace/features/STORY-456-redesign-checkout/frontend
```

Empty features (created with `xf new` but without worktrees yet) are shown with `(empty)`.

### `xf sync <feature-name>`

Sync a feature with the latest main branch from source repos:

```bash
xf sync STORY-123-add-payment
xf sync STORY-123-add-payment --from develop
```

For each worktree in the feature:

1. Fetches latest changes from remote
2. Rebases the feature branch onto `origin/<branch>` (default: auto-detected from origin/HEAD)
3. Stops on first conflict with an error message

Typical workflow before merging:

```bash
xf sync STORY-123-add-payment   # rebase onto latest main
# resolve any conflicts if needed
xf sync STORY-123-add-payment   # verify clean sync
```

### `xf remove <feature-name>`

Remove a feature and its worktrees. Prompts for confirmation by default:

```bash
xf remove STORY-123-add-payment
xf remove STORY-123-add-payment --yes   # skip confirmation (for scripts)
```

Shows a summary of what will be removed, including warnings about uncommitted changes.

## Typical workflow

1. **Create a feature and add worktrees:**

   ```bash
   xf new checkout-v2
   xf add checkout-v2 payment-service checkout-service
   ```

2. **Work on the feature:**

   ```bash
   cd "$XF_FEATURES_DIR/checkout-v2"
   cd payment-service
   # make changes, commit, push
   ```

3. **Stay in sync with main:**

   ```bash
   xf sync checkout-v2
   ```

4. **Clean up when done:**

   ```bash
   xf remove checkout-v2
   ```

## AI-assisted development

Run multiple AI coding agents on different features simultaneously — each agent gets its own isolated workspace:

```bash
# Terminal 1 — working on payment fix
xf new ai-payment-fix
xf add ai-payment-fix payment-service --from develop

# Terminal 2 — working on checkout redesign
xf new ai-checkout-v3
xf add ai-checkout-v3 checkout-service frontend --from develop
```

Both agents work independently on their own branches. Before merging, sync each feature:

```bash
xf sync ai-payment-fix
xf sync ai-checkout-v3
```

## Important notes

- Each feature directory under `$XF_FEATURES_DIR` contains git worktrees (not clones)
- Worktrees are named after the source repositories in `$XF_REPOS_DIR`
- Branch names default to the feature name unless `--branch` is specified
- `--from` specifies the source branch to create the feature branch from
- Always run `xf sync` before merging to ensure the feature is up to date
- The `xf` shell wrapper (from `xfeat init zsh` or `xfeat init bash`) provides tab completion for commands and arguments
