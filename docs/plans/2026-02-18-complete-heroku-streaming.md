# Complete Heroku Streaming Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Test, document, and merge the `heroku-streaming` branch (which has all streaming code) into `master`.

**Architecture:** The `heroku-streaming` worktree at `.worktrees/heroku-streaming` has all code changes already committed through Task 7 of the original streaming plan. Tasks 8 (testing) and 9 (docs/cleanup) remain, then merge.

**Tech Stack:** Rust, Tokio, Dioxus desktop, Heroku CLI

---

## Context: What's Already Done

The `heroku-streaming` branch has these commits already:
- `feat(desktop): add Heroku CLI wrapper module` — `src/desktop/heroku_cli.rs`
- `feat(desktop): add StreamManager for process lifecycle` — `src/desktop/stream_manager.rs`
- `feat(desktop): add ConnectionPanel component`
- `feat(desktop): add StatusIndicator component`
- `refactor(desktop): simplify StatsHeader for streaming`
- `feat(desktop): integrate Heroku streaming into main app`
- `fix(desktop): resolve memory and task leaks in streaming`
- `refactor(desktop): remove file picker and export features`

**Working directory for all tasks below:** `.worktrees/heroku-streaming`

---

## Task 1: Verify the Build Compiles

**Files:**
- Read: `logs-parser/src/desktop/main.rs` (sanity check)

**Step 1: Build the desktop binary**

```bash
cd /Users/alexharvey/development/logs-parser/.worktrees/heroku-streaming/logs-parser
cargo build --bin logs-parser-desktop
```

Expected: Compiles successfully with no errors

**Step 2: Run the existing test suite**

```bash
cargo test
```

Expected: All tests pass (42+ tests)

**Step 3: If build fails, fix compilation errors**

Look at each error message and fix. Common issues:
- Unused import warnings (add `#[allow(unused)]` or remove)
- Lifetime/borrow issues in async closures

**Step 4: Commit if any fixes were needed**

```bash
git add -p
git commit -m "fix(desktop): resolve compilation errors in streaming integration"
```

---

## Task 2: Test No Heroku CLI Error Path

**Files:**
- None (manual test)

**Step 1: Run app without Heroku CLI on PATH**

```bash
cd /Users/alexharvey/development/logs-parser/.worktrees/heroku-streaming/logs-parser
PATH="" cargo run --bin logs-parser-desktop
```

Expected: App launches and shows error message "Heroku CLI not found. Install from heroku.com/cli" in the status bar

**Step 2: Verify error message is readable**

The status indicator should show the error in red. If the error message doesn't display, check `src/desktop/main.rs` and the `use_effect` that calls `check_cli_installed`.

**Step 3: Close the app**

---

## Task 3: Test Happy Path (Heroku Connected)

**Files:**
- None (manual test — requires Heroku CLI installed and authenticated)

**Prerequisite check:**

```bash
heroku auth:whoami
```

If not authenticated, run `heroku login` first.

**Step 1: Run the streaming app**

```bash
cd /Users/alexharvey/development/logs-parser/.worktrees/heroku-streaming/logs-parser
cargo run --bin logs-parser-desktop
```

**Step 2: Verify startup flow**

Check each state in order:
1. Status shows "Loading..." (orange dot)
2. Apps populate in dropdown
3. Status shows "Ready to connect" (grey dot)

If any step fails, check `src/desktop/main.rs` in the `use_effect` initialization block.

**Step 3: Test connection**

1. Select an app from the dropdown
2. Click "Connect"
3. Status changes to "Connecting..." then "Streaming" (green dot)
4. Logs appear in the log view

**Step 4: Test filtering while streaming**

1. Add a filter (e.g., type "error" in the filter bar, press Enter)
2. Verify only matching logs show
3. Toggle AND/OR mode button
4. Click "Clear" to remove filters
5. Verify all logs return

**Step 5: Test disconnect**

1. Click "Disconnect"
2. Status returns to "Ready to connect"
3. Log view clears
4. Can select a different app

**Step 6: Verify no orphaned processes**

After disconnecting, check no `heroku logs` process remains:

```bash
ps aux | grep "heroku logs" | grep -v grep
```

Expected: No output (no orphaned processes)

**Step 7: Close the app and check again**

```bash
ps aux | grep "heroku logs" | grep -v grep
```

Expected: No output

---

## Task 4: Update README

**Files:**
- Modify: `logs-parser/README.md`

**Step 1: Read the current README**

Read the Desktop App section — it currently describes the file picker workflow. We need to replace it with streaming documentation.

**Step 2: Replace the Desktop App section**

Find and replace the "### Desktop App (GUI)" section with:

```markdown
### Desktop App (GUI)

Run the desktop application:

```bash
cargo run --release --bin logs-parser-desktop
```

**Prerequisites:**
- Heroku CLI installed (`brew install heroku/brew/heroku` or see heroku.com/cli)
- Authenticated (`heroku login`)
- Access to at least one Heroku app

**Features:**
- Live Heroku log streaming via Heroku CLI
- App selection from authenticated account
- Real-time filtering with text search, regex, and field filters
- AND/OR filter logic toggle
- Color-coded log levels
- Auto-reconnection on failures (exponential backoff, 5 attempts)
- Error messages for common setup issues (CLI missing, not authenticated)

**Connection Flow:**
1. App checks for Heroku CLI and authentication on startup
2. Select app from dropdown
3. Click "Connect" to start streaming
4. Logs stream in real-time
5. Apply filters to focus on specific logs
6. Click "Disconnect" to stop streaming

**Keyboard Shortcuts:**
- `C` - Clear all filters
- `Ctrl+Q` - Quit application
```

**Step 3: Update the "Known Issues" and "Development Status" sections**

Remove file picker references. Update Development Status to reflect streaming is complete.

**Step 4: Run cargo fmt and clippy**

```bash
cd logs-parser
cargo fmt
cargo clippy --bin logs-parser-desktop -- -D warnings
```

Fix any clippy warnings.

**Step 5: Commit**

```bash
git add logs-parser/README.md
git commit -m "docs: update README for streaming-only desktop app

- Document Heroku CLI prerequisites
- Describe connection flow
- List streaming features
- Remove file picker documentation"
```

---

## Task 5: Merge to Master

**Files:**
- None (git operations)

**Step 1: Verify the heroku-streaming branch is clean**

```bash
cd /Users/alexharvey/development/logs-parser/.worktrees/heroku-streaming
git status
```

Expected: `nothing to commit, working tree clean`

**Step 2: Check what commits will be merged**

```bash
git log master..HEAD --oneline
```

Expected: Should show the 8 streaming feature commits.

**Step 3: Switch to master and merge**

```bash
cd /Users/alexharvey/development/logs-parser
git merge heroku-streaming --no-ff -m "feat(desktop): integrate live Heroku log streaming

Replaces file-picker desktop app with live streaming from Heroku CLI:

- Add heroku_cli module (check CLI, auth, fetch apps)
- Add StreamManager with auto-reconnect (exponential backoff)
- Add ConnectionPanel and StatusIndicator components
- Integrate streaming into main App component
- Remove file picker and export features

Streaming architecture:
- Spawns 'heroku logs --tail' via tokio::process
- Reads stdout line-by-line via async BufReader
- Sends LogEntry structs via unbounded mpsc channel
- Reconnects automatically on process failure"
```

**Step 4: Verify tests still pass on master**

```bash
cargo test
```

Expected: All tests pass

**Step 5: Verify desktop binary builds**

```bash
cargo build --release --bin logs-parser-desktop
```

Expected: Successful build

**Step 6: Remove worktree (optional — ask user first)**

If everything looks good:

```bash
git worktree remove .worktrees/heroku-streaming
git branch -d heroku-streaming
```

---

## Success Criteria Checklist

- [ ] `cargo build --bin logs-parser-desktop` succeeds in worktree
- [ ] All 42+ unit tests pass
- [ ] App shows error when Heroku CLI is missing
- [ ] App shows apps list when CLI is installed and authenticated
- [ ] Logs stream in real-time after connecting
- [ ] Filters work on streamed logs
- [ ] No orphaned `heroku logs` processes after disconnect/exit
- [ ] README updated with streaming docs
- [ ] Branch merged to master
- [ ] Tests pass on master post-merge
