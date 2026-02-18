# Quick Start Guide

## Installation & Build

```bash
cd logs-parser
cargo build --release
```

## Quick Test

```bash
# Test with sample logs
./test_logs.sh | cargo run --release
```

## With Real Heroku Logs

```bash
# Replace YOUR_APP with your Heroku app name
heroku logs --tail --app YOUR_APP | cargo run --release
```

## Essential Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `p` / `Space` | Pause/Resume |
| `/` | Search (enter query, press Enter) |
| `f` | Toggle AND/OR filter mode |
| `c` | Clear all filters |
| `1` / `2` / `3` | Switch view modes |
| `Ctrl+C` | Copy filtered logs to clipboard |
| `Ctrl+S` | Save filtered logs to file |
| `q` / `Esc` | Quit |

## Filtering Examples

### Search for errors:
1. Press `/`
2. Type `error`
3. Press `Enter`

### Search for specific dyno:
1. Press `/`
2. Type `web.1`
3. Press `Enter`

### Multiple filters with AND logic:
1. Add first filter: `/` → `error` → `Enter`
2. Add second filter: `/` → `app` → `Enter`
3. Both conditions must match

### Multiple filters with OR logic:
1. Add filters as above
2. Press `f` to toggle to OR mode
3. Either condition can match

## View Modes

- **List (1)**: Scrollable list of all logs
- **Detail (2)**: Full detail view of selected log
- **Split (3)**: List + detail side-by-side

## Status Display

The header shows:
- Total logs / Filtered logs
- Number of active filters
- Filter mode (AND/OR)
- Current view mode
- [PAUSED] indicator when streaming is paused

## Tips

- **Pause before scrolling**: Press `p` to pause, then scroll through logs freely
- **Clear filters to see all**: Press `c` to remove all filters
- **Watch for colors**: Red=Error, Yellow=Warning, Green=Info, Blue=Debug
- **Use the buffer**: The app stores the last 10,000 logs automatically

## Troubleshooting

**Problem**: "Device not configured" error
**Solution**: Make sure you're running in a terminal, not just piping input

**Problem**: No logs appearing
**Solution**: Check that your input matches Heroku log format:
`YYYY-MM-DDTHH:MM:SS.mmmmmm+00:00 source[dyno]: message`

**Problem**: Can't see all logs
**Solution**: The buffer holds 10,000 logs. Older logs are automatically removed.
