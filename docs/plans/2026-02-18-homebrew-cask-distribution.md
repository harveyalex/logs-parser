# Homebrew Cask Distribution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `logs-parser-desktop` installable on macOS via `brew install --cask logs-parser`.

**Architecture:** `dx bundle` produces a macOS `.app` from `Dioxus.toml` metadata. A GitHub Actions workflow triggers on `v*` tags, packages the `.app` into a `.dmg`, and uploads it to a GitHub Release. A separate `homebrew-tap` repo holds the Cask formula pointing to the release DMG.

**Tech Stack:** Rust, Dioxus CLI (`dx`), GitHub Actions, Homebrew Cask, `hdiutil` (macOS built-in), `softprops/action-gh-release`

---

### Task 1: Install dioxus-cli and add Dioxus.toml

**Files:**
- Create: `Dioxus.toml`

**Step 1: Install the Dioxus CLI**

```bash
cargo install dioxus-cli
```

Wait for it to finish (takes a few minutes). Verify:

```bash
dx --version
```

Expected: prints a version string like `dx 0.6.x`.

**Step 2: Create `Dioxus.toml` at the project root**

```toml
[application]
name = "logs-parser"
default_platform = "desktop"

[bundle]
identifier = "com.harveyalex.logs-parser"
name = "Logs Parser"
short_description = "Stream and filter Heroku logs"
```

**Step 3: Run `dx bundle` locally to verify output**

```bash
dx bundle --platform desktop --release
```

Expected: builds successfully and produces a `.app` somewhere under `dist/`. Note the exact path — you'll need it in the next task.

```bash
find dist -name "*.app"
```

Expected output: something like `dist/bundle/macos/Logs Parser.app`.

**Step 4: Commit**

```bash
git add Dioxus.toml
git commit -m "feat: add Dioxus.toml for app bundle configuration"
```

---

### Task 2: Create GitHub Actions release workflow

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Create the workflow directory**

```bash
mkdir -p .github/workflows
```

**Step 2: Create `.github/workflows/release.yml`**

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: macos-latest
    permissions:
      contents: write

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install dioxus-cli
        run: cargo install dioxus-cli

      - name: Build app bundle
        run: dx bundle --platform desktop --release

      - name: Find .app path
        id: app_path
        run: |
          APP=$(find dist -name "*.app" | head -1)
          echo "path=$APP" >> "$GITHUB_OUTPUT"

      - name: Create DMG
        run: |
          hdiutil create \
            -volname "Logs Parser" \
            -srcfolder "${{ steps.app_path.outputs.path }}" \
            -ov \
            -format UDZO \
            logs-parser.dmg

      - name: Compute SHA256
        run: shasum -a 256 logs-parser.dmg

      - name: Upload to GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: logs-parser.dmg
```

**Step 3: Commit and push**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add GitHub Actions release workflow"
git push
```

---

### Task 3: Create the homebrew-tap repo

This task is done in the browser and terminal, not in the source repo.

**Step 1: Create the repo on GitHub**

Go to https://github.com/new and create a **public** repo named exactly `homebrew-tap` under `harveyalex`. No README needed.

**Step 2: Clone it and add the Cask skeleton**

```bash
cd ~/development
git clone https://github.com/harveyalex/homebrew-tap.git
cd homebrew-tap
mkdir Casks
```

**Step 3: Create `Casks/logs-parser.rb`**

Use a placeholder SHA256 for now (we'll update it after the first real release in Task 4):

```ruby
cask "logs-parser" do
  version "0.1.0"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"

  url "https://github.com/harveyalex/logs-parser/releases/download/v#{version}/logs-parser.dmg"
  name "Logs Parser"
  desc "Stream and filter Heroku logs"
  homepage "https://github.com/harveyalex/logs-parser"

  app "Logs Parser.app"
end
```

**Step 4: Commit and push**

```bash
git add Casks/logs-parser.rb
git commit -m "feat: add logs-parser cask"
git push
```

---

### Task 4: Cut the v0.1.0 release and update the Cask

**Step 1: Tag and push from the source repo**

```bash
cd /Users/alexharvey/development/logs-parser
git tag v0.1.0
git push origin v0.1.0
```

**Step 2: Watch the CI run**

Go to https://github.com/harveyalex/logs-parser/actions and watch the `Release` workflow. It will take ~10 minutes (most of it is `cargo install dioxus-cli`).

Expected: workflow completes green, and a release appears at https://github.com/harveyalex/logs-parser/releases with `logs-parser.dmg` attached.

**Step 3: Copy the SHA256 from the workflow logs**

In the `Compute SHA256` step of the workflow log, you'll see a line like:

```
abc123...  logs-parser.dmg
```

Copy the 64-character hash.

**Step 4: Update the Cask with the real SHA256**

In `~/development/homebrew-tap/Casks/logs-parser.rb`, replace the placeholder:

```ruby
sha256 "abc123..."  # paste your actual hash here
```

**Step 5: Commit and push the tap**

```bash
cd ~/development/homebrew-tap
git add Casks/logs-parser.rb
git commit -m "release: v0.1.0"
git push
```

---

### Task 5: Verify the installation works

**Step 1: Add the tap**

```bash
brew tap harveyalex/tap
```

Expected: `Tapped 1 cask (N files, N KB).`

**Step 2: Install the cask**

```bash
brew install --cask logs-parser
```

Expected: Homebrew downloads the DMG, mounts it, copies `Logs Parser.app` to `/Applications`, and reports success.

**Step 3: Open the app**

```bash
open /Applications/Logs\ Parser.app
```

Expected: The Logs Parser window opens.

**Step 4: Verify `brew info` works**

```bash
brew info --cask logs-parser
```

Expected: Shows name, version, homepage, and install path.
