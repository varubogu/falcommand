#!/bin/bash

echo "=== Reproducing the configuration error ==="
echo "Building the application..."
cargo build

echo -e "\n=== Running the application to reproduce error ==="
cargo run 2>&1

echo -e "\n=== Looking for config files in typical locations ==="
echo "Checking ~/Library/Preferences (macOS default):"
find ~/Library/Preferences -name "*falcommand*" 2>/dev/null || echo "None found"

echo -e "\nChecking ~/.config:"
find ~/.config -name "*falcommand*" 2>/dev/null || echo "None found"

echo -e "\nChecking current directory for config files:"
find . -name "*.json" -o -name "config*" | head -10