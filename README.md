# xfeat

CLI utility for managing git worktrees across multiple repositories.

> **⚠️ Alpha:** This project is under active development. APIs and behavior may change.

## Rationale

Why? I work on several large products simultaneously.
In total, that's 90+ repositories, and a feature often touches multiple products at once.
This used to be a pain: constant branch switching, stashing, and a mess of open folders.

`xfeat` fixes this — each feature gets its own isolated workspace via git worktrees.
No context switching. No stashing. Perfect for parallel work with AI coding agents.

## Overview

`xfeat` is designed for developers working on multiple projects simultaneously. Each project has its own workspace with `repos` (source repositories) and `features` (worktree branches) directories. Environment variables `XF_REPOS_DIR` and `XF_FEATURES_DIR` are scoped per-project, allowing isolated feature development across multiple repositories within a single project context.

By leveraging git worktrees, `xfeat` enables parallel development on multiple features without the overhead of cloning repositories or switching branches. Each feature gets its own isolated workspace, making it ideal for AI-assisted development where multiple coding agents can work on different features simultaneously without conflicts.

## Installation

### Requirements

- **git** — xfeat relies on git worktrees

### Install

**Using cargo:**

```bash
cargo install xfeat --locked
```

**Using mise:**

```bash
mise install github:just-sultanov/xfeat
```

**Using curl (macOS / Linux):**

```bash
# Install to ~/.local/bin (default)
curl -fsSL https://raw.githubusercontent.com/just-sultanov/xfeat/main/install.sh | bash

# Install to a custom directory
curl -fsSL https://raw.githubusercontent.com/just-sultanov/xfeat/main/install.sh | bash -s -- --prefix /usr/local/bin
```

**Using PowerShell (Windows):**

```powershell
# Install to $env:USERPROFILE\.local\bin (default)
irm https://raw.githubusercontent.com/just-sultanov/xfeat/main/install.ps1 | iex

# Install to a custom directory
irm https://raw.githubusercontent.com/just-sultanov/xfeat/main/install.ps1 | iex -ArgumentList '-Prefix', 'C:\tools'
```

**Using pre-built binaries:**

Download the latest release from [GitHub Releases](https://github.com/just-sultanov/xfeat/releases).

| Platform               | File                                     |
| ---------------------- | ---------------------------------------- |
| macOS (Apple Silicon)  | `xfeat-aarch64-apple-darwin.tar.gz`      |
| macOS (Intel)          | `xfeat-x86_64-apple-darwin.tar.gz`       |
| Linux (x86_64, static) | `xfeat-x86_64-unknown-linux-musl.tar.gz` |
| Windows (x86_64)       | `xfeat-x86_64-pc-windows-gnu.zip`        |

## Quick Start

### Set up environment

Choose one of the following methods to configure your project:

**Using export:**

```bash
export XF_REPOS_DIR=~/projects/store/repos
export XF_FEATURES_DIR=~/projects/store/features
```

**Using direnv:**

Create an `.envrc` file:

```bash
export XF_REPOS_DIR=~/projects/store/repos
export XF_FEATURES_DIR=~/projects/store/features
```

**Using mise env:**

Add to `mise.toml`:

```toml
[env]
XF_REPOS_DIR = "~/projects/store/repos"
XF_FEATURES_DIR = "~/projects/store/features"
```

### Initialize shell integration

Add to your `~/.zshrc` or `~/.bashrc`:

```bash
eval "$(xfeat init zsh)"   # or: eval "$(xfeat init bash)"
```

### Create your first feature

Set up your project workspace and start developing features in parallel:

```
~/projects/store/
├── repos/
│   ├── payment-service/
│   ├── checkout-service/
│   └── frontend/
└── features/ # empty
```

**1. Create a new feature and add worktrees:**

```bash
xf new checkout-v2
xf add checkout-v2 payment-service checkout-service
```

```
~/projects/store/
├── repos/
│   ├── payment-service/
│   ├── checkout-service/
│   └── frontend/
└── features/
    └── checkout-v2/
        ├── payment-service/  # worktree on branch checkout-v2
        └── checkout-service/ # worktree on branch checkout-v2
```

**2. Work on your feature** — each worktree is a fully independent git checkout:

```bash
cd "$XF_FEATURES_DIR/checkout-v2"
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
│   ├── payment-service
│   │   branch: checkout-v2
│   └── checkout-service
│       branch: checkout-v2
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
xf add ai-checkout-v3 checkout-service frontend --from develop
```

Now launch AI agents in separate terminals:

```bash
# Terminal 1 — Claude Code working on payment fix
cd "$XF_FEATURES_DIR/ai-payment-fix"
cd payment-service
claude

# Terminal 2 — Codex working on checkout redesign
cd "$XF_FEATURES_DIR/ai-checkout-v3"
cd checkout-service
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
xf add checkout-v2 payment-service checkout-service

# Project B — analytics dashboard
cd ~/projects/analytics
export XF_REPOS_DIR=~/projects/analytics/repos
export XF_FEATURES_DIR=~/projects/analytics/features

xf new dashboard-redesign
xf add dashboard-redesign frontend backend
```

View everything at a glance:

```bash
xf list
```

```
├── checkout-v2
│   ├── payment-service
│   │   branch: checkout-v2
│   └── checkout-service
│       branch: checkout-v2
├── payment-refactor
│   └── payment-service
│       branch: payment-refactor
└── dashboard-redesign
    ├── frontend
    │   branch: dashboard-redesign
    └── backend
        branch: dashboard-redesign
```

## Commands

### `xfeat new`

Create a new empty feature directory:

```bash
xf new <feature-name>
```

**Example:**

```bash
xf new STORY-123-add-payment
```

This creates an empty directory. Add worktrees with `xf add`:

```bash
xf add STORY-123-add-payment payment-service checkout-service frontend
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
xf add STORY-123-add-payment payment-service checkout-service

# Add repos, branching from a specific source branch
xf add STORY-123-add-payment payment-service --from develop

# Add repos with a custom branch name
xf add STORY-123-add-payment payment-service --branch feature/TASK-123-add-payment

# Combine: branch from 'develop' with a custom name
xf add STORY-123-add-payment payment-service --from develop --branch feature/TASK-123-add-payment
```

Skips repositories that already have worktrees in the feature.

### `xfeat list`

List all features with their worktrees and current branches:

```bash
xf list
xf list --path
```

**Default output:**

```
├── STORY-123-add-payment
│   ├── payment-service
│   │   branch: STORY-123-add-payment
│   └── checkout-service
│       branch: STORY-123-add-payment
├── STORY-456-redesign-checkout
│   └── frontend
│       branch: STORY-456-redesign-checkout
└── STORY-789-empty (empty)
```

**With `--path`:**

```
├── STORY-123-add-payment
│   ├── payment-service
│   │   branch: STORY-123-add-payment
│   │   path: ~/workspace/features/STORY-123-add-payment/payment-service
│   └── checkout-service
│       branch: STORY-123-add-payment
│       path: ~/workspace/features/STORY-123-add-payment/checkout-service
└── STORY-456-redesign-checkout
    └── frontend
        branch: feature/TASK-456-redesign-checkout
        path: ~/workspace/features/STORY-456-redesign-checkout/frontend
```

Empty features (created with `xf new` but without worktrees yet) are shown with `(empty)`.

### `xfeat sync`

Sync a feature with the latest main branch from source repos:

```bash
xf sync <feature-name>
xf sync <feature-name> --from <branch>
```

For each worktree in the feature:

1. Fetches latest changes from remote
2. Rebases the feature branch onto `origin/<branch>` (default: auto-detected from origin/HEAD)
3. Stops on first conflict with an error message

**Example:**

```bash
xf sync STORY-123-add-payment
xf sync STORY-123-add-payment --from develop
```

**Typical workflow before merging:**

```bash
xf sync STORY-123-add-payment   # rebase onto latest main
# resolve any conflicts if needed
xf sync STORY-123-add-payment   # verify clean sync
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
Feature 'STORY-123-add-payment' contains:
  - payment-service (STORY-123-add-payment)
  - checkout-service (STORY-123-add-payment) ⚠ has uncommitted changes

Remove feature 'STORY-123-add-payment'? [y/N] y
Feature 'STORY-123-add-payment' removed.
```

### `xfeat init`

Generate shell initialization code with autocompletion and `xf` wrapper function:

```bash
eval "$(xfeat init zsh)"   # or: eval "$(xfeat init bash)"
```

**Supported shells:** `zsh`, `bash`

The `xf` wrapper:

- `xf new <feature>` — creates an empty feature directory
- `xf add <feature> <repos...>` — adds worktrees to a feature
- `xf remove <feature>` — removes feature (with confirmation) and `cd`s out if needed
- `xf sync <feature>` — syncs feature with main
- `xf list` and other commands — proxied to `xfeat`
- Tab completion for repository names (`xf add <TAB>`), feature names (`xf new <TAB>`, `xf remove <TAB>`, `xf sync <TAB>`)

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
