#!/bin/bash

# Test if it works in a different terminal
echo "Testing in Terminal.app or iTerm2..."
echo ""

./test_terminal.sh | /Users/alexharvey/development/logs-parser/target/release/logs-parser
