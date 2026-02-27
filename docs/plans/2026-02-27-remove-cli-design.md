# Design: Remove CLI Artifacts, Flatten Crate, Refresh Docs

**Date:** 2026-02-27
**Status:** Approved

## Context

The project started as a TUI (terminal) CLI application where users would pipe Heroku logs into it
(`heroku logs --tail --app my-app | logs-parser`). It has since been fully converted to a Dioxus
desktop GUI application. The Heroku CLI is still used internally to stream logs — this is
intentional and stays. What this design removes is everything that made logs-parser itself a CLI
tool.

## What Gets Removed

### Dead source files
- `src/lib.rs` — lib crate entry point; the desktop binary will own `parser` and `filters` directly
- `src/desktop/state.rs` — dead code that references the removed `CircularBuffer`
- `src/desktop/mod.rs` — orphaned module file not used by the binary or lib

### CLI/TUI-era artifacts
- `examples/simple_test.rs` + `examples/` directory — crossterm terminal test
- `output.log` — debug artifact
- `test_logs.sh`, `test_terminal.sh`, `try_terminal.sh`, `debug_run.sh`, `logs-parser-wrapper.sh`, `run_desktop.sh` — shell scripts for the old piping workflow
- `QUICKSTART.md` — describes TUI piping usage
- `IMPLEMENTATION_SUMMARY.md` — describes Ratatui/TEA architecture

## Code Changes

### Move parser + filters into the desktop module

- `src/filters.rs` → `src/desktop/filters.rs`
- `src/parser.rs` → `src/desktop/parser.rs`
- Add `mod filters; mod parser;` to `src/desktop/main.rs`
- Remove the stale doctest in `parser.rs` that references `use logs_parser::parser::parse_log_line`

### Update 6 import sites

All `use logs_parser::...` references become crate-local:

| File | Old | New |
|------|-----|-----|
| `desktop/main.rs` | `logs_parser::filters::Filter` | `filters::Filter` |
| `desktop/main.rs` | `logs_parser::parser::{LogEntry, LogLevel}` | `parser::{LogEntry, LogLevel}` |
| `desktop/stream_manager.rs` | `logs_parser::parser::parse_log_line` | `crate::parser::parse_log_line` |
| `desktop/stream_manager.rs` | `logs_parser::parser::LogEntry` | `crate::parser::LogEntry` |
| `desktop/components/filter_bar.rs` | `logs_parser::filters::Filter` | `crate::filters::Filter` |
| `desktop/components/log_view.rs` | `logs_parser::parser::{LogEntry, LogLevel}` | `crate::parser::{LogEntry, LogLevel}` |

`Cargo.toml` requires no changes — Cargo auto-detects the lib crate from `src/lib.rs`; deleting
that file removes the lib crate automatically.

### Update documentation

- `README.md` — rewrite to cover only the desktop app; remove TUI section, piping examples,
  Ratatui architecture diagram, TUI keyboard shortcuts
- New `QUICKSTART.md` — desktop-focused quickstart covering: install Heroku CLI, build/run the app,
  connect to an app, and use filters

## Final File Structure

```
src/
└── desktop/
    ├── main.rs             (binary entry point, declares all modules)
    ├── filters.rs          (moved from src/filters.rs)
    ├── parser.rs           (moved from src/parser.rs)
    ├── heroku_cli.rs       (unchanged)
    ├── stream_manager.rs   (updated imports)
    └── components/
        ├── mod.rs
        ├── connection_panel.rs
        ├── custom_select.rs
        ├── filter_bar.rs   (updated import)
        ├── log_view.rs     (updated import)
        ├── stats_header.rs
        └── status_indicator.rs
```

## Testing

Run `cargo test` after each step to confirm the 51-test suite stays green throughout.
