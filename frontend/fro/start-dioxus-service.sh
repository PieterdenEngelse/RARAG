#!/bin/bash
# Dioxus startup script for systemd service
# Logs to both journald (for systemd) and a file

cd /home/pde/ag/frontend/fro

# Set extensive logging
export RUST_LOG=trace,dioxus=debug
export RUST_BACKTRACE=1

# Start dx serve with verbose output
# Output goes to journald automatically when run by systemd
exec dx serve --platform web --port 1789 --verbose
