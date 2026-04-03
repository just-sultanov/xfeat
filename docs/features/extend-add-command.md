# Feature: Extend `xf add` with `--from` and `--branch`

## Overview

Add `--from` and `--branch` flags to the `xf add` command for flexible worktree creation from any branch with custom branch names.

## Usage

```bash
xf add <feature-name> <repos...>
xf add <feature-name> <repos...> --from <branch>
xf add <feature-name> <repos...> --branch <branch-name>
xf add <feature-name> <repos...> --from <branch> --branch <branch-name>
```

## Examples

```bash
# Default: branch named after feature, from default branch
xf add JIRA-123 service-a

# Create branch JIRA-123 from master
xf add JIRA-123 service-a --from master

# Create branch JIRA-123 from remote develop
xf add JIRA-123 service-a --from origin/develop

# Create custom branch name from master
xf add JIRA-123 service-a --from master --branch bugfix/JIRA-123

# Multiple repos, same flags apply to all
xf add JIRA-123 service-a service-b --from develop --branch fix/JIRA-123
```

## Implementation Plan

### 1. `src/cli.rs`

Add `--from` and `--branch` options to `Add`:

```rust
Add {
    feature_name: String,
    repos: Vec<String>,
    /// Create branch from a specific branch (e.g., develop, origin/develop)
    #[arg(long)]
    from: Option<String>,
    /// Custom branch name (defaults to feature name)
    #[arg(long)]
    branch: Option<String>,
},
```

### 2. `src/worktree.rs`

Add three new functions and modify `create_worktree`:

#### `fn fetch_repo(repo: &Path) -> Result<()>`

Run `git fetch origin` in the source repository to ensure remote refs are up to date.

#### `fn branch_exists(repo: &Path, branch: &str) -> bool`

Check if a branch exists:
1. `git rev-parse --verify refs/heads/<branch>` (local branches)
2. `git rev-parse --verify refs/remotes/<branch>` (remote refs like `origin/develop`)

Return `true` if either exists.

#### `fn is_branch_in_use(repo: &Path, branch: &str) -> Option<PathBuf>`

Check if a branch is already checked out in any worktree:
- Run `git worktree list --porcelain` from the source repo
- Parse output for `branch <name>` lines
- Return the worktree path if found, `None` otherwise

#### Modify `create_worktree`

Change signature:
```rust
pub fn create_worktree(
    source_repo: &Path,
    worktree_path: &Path,
    from_branch: Option<&str>,
    branch_name: &str,
) -> Result<()>
```

If `from_branch` is provided, add it to git args before `-b <branch_name>`:
```
git -C <source> worktree add <path> <from_branch> -b <branch_name>
```

Otherwise:
```
git -C <source> worktree add <path> -b <branch_name>
```

### 3. `src/commands/add.rs`

New signature:
```rust
pub fn run(
    feature_name: &str,
    repos: &[String],
    from_branch: Option<&str>,
    branch_name: Option<&str>,
    config: &Config,
) -> anyhow::Result<()>
```

Logic:
1. `let target_branch = branch_name.unwrap_or(feature_name);`
2. If `from_branch` is provided:
   - For each repo: `fetch_repo()` → `branch_exists()`, bail with error if not found
3. If `branch_name` is provided:
   - For each repo: `is_branch_in_use()`, bail with error showing worktree path if in use
4. For each repo (skip if worktree exists):
   - `create_worktree(&repo_path, &worktree_path, from_branch, target_branch)`
5. Print appropriate messages

### 4. `src/main.rs`

Update dispatch:
```rust
cli::Commands::Add { feature_name, repos, from, branch } => {
    commands::add::run(&feature_name, &repos, from.as_deref(), branch.as_deref(), &config)?;
}
```

### 5. `shell/init.zsh`

No changes needed — `"$@"` already proxies all arguments including `--from` and `--branch`.

### 6. Tests

- `test_add_with_from_branch` — creates worktree from specified branch
- `test_add_with_from_branch_not_found` — error when branch doesn't exist
- `test_add_with_custom_branch` — creates worktree with custom branch name
- `test_add_with_branch_already_in_use` — error when branch is checked out elsewhere
- `test_add_with_from_and_branch` — both flags together

Use `TestEnv` fixture pattern (same as other commands).

### 7. `README.md`

Update `xf add` section with `--from` and `--branch` documentation.

## Validation Rules

1. `--from` branch must exist (checked via `git rev-parse --verify` after `git fetch`)
2. `--branch` must not be checked out in any worktree (checked via `git worktree list --porcelain`)
3. Both checks run for each repository before any worktree creation
4. If `--branch` is not provided, use feature name as branch name
5. `--from` supports short names (`develop`) and remote refs (`origin/develop`)
