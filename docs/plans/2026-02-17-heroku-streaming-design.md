# Heroku Log Streaming for Desktop App

**Date:** 2026-02-17
**Status:** Approved

## Overview

Add live Heroku log streaming to the desktop GUI application using the Heroku CLI as the connection mechanism. The desktop app will pivot from file-based log viewing to a pure streaming interface.

## Goals

- Enable live streaming of Heroku application logs in the desktop GUI
- Provide automatic reconnection on connection failures
- Maintain existing filtering and viewing capabilities
- Simplify the interface by removing file picker and export features

## Non-Goals

- Direct Heroku API integration (using CLI instead)
- Multiple simultaneous app streams
- Historical log file viewing
- Export/clipboard functionality

## Requirements Summary

- **Interface:** Desktop app only (TUI already supports stdin streaming)
- **Connection:** Execute Heroku CLI commands (`heroku logs --tail`)
- **App Selection:** Dropdown of available apps (via `heroku apps --json`)
- **Mode:** Stream-only (remove file picker functionality)
- **Filtering:** Simple connection, apply filters within app UI
- **Error Handling:** Graceful detection with helpful error messages
- **Reconnection:** Auto-reconnect with exponential backoff
- **Export:** Remove clipboard and file export features

## Architecture

### High-Level Flow

```
User launches app
    ↓
Check Heroku CLI availability
    ↓ (if available)
Fetch available apps → Show dropdown
    ↓
User selects app + clicks Connect
    ↓
Spawn `heroku logs --tail --app <name>` (tokio::process)
    ↓
Background task reads stdout line-by-line
    ↓
Parse lines → LogEntry structs
    ↓
Update Signal<Vec<LogEntry>> → UI rerenders
    ↓
If process exits/errors → Auto-reconnect with backoff
```

### UI States

1. **Loading** - Checking CLI and fetching apps
2. **Ready** - Dropdown populated, ready to connect
3. **Connecting** - Process starting
4. **Streaming** - Live logs flowing
5. **Reconnecting** - Process died, retrying
6. **Error** - CLI missing or auth failed

## Components

### New Modules

```
src/desktop/
├── main.rs                    (Modified - new UI, remove file picker)
├── heroku_cli.rs              (New - CLI interaction)
├── stream_manager.rs          (New - process lifecycle)
├── components/
│   ├── connection_panel.rs    (New - app dropdown + connect button)
│   ├── status_indicator.rs    (New - connection status display)
│   ├── stats_header.rs        (Modified - remove file-related stats)
│   ├── filter_bar.rs          (Keep as-is)
│   └── log_view.rs            (Keep as-is)
```

### Module Responsibilities

**`heroku_cli.rs`** - Heroku CLI wrapper
- `check_cli_installed() -> Result<bool>` - Verify `heroku` command exists
- `check_authentication() -> Result<String>` - Run `heroku auth:whoami`, return email
- `fetch_apps() -> Result<Vec<AppInfo>>` - Parse `heroku apps --json`

**`stream_manager.rs`** - Process lifecycle management
- `StreamManager::connect(app_name) -> Result<()>` - Spawn streaming process
- `StreamManager::disconnect()` - Kill process gracefully
- `StreamManager::reconnect()` - Retry with exponential backoff
- Manages: process handle, stdout reader task, reconnection state

**`connection_panel.rs`** - Connection UI component
- App dropdown (populated from `fetch_apps()`)
- "Connect" / "Disconnect" button (state-dependent)
- Shows selected app name when connected

**`status_indicator.rs`** - Visual connection status
- Shows: Ready / Connecting / Streaming / Reconnecting / Error
- Color-coded (green=streaming, yellow=reconnecting, red=error)

## Data Flow

### App State Structure

```rust
struct AppState {
    // Connection state
    connection_status: Signal<ConnectionStatus>,
    selected_app: Signal<Option<String>>,
    available_apps: Signal<Vec<AppInfo>>,

    // Log data
    logs: Signal<Vec<LogEntry>>,

    // Existing filter state (unchanged)
    filters: Signal<Vec<Filter>>,
    filter_mode_and: Signal<bool>,

    // Stream manager handle
    stream_manager: Signal<Option<StreamManager>>,
}

enum ConnectionStatus {
    Loading,           // Fetching apps
    Ready,             // Ready to connect
    Connecting,        // Starting process
    Streaming,         // Live and receiving logs
    Reconnecting(u32), // Retry attempt N
    Error(String),     // CLI missing, auth failed, etc.
}
```

### Connection Flow

1. **Startup:**
   - Check CLI → Set status to Ready or Error
   - Fetch apps → Populate dropdown

2. **Connect:**
   - User selects app + clicks Connect
   - Status → Connecting
   - `StreamManager::connect(app_name)` spawns process
   - Background task reads stdout

3. **Streaming:**
   - Each line → Parse to `LogEntry`
   - Append to `logs` signal
   - Status → Streaming
   - UI auto-updates (Dioxus reactivity)

4. **Reconnection:**
   - Process exits → Status → Reconnecting(1)
   - Wait 1 second → Retry
   - If fails → Reconnecting(2), wait 2 seconds
   - If fails → Reconnecting(3), wait 4 seconds
   - Max 5 attempts → Status → Error

5. **Disconnect:**
   - User clicks Disconnect
   - Kill process
   - Clear logs
   - Status → Ready

## Error Handling

### Error Scenarios

**1. Heroku CLI Not Installed**
- Detection: `which heroku` returns empty or error
- Message: "Heroku CLI not found. Please install from heroku.com/cli"
- Action: Block connection UI, show installation guide link

**2. Not Authenticated**
- Detection: `heroku auth:whoami` exits with error
- Message: "Not logged in to Heroku. Run 'heroku login' in terminal"
- Action: Block connection UI, offer "Retry" button

**3. No Apps Available**
- Detection: `heroku apps --json` returns empty array
- Message: "No Heroku apps found in your account"
- Action: Disable Connect button

**4. Connection Fails**
- Detection: Process exits immediately with error
- Handling: Parse stderr, show user-friendly message
- Action: Show error in status indicator, allow retry

**5. Stream Disconnects Mid-Session**
- Detection: Process exits unexpectedly
- Handling: Auto-reconnect with exponential backoff
- Status: Show "Reconnecting... (attempt N/5)"
- Action: Retry up to 5 times, then manual reconnect

**6. Process Cleanup on App Exit**
- Detection: User closes window or quits app
- Handling: Kill child process in cleanup handler

### Edge Cases

- **App renamed/deleted during session:** Stream breaks → Reconnect fails → Show error
- **Network interruption:** Auto-reconnect handles it
- **Multiple rapid connect/disconnect:** Debounce connect button, cancel previous process
- **Huge log volume:** Existing circular buffer (10k entries) handles it
- **Malformed log lines:** Parser handles gracefully (Unknown level)

## Testing Strategy

### Unit Tests

**`heroku_cli.rs` tests:**
- Mock CLI commands with test fixtures
- Test JSON parsing of `heroku apps --json`
- Test error handling for missing CLI
- Test authentication check parsing

**`stream_manager.rs` tests:**
- Test process spawning and cleanup
- Test line reading and parsing integration
- Test reconnection backoff logic
- Test graceful shutdown

### Integration Tests

**End-to-end flow:**
1. App startup → CLI check → App list fetch
2. User selects app → Connect → Process spawns
3. Mock log lines → Verify parsing → UI updates
4. Disconnect → Process killed → State reset

**Reconnection test:**
- Spawn process that exits after N lines
- Verify reconnection attempts
- Verify exponential backoff timing
- Verify max retry limit

### Manual Testing Checklist

- [ ] Fresh install (no Heroku CLI) → Shows error
- [ ] CLI installed but not logged in → Shows auth error
- [ ] Valid setup → Apps populate dropdown
- [ ] Connect to app → Logs stream in real-time
- [ ] Apply filters → Logs filter correctly
- [ ] Kill network → Auto-reconnects
- [ ] Close app while streaming → No orphaned processes
- [ ] Rapid connect/disconnect → No crashes

## Implementation Approach

**Chosen Approach:** Tokio Async Process Streaming

- Use `tokio::process::Command` with stdout piped
- Background task reads lines using `BufReader`
- Parse each line through existing `LogEntry` parser
- Send parsed logs to UI via `Signal` updates
- Process handle stored for cleanup/reconnection

**Why this approach:**
- Clean async integration with existing Dioxus runtime
- Easy to kill/restart process for reconnection
- Non-blocking, fits naturally with current desktop app architecture
- Can detect process exit and trigger reconnection
- Good balance of simplicity and capability

## Removed Features

To streamline the interface for pure streaming:
- File picker (Ctrl+O)
- Export to file (Ctrl+S)
- Copy to clipboard (Ctrl+C)
- All file-related UI buttons and handlers

## Dependencies

Existing dependencies are sufficient:
- `tokio` - Process spawning and async runtime
- `dioxus` - Desktop UI framework
- `logs_parser` lib - Existing parser, filters, components

## Success Criteria

- [ ] Desktop app can list Heroku apps
- [ ] User can connect to selected app
- [ ] Logs stream in real-time
- [ ] Filters work on streamed logs
- [ ] Auto-reconnection works during network issues
- [ ] Graceful error messages for setup issues
- [ ] No orphaned processes on app exit
- [ ] All tests pass
