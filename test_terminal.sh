#!/bin/bash

# Simple test to generate logs slowly so you can test input
echo "Starting test log generator..."
echo "Press 'q' in the parser to quit"
echo ""

for i in {1..100}; do
    echo "2010-09-16T15:13:46.677020+00:00 app[web.1]: Test message $i"
    sleep 0.5
done
