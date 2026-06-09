# Release Process

This document describes the release workflow for the play-wow-on-silicon project.

## Overview

We use a **Git Flow-inspired** workflow with `develop` and `main` branches:

- **`develop`**: Active development branch. All feature PRs merge here.
- **`main`**: Production branch. Merges from `develop` trigger automated releases.

## How Releases Work

### Automatic Releases (Default)

When a PR is merged from `develop` → `main`:

1. **GitHub Actions** triggers the release workflow
2. **release-plz** analyzes conventional commits since the last tag
3. **Version bump**: Determines next version (patch/minor/major) based on commit types
4. **CHANGELOG update**: Appends changes to each package's `CHANGELOG.md`
5. **Git tag**: Creates a tag like `v0.2.0`
6. **GitHub Release**: Publishes a release with generated notes

### Manual Releases

To trigger a release manually:

```bash
git checkout main
git merge origin/develop
git push origin main  # Triggers workflow
```

Or create a PR from `develop` to `main` and merge via GitHub UI.

## Version Bumping

Version bumps are **automatic** based on conventional commits:

| Commit Type | Version Bump | Example |
|-------------|--------------|---------|
| `fix:` | Patch (`0.2.0` → `0.2.1`) | Bug fix |
| `feat:` | Minor (`0.2.0` → `0.3.0`) | New feature |
| `feat!:` or `BREAKING CHANGE:` | Major (`0.2.0` → `1.0.0`) | Breaking change |
| `chore:`, `docs:`, `refactor:` | No bump | Internal changes |

### Synchronized Versioning

All packages share the same version:

- `wow-silicon-core`
- `wowplay` (CLI)
- `wow-silicon-integration`
- `wow-silicon-profiler` (Python tool)

When one package changes, **all** packages bump together.

## CHANGELOGs

Each package maintains its own `CHANGELOG.md` in the Keep a Changelog format:

- `packages/rust-core/CHANGELOG.md`
- `packages/cli/CHANGELOG.md`
- `packages/integration/CHANGELOG.md`
- `tools/profiler/CHANGELOG.md`

release-plz automatically appends changes under the appropriate version header.

## Skipping a Release

Add `[skip release]` to your commit message to prevent a release:

```bash
git commit -m "docs: update README [skip release]"
```

## Patch Releases

For urgent bugfixes that can't wait for the next regular release:

1. Create a hotfix branch from `main`:
   ```bash
   git checkout -b hotfix/critical-fix main
   ```

2. Make the fix and commit with `fix:` prefix

3. Open PR to `main` (not `develop`)

4. Merge — this triggers a patch release

5. Cherry-pick or merge the fix back to `develop`:
   ```bash
   git checkout develop
   git merge hotfix/critical-fix
   ```

## Release Checklist

Before merging `develop` → `main`:

- [ ] All tests pass (`cargo test`)
- [ ] CHANGELOGs are up to date (release-plz will append)
- [ ] No `[skip release]` commits that should release
- [ ] Version bump level is expected (check conventional commits)

## Troubleshooting

### "No version bump detected"

Ensure your commits follow [Conventional Commits](https://www.conventionalcommits.org/). Only `fix:`, `feat:`, and breaking changes trigger bumps.

### "semver check failed"

release-plz runs `cargo-semver-checks` to detect API-breaking changes. If it reports a mismatch:

1. Review the breaking changes
2. If intentional, use a `feat!:` or `BREAKING CHANGE:` commit
3. If unintentional, fix the API and retry

### "Workflow didn't trigger"

Check that:
- The push was to `main` (not another branch)
- The workflow file exists at `.github/workflows/release.yml`
- GitHub Actions is enabled in repository settings

## Configuration

Release behavior is configured in:

- `release.toml` — release-plz configuration
- `.github/workflows/release.yml` — GitHub Actions workflow
- Root `Cargo.toml` — workspace version inheritance

## No crates.io Publishing

All packages are internal tools and are **not published** to crates.io. The `publish = false` setting in `release.toml` ensures releases only create git tags and GitHub Releases.
