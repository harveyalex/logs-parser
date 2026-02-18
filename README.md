# Heroku Logs Parser TUI

A terminal UI application for parsing and filtering Heroku log streams with real-time analysis capabilities.

## Features

### Implemented ✅

**Phase 1: Core Parsing**
- ✅ Heroku log format parsing (RFC5424 timestamps, source, dyno, message)
- ✅ Log level detection (ERROR, WARN, INFO, DEBUG)
- ✅ Circular buffer for log storage (10,000 entry default)
- ✅ Comprehensive test coverage (42 passing tests)

**Phase 2: State Management**
- ✅ The Elm Architecture (TEA) pattern with Message-based updates
- ✅ Application state management with scroll position, filters, and view modes
- ✅ Pause/resume log streaming
- ✅ Filter state management with AND/OR logic

**Phase 3: Basic TUI**
- ✅ Terminal UI with ratatui
- ✅ Keyboard navigation (j/k, arrows, page up/down)
- ✅ Three view modes: List, Detail, Split
- ✅ Color-coded log levels (Red=Error, Yellow=Warn, Green=Info, Blue=Debug)
- ✅ Header with statistics
- ✅ Filter bar showing active filters
- ✅ Status bar with help text

**Phase 4: Filtering**
- ✅ Case-insensitive text search
- ✅ Regex pattern matching
- ✅ Field-specific filters (dyno, source, log level)
- ✅ Multiple simultaneous filters
- ✅ AND/OR filter logic
- ✅ Real-time filter application

**Phase 6: Async Event Loop**
- ✅ Tokio async runtime
- ✅ Separate tasks for stdin reading and keyboard events
- ✅ Event coordination with tokio::select!
- ✅ 100ms refresh interval

### Phase 5: Advanced Features (Partially Complete)
- ✅ Export filtered logs to file (Ctrl+S)
- ✅ Copy filtered logs to clipboard (Ctrl+C)
- ⏳ Time range filtering
- ⏳ Help screen (press '?')

**Phase 7: Polish**
- ⏳ CLI arguments for buffer size and initial filters
- ⏳ Performance optimization for high-throughput streams
- ⏳ Additional view options

## Installation

```bash
# Build the TUI version
cargo build --release --bin logs-parser-tui

# Build the desktop version
cargo build --release --bin logs-parser-desktop

# Run tests
cargo test
```

## Usage

### Desktop App (GUI)

Run the desktop application:

```bash
cargo run --release --bin logs-parser-desktop
```

**Features:**
- Native window interface
- File picker to open log files
- Real-time filtering with text search, regex, and field filters
- AND/OR filter logic toggle
- Color-coded log levels
- Export filtered logs to file
- Copy filtered logs to clipboard

**Keyboard Shortcuts:**
- `Ctrl+O` - Open log file
- `Ctrl+C` - Copy filtered logs to clipboard
- `Ctrl+S` - Export filtered logs to file
- `Ctrl+Q` - Quit application
- `C` - Clear all filters
- `P` or `Space` - Pause/resume (placeholder for live streaming)

### Terminal App (TUI)

#### Basic Usage

Pipe Heroku logs into the parser:

```bash
heroku logs --tail --app YOUR_APP | cargo run --release
```

Or use the test log generator:

```bash
./test_logs.sh | cargo run --release
```

### Keyboard Controls

**Navigation:**
- `j` or `↓` - Scroll down
- `k` or `↑` - Scroll up
- `PageUp` - Page up
- `PageDown` - Page down
- `Home` or `g` - Scroll to top
- `End` or `G` - Scroll to bottom

**Actions:**
- `p` or `Space` - Pause/resume log streaming
- `q` or `Esc` - Quit application

**Filtering:**
- `/` - Enter search mode (type your query, press Enter to apply)
- `f` - Toggle filter mode (AND/OR)
- `c` - Clear all filters

**View Modes:**
- `1` - List view (default)
- `2` - Detail view
- `3` - Split view (list + detail)

**Export & Clipboard:**
- `Ctrl+C` - Copy filtered logs to clipboard
- `Ctrl+S` - Save filtered logs to timestamped file

## Architecture

The application follows **The Elm Architecture (TEA)** pattern:

```
┌─────────────┐
│   Model     │  AppState with logs, filters, UI state
│  (AppState) │
└─────────────┘
       │
       ├──────────────┐
       ▼              ▼
┌─────────────┐  ┌──────────┐
│   Update    │  │   View   │
│  (Message)  │  │  (UI)    │
└─────────────┘  └──────────┘
       │              │
       └──────────────┘
```

### Module Structure

```
src/
├── main.rs      - Async runtime, event loop
├── app.rs       - AppState, Message enum, update logic
├── parser.rs    - Heroku log parsing with regex
├── filters.rs   - Filter engine and predicates
├── ui.rs        - Ratatui rendering
├── buffer.rs    - Circular buffer for log storage
├── events.rs    - Keyboard input handling
└── lib.rs       - Module exports
```

## Testing

Run the full test suite:

```bash
cargo test
```

Current test coverage: **42 tests, 100% passing**

Test with sample logs:

```bash
# Option 1: Single log line
echo "2010-09-16T15:13:46.677020+00:00 app[web.1]: Test message" | cargo run

# Option 2: Continuous stream with test script
./test_logs.sh | cargo run
```

## Examples

### Example 1: Basic Parsing and Display

```bash
./test_logs.sh | cargo run
```

This will show a live stream of logs with color-coded log levels.

### Example 2: Filtering Error Logs

1. Run the parser: `./test_logs.sh | cargo run`
2. Press `/` to enter search mode
3. Type `error` and press Enter
4. Only error logs will be displayed

### Example 3: Filtering by Dyno

1. Run the parser
2. Press `/`
3. Type `web.1` and press Enter
4. Only logs from the web.1 dyno will be displayed

### Example 4: Pause and Resume

1. Run the parser
2. Press `p` to pause - logs stop updating
3. Scroll through existing logs with arrow keys
4. Press `p` again to resume streaming

## Performance

- **Buffer Size:** 10,000 entries (configurable)
- **Memory:** Fixed memory footprint, oldest logs auto-removed (FIFO)
- **Refresh Rate:** 100ms (10 FPS)
- **Filtering:** Applied on display, all logs preserved in buffer

## Known Issues

1. **Time range filtering:** Not yet implemented (Phase 5)
2. **Help screen:** Not yet implemented (press '?' will be added)

## Development Status

**Current Phase:** Phase 5 (Partial) Complete ✅

The application successfully implements:
- Full log parsing and display
- Real-time filtering with text search and regex
- Async event handling
- Professional TUI with multiple view modes
- **Clipboard support** - Copy filtered logs with Ctrl+C
- **File export** - Save filtered logs to timestamped files with Ctrl+S
- Comprehensive test coverage (51 tests)

**Next Steps:**
- Add time range filtering
- Add help screen (press '?')
- CLI arguments for configuration
- Performance optimizations for high-volume streams

## Contributing

To add new features:

1. Follow the existing architecture (TEA pattern)
2. Add tests for new functionality
3. Update this README

## License

MIT
