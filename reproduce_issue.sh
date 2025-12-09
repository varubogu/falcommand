#!/bin/bash

echo "=== FalCommand Reproduction Script ==="
echo "Testing immediate termination issue..."
echo

# Check if the project compiles
echo "1. Building the project..."
cargo build
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to build the project"
    exit 1
fi

echo "2. Running the application..."
echo "Starting application with logging enabled..."
export RUST_LOG=debug
timeout 10s cargo run 2>&1 | tee app_output.log

echo
echo "3. Checking application behavior..."
if [ $? -eq 124 ]; then
    echo "Application ran for 10 seconds (timeout reached) - this suggests it's staying alive"
else
    echo "Application terminated before timeout - confirming immediate termination issue"
fi

echo
echo "4. Output analysis:"
cat app_output.log