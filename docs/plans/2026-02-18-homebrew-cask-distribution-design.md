# Homebrew Cask Distribution Design

**Date:** 2026-02-18
**Status:** Approved

## Overview

Distribute `logs-parser-desktop` as a macOS `.app` bundle installable via:

```
brew tap harveyalex/tap
brew install --cask logs-parser
```

## Architecture

Three pieces work together:

1. **`harveyalex/logs-parser`** — source repo. A GitHub Actions workflow triggers on `v*` tags, builds the `.app` with `dx bundle`, wraps it in a `.dmg`, and uploads it as a GitHub Release asset.

2. **`harveyalex/homebrew-tap`** — new public repo (must be named `homebrew-tap` for Homebrew's tap convention). Contains a single Cask file at `Casks/logs-parser.rb` pointing to the release DMG.

3. **GitHub Releases** — bridge between the two. DMG lives at a stable versioned URL:
   `https://github.com/harveyalex/logs-parser/releases/download/v{version}/logs-parser.dmg`

## Dioxus.toml

Add a `Dioxus.toml` at the project root with bundle metadata required by `dx bundle`:

```toml
[bundle]
identifier = "com.harveyalex.logs-parser"
name = "Logs Parser"
short_description = "Stream and filter Heroku logs"
```

No icon for now — will use the default Dioxus icon.

## GitHub Actions Workflow

File: `.github/workflows/release.yml`

Trigger: `push` to tags matching `v*`
Runner: `macos-latest`

Steps:
1. Checkout code
2. Install Rust (stable)
3. Install `dx` CLI via `cargo install dioxus-cli`
4. Run `dx bundle --platform desktop --release`
5. Create `.dmg` from the `.app` output using `hdiutil` (built into macOS runners)
6. Compute SHA256 of the DMG and print to logs
7. Create GitHub Release and upload DMG via `softprops/action-gh-release`

No extra secrets needed — uses the built-in `GITHUB_TOKEN`.

## Homebrew Tap

New repo: `harveyalex/homebrew-tap`

File: `Casks/logs-parser.rb`

```ruby
cask "logs-parser" do
  version "0.2.0"
  sha256 "<sha256-of-dmg>"

  url "https://github.com/harveyalex/logs-parser/releases/download/v#{version}/logs-parser.dmg"
  name "Logs Parser"
  desc "Stream and filter Heroku logs"
  homepage "https://github.com/harveyalex/logs-parser"

  app "Logs Parser.app"
end
```

## Release Workflow

To cut a new release:
1. Bump `version` in `Cargo.toml`
2. Commit and push
3. Push a `v*` tag: `git tag v0.2.0 && git push origin v0.2.0`
4. CI builds and uploads the DMG automatically
5. Copy the SHA256 from the workflow logs
6. Update `version` and `sha256` in `Casks/logs-parser.rb` in the tap repo

## Out of Scope

- Code signing / notarization (can be added later for Gatekeeper compatibility)
- Automated Cask version bumps
- App icon
