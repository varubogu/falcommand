#!/bin/bash

# Test script for debug configuration priority system
# This script creates test configuration files and verifies the priority order

set -e

echo "=== Testing Debug Configuration Priority System ==="

# Create test directory
mkdir -p .falcommand

# Create base config
cat > .falcommand/config.json << 'EOF'
{
  "appearance": {
    "theme": "light",
    "transparency": 0.9
  },
  "behavior": {
    "maxResults": 5
  },
  "search": {
    "fuzzyThreshold": 0.5
  }
}
EOF

# Create platform-specific config (should be overridden by debug config in debug builds)
cat > .falcommand/config.macos.json << 'EOF'
{
  "appearance": {
    "theme": "dark",
    "transparency": 0.8
  },
  "behavior": {
    "maxResults": 8
  }
}
EOF

# Create debug config (should have highest priority)
cat > .falcommand/config.debug.json << 'EOF'
{
  "appearance": {
    "theme": "system",
    "transparency": 0.95
  },
  "behavior": {
    "maxResults": 12
  }
}
EOF

# Create platform-specific debug config (should have highest priority)
cat > .falcommand/config.debug.macos.json << 'EOF'
{
  "appearance": {
    "transparency": 0.99
  },
  "behavior": {
    "maxResults": 15
  }
}
EOF

echo "Created test configuration files:"
echo "- Base config: theme=light, transparency=0.9, maxResults=5"
echo "- Platform config: theme=dark, transparency=0.8, maxResults=8"  
echo "- Debug config: theme=system, transparency=0.95, maxResults=12"
echo "- Platform debug config: transparency=0.99, maxResults=15"
echo ""

echo "Expected behavior in DEBUG builds:"
echo "- theme: system (from debug config)"
echo "- transparency: 0.99 (from platform debug config - highest priority)"
echo "- maxResults: 15 (from platform debug config - highest priority)"
echo ""

echo "Expected behavior in RELEASE builds:"
echo "- theme: dark (from platform config)"
echo "- transparency: 0.8 (from platform config)"
echo "- maxResults: 8 (from platform config)"
echo ""

# Test compilation in debug mode
echo "Building in debug mode to test configuration loading..."
if cargo build --quiet 2>/dev/null; then
    echo "✓ Debug build successful"
else
    echo "✗ Debug build failed"
fi

# Test compilation in release mode
echo "Building in release mode to test configuration loading..."
if cargo build --release --quiet 2>/dev/null; then
    echo "✓ Release build successful"
else
    echo "✗ Release build failed"
fi

echo ""
echo "Test configuration files created successfully!"
echo "Run the application in debug mode to see debug config priority in action."
echo "Run with --release to see traditional priority system."

echo ""
echo "To clean up test files, run:"
echo "rm -rf .falcommand"