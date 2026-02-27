# Versioning + Homebrew Automation Design

**Date:** 2026-02-27
**Status:** Approved

## Context

The release workflow at `.github/workflows/release.yml` already:
- Triggers on `v*` tags
- Builds a macOS DMG via `dx bundle --platform desktop --release`
- Computes SHA256 (but discards it)
- Uploads `logs-parser.dmg` to a GitHub Release

The Homebrew tap (`harveyalex/homebrew-tap`) already exists with `Casks/logs-parser.rb`, but it's on `0.1.3` while the app is at `0.1.4` — the tap update was never automated.

## Changes

### 1. Version bump

`Cargo.toml`: `0.1.4` → `0.2.0`

The CLI-removal refactor is significant enough to warrant a minor version bump.

### 2. Release workflow: wire SHA256 and add tap update step

Replace the current `shasum` step (which just prints) with one that captures the SHA256 as a step output, then add a new step that uses the GitHub API to update the cask formula.

**Updated SHA256 step:**
```yaml
- name: Compute SHA256
  id: sha
  run: |
    SHA=$(shasum -a 256 logs-parser.dmg | awk '{print $1}')
    echo "sha256=$SHA" >> "$GITHUB_OUTPUT"
```

**New tap update step:**
```yaml
- name: Update Homebrew tap
  env:
    GH_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
  run: |
    VERSION="${GITHUB_REF_NAME#v}"
    FILE_SHA=$(gh api repos/harveyalex/homebrew-tap/contents/Casks/logs-parser.rb --jq '.sha')
    CURRENT=$(gh api repos/harveyalex/homebrew-tap/contents/Casks/logs-parser.rb --jq '.content' | base64 -d)
    UPDATED=$(echo "$CURRENT" \
      | sed "s/  version \".*\"/  version \"$VERSION\"/" \
      | sed "s/  sha256 \".*\"/  sha256 \"${{ steps.sha.outputs.sha256 }}\"/" )
    CONTENT=$(echo "$UPDATED" | base64 | tr -d '\n')
    gh api repos/harveyalex/homebrew-tap/contents/Casks/logs-parser.rb \
      --method PUT \
      -f message="chore: update logs-parser to $VERSION" \
      -f content="$CONTENT" \
      -f sha="$FILE_SHA"
```

### 3. One-time manual step (not automated)

Add a `HOMEBREW_TAP_TOKEN` secret to the `harveyalex/logs-parser` GitHub repo:
- Go to repo Settings → Secrets and variables → Actions → New repository secret
- Name: `HOMEBREW_TAP_TOKEN`
- Value: a GitHub PAT with `repo` scope on `harveyalex/homebrew-tap`

## Release process going forward

```bash
# 1. Bump version in Cargo.toml
# 2. Commit
git commit -m "chore: bump version to 0.2.0"
# 3. Tag and push
git tag v0.2.0
git push origin master --tags
```

GitHub Actions then handles everything: build → DMG → GitHub Release → tap update.

Users can install or upgrade with:
```bash
brew install --cask harveyalex/tap/logs-parser
# or
brew upgrade --cask logs-parser
```
