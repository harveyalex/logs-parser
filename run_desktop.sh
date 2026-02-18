#!/bin/bash
set -e

echo "Building Dioxus Desktop App..."
cargo build --release --bin logs-parser-desktop

echo ""
echo "Starting desktop application..."
./target/release/logs-parser-desktop
