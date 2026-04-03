# Release Process

## Prerequisites

Ensure required tools are installed:

```bash
mise run setup-tools
```

## Steps

### 1. Bump version

```bash
mise run bump-version patch   # 0.1.0 → 0.1.1
mise run bump-version minor   # 0.1.0 → 0.2.0
mise run bump-version major   # 0.1.0 → 1.0.0
```

### 2. Update CHANGELOG.md

Edit `CHANGELOG.md` to reflect the changes in the new version. Update the date placeholder:

```markdown
## [0.1.1] - 2026-04-03
```

### 3. Commit changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "Release v0.1.1"
```

### 4. Create git tag

```bash
mise run pre-publish
```

This validates:

- You are on `main` branch
- Working tree is clean
- Tag does not already exist

Then creates an annotated tag with message `Release <version>`.

### 5. Push to remote

```bash
mise run publish
```

This pushes the current branch and the new tag to `origin`.

### 6. Draft release

The `release.yml` GitHub Actions workflow triggers automatically on tag push:

1. Runs lint and tests
2. Builds binaries for all platforms (Linux, macOS arm/x86, Windows)
3. Packages them into archives
4. Creates a **draft** release on GitHub with:
   - Auto-generated changelog from git commits
   - All platform binaries attached

### 7. Review and publish

1. Go to GitHub → Releases → Drafts
2. Review the auto-generated changelog and edit if needed
3. Click **Publish release**

## Install after release

```bash
mise install github:just-sultanov/xfeat
```
