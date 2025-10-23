#!/bin/bash

# Toggle rust-analyzer in VS Code
# This script disables/enables the rust-analyzer extension and restarts VS Code

EXTENSION_ID="rust-lang.rust-analyzer"

# Check if extension is currently enabled
IS_ENABLED=$(code --list-extensions --show-versions | grep -i "$EXTENSION_ID" || true)

if [ -n "$IS_ENABLED" ]; then
    echo "Disabling rust-analyzer extension..."
    code --uninstall-extension "$EXTENSION_ID"
    
    echo "Killing rust-analyzer processes..."
    pkill -9 rust-analyzer 2>/dev/null || echo "No rust-analyzer processes found"
    
    echo "Restarting VS Code to fully kill the process..."
    # Kill VS Code
    pkill -9 "code" 2>/dev/null || pkill -9 "Code" 2>/dev/null
    
    # Wait a moment for processes to die
    sleep 1
    
    # Restart VS Code
    code . &
    
    echo "✓ Rust-analyzer has been disabled, processes killed, and VS Code restarted"
else
    echo "Rust-analyzer is already disabled or not installed"
    echo "Installing rust-analyzer extension..."
    code --install-extension "$EXTENSION_ID"
    
    echo "Restarting VS Code..."
    pkill -9 "code" 2>/dev/null || pkill -9 "Code" 2>/dev/null
    sleep 1
    code . &
    
    echo "✓ Rust-analyzer has been enabled and VS Code restarted"
fi