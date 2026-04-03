# Feature: Simplify `xf new` Command

## Overview

Simplify `xf new` to only create an empty feature directory. All worktree creation logic moves to `xf add`. This separates concerns and makes both commands simpler and more focused.

## New Workflow

```bash
# Before: single command
xf new JIRA-123 service-a service-b

# After: two-step process
xf new JIRA-123
xf add JIRA-123 service-a service-b --from develop --branch bugfix/JIRA-123
```

## Rationale

- `xf new` becomes trivial — just creates an empty directory
- `xf add` owns all worktree logic (base branch, custom branch names, validation)
- No rollback needed in `new` (no partial state possible)
- Consistent UX: `add` is the only way to add worktrees, regardless of whether the feature is new or existing

## Implementation Plan

### 1. `src/cli.rs`

Simplify `New` variant — remove `repos`, `--from`, `--branch`:

```rust
/// Create a new feature directory
New {
    /// Name of the feature
    feature_name: String,
},
```

### 2. `src/commands/new.rs`

Simplify to only create an empty directory:

```rust
pub fn run(feature_name: &str, config: &Config) -> anyhow::Result<()> {
    let feature_dir = config.features_dir.join(feature_name);
    if feature_dir.exists() {
        anyhow::bail!("feature directory '{}' already exists", feature_dir.display());
    }
    fs::create_dir_all(&feature_dir)?;
    println!("Feature '{feature_name}' created at: {}", feature_dir.display());
    Ok(())
}
```

Remove `rollback` function entirely.

### 3. `src/main.rs`

Simplify dispatch:

```rust
cli::Commands::New { feature_name } => {
    commands::new::run(&feature_name, &config)?;
}
```

### 4. `shell/init.zsh`

- Remove repository autocomplete for `new` — now `xf new` only takes a feature name
- Update description: `"new:create a new feature"`

### 5. Tests

#### Remove from `new.rs` (7 tests):
- `test_new_command_creates_worktree_and_branch`
- `test_new_command_creates_feature_directory`
- `test_new_command_worktree_linked_to_source`
- `test_new_command_multiple_repos_all_have_correct_branch`
- `test_new_command_worktree_is_valid_git_repo`
- `test_rollback_cleans_up_created_worktrees`
- `test_new_command_fails_for_missing_repo`

#### Keep and update:
- `test_new_command_fails_for_existing_feature` → `test_new_fails_if_feature_exists`
- `test_new_command_creates_feature_directory` → `test_new_creates_empty_directory`

#### Add to `new.rs`:
- `test_new_creates_empty_directory` — verifies directory is created and is empty
- `test_new_fails_if_feature_exists` — verifies error on duplicate

#### Add to `add.rs`:
- `test_add_to_empty_feature` — adds worktree to a feature created via `xf new` (empty directory)

### 6. Documentation

#### `README.md`

Update `xf new` section:

```markdown
### `xfeat new`

Create a new empty feature directory:

```bash
xfeat new <feature-name>
```

**Example:**

```bash
xfeat new JIRA-123-fix-issue
```

Then add worktrees with `xfeat add`:

```bash
xfeat add JIRA-123-fix-issue service-1 service-2 lib-1
```
```

Update `xf add` examples to show the full workflow.

#### `CHANGELOG.md`

Add breaking change note to 0.1.3:

```markdown
### Changed

- `xf new` now only creates an empty feature directory (use `xf add` to add worktrees)
```

#### `docs/mvp.md`

Update `Implemented Commands` section:

```markdown
### `xfeat new <feature-name>`

Creates an empty feature directory. Use `xfeat add` to add worktrees.
```
