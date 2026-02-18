#!/bin/bash

echo "=== Heroku Logs Parser Debug ==="
echo "Terminal: $TERM"
echo "Ghostty version: $(ghostty --version 2>/dev/null || echo 'unknown')"
echo ""
echo "Starting in 2 seconds..."
sleep 2

# Run with error output
cargo run --release 2>&1
