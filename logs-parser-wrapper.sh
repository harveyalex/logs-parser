#!/bin/bash

# Wrapper script that ensures keyboard input works even with piped stdin
# by explicitly connecting the terminal

# Save piped input to a temp file if stdin is piped
if [ ! -t 0 ]; then
    TEMP_LOG=$(mktemp)
    cat > "$TEMP_LOG"

    # Run the parser with stdin from terminal and logs from temp file
    tail -f "$TEMP_LOG" < /dev/tty | cargo run --release

    rm -f "$TEMP_LOG"
else
    # No piped input, run normally
    cargo run --release
fi
