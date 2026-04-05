---
name: xfeat
description: Use the xfeat CLI to manage git worktrees across multiple repositories — create features, add worktrees, sync with main, and switch between feature workspaces
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
xf new JIRA-123-fix-issue
```

This creates an empty directory under `$XF_FEATURES_DIR/<feature-name>/`. Add worktrees with `xf add`.

### `xf add <feature-name> <repos...>`

Add git worktrees for repositories to an existing feature:

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

### `xf list`

List all features with their worktrees and current branches:

```bash
xf list
```

Example output:

```
├── JIRA-123
│   ├── service-1 (JIRA-123)
│   └── service-2 (JIRA-123)
├── JIRA-456
│   └── service-1 (JIRA-456)
└── JIRA-789 (empty)
```

Empty features (created with `xf new` but without worktrees yet) are shown with `(empty)`.

### `xf sync <feature-name>`

Sync a feature with the latest main branch from source repos:

```bash
xf sync JIRA-123-fix-issue
```

For each worktree in the feature:

1. Fetches latest changes from remote
2. Rebases the feature branch onto `origin/main` (auto-detected)
3. Stops on first conflict with an error message

Typical workflow before merging:

```bash
xf sync JIRA-123-fix-issue   # rebase onto latest main
# resolve any conflicts if needed
xf sync JIRA-123-fix-issue   # verify clean sync
```

### `xf remove <feature-name>`

Remove a feature and its worktrees. Prompts for confirmation by default:

```bash
xf remove JIRA-123-fix-issue
xf remove JIRA-123-fix-issue --yes   # skip confirmation (for scripts)
```

Shows a summary of what will be removed, including warnings about uncommitted changes.

### `xf switch <feature-name>`

Switch to a feature directory (shell wrapper function):

```bash
xf switch JIRA-123-fix-issue   # cd into the feature directory
```

This is provided by the shell initialization (`eval "$(xfeat init zsh)"`), not the binary directly.

## Typical workflow

1. **Create a feature and add worktrees:**

   ```bash
   xf new checkout-v2
   xf add checkout-v2 payment-service checkout-api
   ```

2. **Switch to the feature and work:**

   ```bash
   xf switch checkout-v2
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
xf switch ai-payment-fix

# Terminal 2 — working on checkout redesign
xf new ai-checkout-v3
xf add ai-checkout-v3 checkout-api frontend --from develop
xf switch ai-checkout-v3
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
- The `xf` shell wrapper (from `xfeat init zsh`) provides `xf switch` and tab completion
