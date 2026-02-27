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
