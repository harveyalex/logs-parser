# Versioning + Homebrew Automation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bump the app to v0.2.0 and automate Homebrew tap updates so that pushing a git tag fully releases the app.

**Architecture:** Three changes — bump `Cargo.toml` version, update the GitHub Actions release workflow to capture the DMG SHA256 and push it to the Homebrew tap via the GitHub API, and tag the release. The tap update uses `gh api` with a `HOMEBREW_TAP_TOKEN` secret that must be added manually once.

**Tech Stack:** Rust/Cargo, GitHub Actions, GitHub API (`gh` CLI), Homebrew cask formula (Ruby DSL)

---

### Task 1: Bump version to 0.2.0

**Files:**
- Modify: `Cargo.toml:3`

**Step 1: Update the version field**

In `Cargo.toml`, find:
```toml
version = "0.1.4"
```

Replace with:
```toml
version = "0.2.0"
```

**Step 2: Verify Cargo.lock updates**

```bash
cargo build --bin logs-parser-desktop 2>&1 | tail -3
```

Expected: builds cleanly, no errors.

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"
```

---

### Task 2: Update release workflow for automated tap updates

**Files:**
- Modify: `.github/workflows/release.yml`

**Context:** The current workflow computes SHA256 with `shasum -a 256 logs-parser.dmg` but just prints it — the value is never captured or used. We need to:
1. Capture the SHA256 as a step output
2. Add a new step that uses the GitHub API to update `Casks/logs-parser.rb` in `harveyalex/homebrew-tap`

The `harveyalex/homebrew-tap` repo already has `Casks/logs-parser.rb` with this structure:
```ruby
cask "logs-parser" do
  version "0.1.3"
  sha256 "c613909b738f985a8330fa76d514593a02cc3c3e6c64727556f8b38f2e5279de"
  ...
end
```

The update strategy: fetch the current file content via the API, sed-replace the `version` and `sha256` lines, base64-encode the result, and PUT it back.

**Step 1: Replace the existing SHA256 step**

Find in `.github/workflows/release.yml`:
```yaml
      - name: Compute SHA256
        run: shasum -a 256 logs-parser.dmg
```

Replace with:
```yaml
      - name: Compute SHA256
        id: sha
        run: |
          SHA=$(shasum -a 256 logs-parser.dmg | awk '{print $1}')
          echo "sha256=$SHA" >> "$GITHUB_OUTPUT"
```

**Step 2: Add the tap update step after the GitHub Release upload step**

The current last step is:
```yaml
      - name: Upload to GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: logs-parser.dmg
```

Add this immediately after it:
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

**Step 3: Verify the workflow file is valid YAML**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" && echo "YAML valid"
```

Expected: `YAML valid`

**Step 4: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: automate Homebrew tap update on release"
```

---

### Task 3: Add HOMEBREW_TAP_TOKEN secret (manual step)

**Files:** None — this is done in the GitHub web UI.

**Context:** The release workflow's new step uses `${{ secrets.HOMEBREW_TAP_TOKEN }}` to authenticate with the GitHub API and write to `harveyalex/homebrew-tap`. This secret must be created once.

**Step 1: Create a GitHub Personal Access Token**

1. Go to https://github.com/settings/tokens/new (classic token)
2. Note: `HOMEBREW_TAP_TOKEN for logs-parser releases`
3. Expiration: No expiration (or set a long one)
4. Scopes: check `repo` (full control of private repositories — needed to push to the tap)
5. Click **Generate token** and copy the value

**Step 2: Add the secret to the logs-parser repo**

1. Go to https://github.com/harveyalex/logs-parser/settings/secrets/actions
2. Click **New repository secret**
3. Name: `HOMEBREW_TAP_TOKEN`
4. Value: paste the token from step 1
5. Click **Add secret**

**Step 3: Verify the secret exists**

```bash
gh secret list --repo harveyalex/logs-parser
```

Expected: `HOMEBREW_TAP_TOKEN` appears in the list.

---

### Task 4: Tag and release v0.2.0

**Step 1: Push master to origin**

```bash
git push origin master
```

**Step 2: Tag and push**

```bash
git tag v0.2.0
git push origin v0.2.0
```

**Step 3: Watch the workflow run**

```bash
gh run watch --repo harveyalex/logs-parser
```

Or open: https://github.com/harveyalex/logs-parser/actions

Expected: the `Release` workflow runs to completion with all steps green.

**Step 4: Verify the GitHub Release**

```bash
gh release view v0.2.0 --repo harveyalex/logs-parser
```

Expected: release exists with `logs-parser.dmg` attached.

**Step 5: Verify the tap was updated**

```bash
gh api repos/harveyalex/homebrew-tap/contents/Casks/logs-parser.rb \
  --jq '.content' | base64 -d | grep -E "version|sha256"
```

Expected:
```
  version "0.2.0"
  sha256 "<some sha256 hash>"
```

**Step 6: Test the install locally**

```bash
brew update
brew upgrade --cask logs-parser
```

Or on a fresh machine:
```bash
brew install --cask harveyalex/tap/logs-parser
```
