# Feature: `xf add` Command

## Overview

Add worktrees for repositories to an existing feature. Useful when you need to add more repos to a feature that was already created.

## Usage

```bash
xf add <feature-name> <repos...>
```

## Examples

```bash
# Feature exists with one worktree
xf list
└── JIRA-123
    └── service-a (JIRA-123)

# Add another repo to the existing feature
xf add JIRA-123 service-b

# Output:
Feature 'JIRA-123' already exists.
  Creating: service-b
Feature 'JIRA-123' updated.

# Try to add a repo that already has a worktree
xf add JIRA-123 service-a

# Output:
Feature 'JIRA-123' already exists.
  Skipping: service-a (worktree exists)
No new worktrees added.
```

## Implementation Plan

### 1. Rust — New Command `xfeat add`

#### `src/cli.rs`

Add variant to `Commands` enum:

```rust
#[command(name = "add", about = "Add worktrees for repos to an existing feature")]
Add {
    feature_name: String,
    repos: Vec<String>,
},
```

#### `src/commands/add.rs` — New File

Logic:
1. Check that feature directory exists in `config.features_dir`. If not → error: "feature not found, use `xf new` to create it"
2. For each repo:
   - Check that repo exists in `config.repos_dir`. If not → error
   - Check if worktree directory already exists at `features_dir/<feature>/<repo>`
   - If exists → warning: "Skipping: <repo> (worktree exists)"
   - If not → create worktree using `worktree::create_worktree`
3. If no new worktrees were created → "No new worktrees added." (exit 0)
4. If worktrees were created → "Feature '<feature>' updated."

#### `src/commands/mod.rs`

Add `pub mod add;`

#### `src/main.rs`

Handle `Commands::Add` — call `commands::add::run`

### 2. Shell — `xf add` Wrapper and Autocomplete

#### `shell/init.zsh`

Add case in `xf()` function:

```zsh
add)
  local feature="$1"
  shift
  xfeat add "$feature" "$@"
  ;;
```

Add to `commands` array:

```zsh
commands=(
  ...
  "add:add worktrees to an existing feature"
)
```

Add `add` to the autocomplete case for feature names:

```zsh
remove|sync|switch|add)
  features_dir="${(e)features_dir}"
  features_dir="${~features_dir}"
  if [[ -d "$features_dir" ]]; then
    features=("${(@f)$(command ls -1 "$features_dir" 2>/dev/null)}")
    if (( ${#features} > 0 )); then
      _describe 'feature' features
    fi
  fi
  ;;
```

### 3. Tests

#### `src/commands/add.rs` — `#[cfg(test)] mod tests`

- `test_add_repo_to_existing_feature` — adds a new worktree to existing feature
- `test_add_existing_worktree_skips` — skips existing worktree, exits 0 with informational message
- `test_add_feature_not_found` — error when feature directory doesn't exist
- `test_add_repo_not_found` — error when repo doesn't exist in repos_dir
- `test_add_mixed_new_and_existing` — adds new repos, skips existing ones

Use `TestEnv` fixture pattern (same as other commands).

### 4. Documentation

#### `README.md`

Add section after `xf new`:

```markdown
### `xf add`

Add worktrees for repos to an existing feature:

```bash
xf add <feature-name> <repos...>
```

**Example:**

```bash
xf add JIRA-123 service-b
```
```

#### `CHANGELOG.md`

Add entry for 0.1.3:

```markdown
## 0.1.3 - YYYY-MM-DD

### Added

- `xf add` — add worktrees for repos to an existing feature
```

#### `docs/mvp.md`

Update `Implemented Commands` and `Shell Wrapper xf` sections.
