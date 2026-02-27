# Remove CLI Artifacts Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove all TUI/CLI-era artifacts, flatten `parser` and `filters` into the desktop module, and rewrite docs for the desktop app.

**Architecture:** The desktop binary at `src/desktop/main.rs` currently imports `parser` and `filters` from a separate lib crate (`src/lib.rs`). We collapse that lib crate by moving those two files into `src/desktop/`, updating 6 import sites, and deleting everything that belonged to the old TUI piping workflow.

**Tech Stack:** Rust, Cargo, Dioxus 0.6 desktop

---

### Task 1: Establish green baseline

**Files:**
- Read: none (just run tests)

**Step 1: Confirm 51 tests pass before touching anything**

```bash
cargo test 2>&1 | tail -5
```

Expected output (exact numbers may vary slightly):
```
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

If any tests fail, stop and investigate before continuing.

---

### Task 2: Delete dead source files

**Files:**
- Delete: `src/desktop/state.rs`
- Delete: `src/desktop/mod.rs`

**Context:** `state.rs` references a `CircularBuffer` type that was removed long ago — it's never compiled because `main.rs` doesn't declare `mod state`. `mod.rs` is the module file for when the desktop dir was treated as a library module; again, nothing uses it.

**Step 1: Delete both files**

```bash
rm src/desktop/state.rs src/desktop/mod.rs
```

**Step 2: Verify the build still passes**

```bash
cargo test 2>&1 | tail -5
```

Expected: same 51 tests passing.

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: remove dead desktop/state.rs and desktop/mod.rs"
```

---

### Task 3: Delete shell scripts, examples, and artifact files

**Files:**
- Delete: `test_logs.sh`
- Delete: `test_terminal.sh`
- Delete: `try_terminal.sh`
- Delete: `debug_run.sh`
- Delete: `logs-parser-wrapper.sh`
- Delete: `run_desktop.sh`
- Delete: `output.log`
- Delete: `examples/simple_test.rs` (and the `examples/` dir)

**Context:** These are all leftovers from the TUI piping workflow (`heroku logs --tail | logs-parser`). None affect compilation.

**Step 1: Delete everything**

```bash
rm test_logs.sh test_terminal.sh try_terminal.sh debug_run.sh logs-parser-wrapper.sh run_desktop.sh output.log
rm -rf examples/
```

**Step 2: Verify build still passes**

```bash
cargo test 2>&1 | tail -5
```

Expected: same 51 tests passing.

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: remove TUI-era shell scripts and examples"
```

---

### Task 4: Move parser.rs into the desktop module

**Files:**
- Create: `src/desktop/parser.rs` (moved from `src/parser.rs`)
- Modify: `src/desktop/main.rs` — add `mod parser;`, update imports
- Modify: `src/desktop/stream_manager.rs` — update imports
- Modify: `src/desktop/components/log_view.rs` — update import
- Delete: `src/parser.rs`

**Step 1: Copy parser.rs to the desktop directory**

```bash
cp src/parser.rs src/desktop/parser.rs
```

**Step 2: Remove the stale doctest from src/desktop/parser.rs**

Find this block in `src/desktop/parser.rs` (around line 82) and remove the entire example block:

```rust
/// # Example
/// ```
/// use logs_parser::parser::parse_log_line;
///
/// let line = "2010-09-16T15:13:46.677020+00:00 app[web.1]: Starting process";
/// let entry = parse_log_line(line).expect("Failed to parse");
/// assert_eq!(entry.source, "app");
/// assert_eq!(entry.dyno, "web.1");
/// ```
```

Replace the entire doc comment on `parse_log_line` with just:

```rust
/// Parse a single Heroku log line
///
/// Returns `Some(LogEntry)` if the line matches the expected format,
/// or `None` if the line cannot be parsed.
pub fn parse_log_line(line: &str) -> Option<LogEntry> {
```

**Step 3: Add `mod parser;` to src/desktop/main.rs**

Find this block at the top of `src/desktop/main.rs`:

```rust
mod components;
mod heroku_cli;
mod stream_manager;
```

Replace with:

```rust
mod components;
mod filters;
mod heroku_cli;
mod parser;
mod stream_manager;
```

(Note: `mod filters;` is added here too — it won't exist yet, but we'll create it in Task 5. For now just add `mod parser;`.)

Actually, add only `mod parser;` for now:

```rust
mod components;
mod heroku_cli;
mod parser;
mod stream_manager;
```

**Step 4: Update imports in src/desktop/main.rs**

Find:
```rust
use logs_parser::parser::{LogEntry, LogLevel};
```

Replace with:
```rust
use parser::{LogEntry, LogLevel};
```

**Step 5: Update imports in src/desktop/stream_manager.rs**

Find:
```rust
use logs_parser::parser::parse_log_line;
use logs_parser::parser::LogEntry;
```

Replace with:
```rust
use crate::parser::parse_log_line;
use crate::parser::LogEntry;
```

**Step 6: Update import in src/desktop/components/log_view.rs**

Find:
```rust
use logs_parser::parser::{LogEntry, LogLevel};
```

Replace with:
```rust
use crate::parser::{LogEntry, LogLevel};
```

**Step 7: Verify it compiles and tests pass**

```bash
cargo test 2>&1 | tail -5
```

Expected: same 51 tests passing. Fix any compile errors before continuing.

**Step 8: Delete the original src/parser.rs**

```bash
rm src/parser.rs
```

**Step 9: Verify again**

```bash
cargo test 2>&1 | tail -5
```

Expected: same 51 tests passing.

**Step 10: Commit**

```bash
git add -A
git commit -m "refactor: move parser.rs into desktop module"
```

---

### Task 5: Move filters.rs into the desktop module

**Files:**
- Create: `src/desktop/filters.rs` (moved from `src/filters.rs`)
- Modify: `src/desktop/main.rs` — add `mod filters;`, update import
- Modify: `src/desktop/components/filter_bar.rs` — update import
- Delete: `src/filters.rs`

**Step 1: Copy filters.rs to the desktop directory**

```bash
cp src/filters.rs src/desktop/filters.rs
```

**Step 2: Add `mod filters;` to src/desktop/main.rs**

Find:
```rust
mod components;
mod heroku_cli;
mod parser;
mod stream_manager;
```

Replace with:
```rust
mod components;
mod filters;
mod heroku_cli;
mod parser;
mod stream_manager;
```

**Step 3: Update import in src/desktop/main.rs**

Find:
```rust
use logs_parser::filters::Filter;
```

Replace with:
```rust
use filters::Filter;
```

**Step 4: Update import in src/desktop/components/filter_bar.rs**

Find:
```rust
use logs_parser::filters::Filter;
```

Replace with:
```rust
use crate::filters::Filter;
```

**Step 5: Verify it compiles and tests pass**

```bash
cargo test 2>&1 | tail -5
```

Expected: same 51 tests passing. The unit tests inside `filters.rs` use `use crate::parser::{LogEntry, LogLevel}` — since `parser` is now a sibling module declared in `main.rs`, `crate::parser` resolves correctly.

**Step 6: Delete the original src/filters.rs**

```bash
rm src/filters.rs
```

**Step 7: Verify again**

```bash
cargo test 2>&1 | tail -5
```

Expected: same 51 tests passing.

**Step 8: Commit**

```bash
git add -A
git commit -m "refactor: move filters.rs into desktop module"
```

---

### Task 6: Delete src/lib.rs

**Files:**
- Delete: `src/lib.rs`

**Context:** `src/lib.rs` only declared `pub mod filters;` and `pub mod parser;`. Both modules are now owned by the desktop binary. Deleting this file removes the lib crate automatically — Cargo only builds a lib crate when `src/lib.rs` exists.

**Step 1: Delete lib.rs**

```bash
rm src/lib.rs
```

**Step 2: Verify build and tests pass**

```bash
cargo test 2>&1 | tail -5
```

Expected: same 51 tests passing.

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: remove lib.rs, logs-parser is now desktop-only"
```

---

### Task 7: Delete old CLI/TUI documentation

**Files:**
- Delete: `QUICKSTART.md`
- Delete: `IMPLEMENTATION_SUMMARY.md`

**Step 1: Delete both files**

```bash
rm QUICKSTART.md IMPLEMENTATION_SUMMARY.md
```

**Step 2: Commit**

```bash
git add -A
git commit -m "docs: remove TUI-era QUICKSTART and IMPLEMENTATION_SUMMARY"
```

---

### Task 8: Rewrite README.md

**Files:**
- Modify: `README.md`

**Step 1: Replace the entire contents of README.md with the following**

```markdown
# Heroku Logs Parser

A desktop app for streaming and filtering Heroku logs in real time.

## Installation

### Homebrew (recommended)

```bash
brew install --cask harveyalex/tap/logs-parser
```

### Download DMG

Download the latest `logs-parser.dmg` from the [GitHub Releases](https://github.com/harveyalex/logs-parser/releases) page, open it, and drag the app to your Applications folder.

### Build from source

Requires [Rust](https://rustup.rs) and the [Heroku CLI](https://devcenter.heroku.com/articles/heroku-cli).

```bash
git clone https://github.com/harveyalex/logs-parser.git
cd logs-parser
cargo build --release --bin logs-parser-desktop
./target/release/logs-parser-desktop
```

## Prerequisites

- [Heroku CLI](https://devcenter.heroku.com/articles/heroku-cli) installed and authenticated
- Access to at least one Heroku app

## Usage

1. Launch the app — it checks for the Heroku CLI and your authentication automatically
2. Select an app from the dropdown
3. Click **Connect** to start streaming logs
4. Use the filter bar to narrow down what you see
5. Click **Disconnect** to stop

## Filtering

| Syntax | Example | Matches |
|--------|---------|---------|
| Plain text | `error` | Any log containing "error" (case-insensitive) |
| Regex | `/5\d\d/` | Any log matching the regex |
| Dyno | `dyno:web.1` | Logs from web.1 only |
| Source | `source:heroku` | Logs with source "heroku" |
| Level | `level:error` | Logs at error level |

Use the **AND/OR** toggle to control how multiple filters combine.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `C` | Clear all filters |
| `Ctrl+Q` | Quit |

## Architecture

```
src/desktop/
├── main.rs             Entry point, app component, state wiring
├── parser.rs           Heroku log format parsing (RFC5424)
├── filters.rs          Filter types and matching logic
├── heroku_cli.rs       Heroku CLI wrappers (auth, app list, login)
├── stream_manager.rs   heroku logs --tail process lifecycle
└── components/
    ├── connection_panel.rs
    ├── filter_bar.rs
    ├── log_view.rs
    ├── stats_header.rs
    ├── status_indicator.rs
    └── custom_select.rs
```

## Testing

```bash
cargo test
```

## License

MIT
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: rewrite README for desktop app"
```

---

### Task 9: Write new QUICKSTART.md

**Files:**
- Create: `QUICKSTART.md`

**Step 1: Create QUICKSTART.md with the following content**

```markdown
# Quick Start

## 1. Install the Heroku CLI

```bash
brew tap heroku/brew && brew install heroku
```

Or see [heroku.com/cli](https://devcenter.heroku.com/articles/heroku-cli) for other platforms.

## 2. Log in to Heroku

```bash
heroku login
```

## 3. Run the app

**From a release build:**

```bash
./target/release/logs-parser-desktop
```

**From source:**

```bash
cargo run --release --bin logs-parser-desktop
```

## 4. Connect to an app

1. The app fetches your Heroku apps on startup
2. Select an app from the dropdown
3. Click **Connect**
4. Logs stream in real time

## 5. Filter logs

Type a filter expression and press **Enter**:

- `error` — text search
- `dyno:web.1` — specific dyno
- `level:error` — log level
- `/pattern/` — regex

Press **C** to clear all filters.
```

**Step 2: Commit**

```bash
git add QUICKSTART.md
git commit -m "docs: add desktop-focused QUICKSTART"
```

---

### Task 10: Final verification

**Step 1: Confirm test suite is still fully green**

```bash
cargo test
```

Expected: all tests pass, 0 failures.

**Step 2: Confirm the binary builds cleanly with no warnings**

```bash
cargo build --release --bin logs-parser-desktop 2>&1 | grep -E "^error|^warning"
```

Expected: no `error:` lines. Address any warnings that refer to unused code introduced by the refactor.

**Step 3: Confirm no logs_parser:: references remain**

```bash
grep -r "logs_parser::" src/
```

Expected: no output.

**Step 4: Confirm no CLI-era files remain**

```bash
ls *.sh 2>/dev/null && echo "FOUND SHELL SCRIPTS" || echo "Clean"
ls examples/ 2>/dev/null && echo "FOUND EXAMPLES" || echo "Clean"
ls src/lib.rs 2>/dev/null && echo "FOUND lib.rs" || echo "Clean"
```

Expected: all three print "Clean".
