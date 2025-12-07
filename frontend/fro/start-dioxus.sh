#!/bin/bash
cd /home/pde/ag/frontend/fro

# Set extensive logging
export RUST_LOG=trace,dioxus=debug
export RUST_BACKTRACE=1

# Log file location
LOG_FILE="/tmp/dioxus-serve.log"

# Start dx serve with verbose output and log everything
echo "Starting Dioxus at $(date)" | tee -a "$LOG_FILE"
echo "Working directory: $(pwd)" | tee -a "$LOG_FILE"
echo "RUST_LOG=$RUST_LOG" | tee -a "$LOG_FILE"
echo "Command: dx serve --platform web --port 1789 --verbose" | tee -a "$LOG_FILE"
echo "----------------------------------------" | tee -a "$LOG_FILE"

exec dx serve --platform web --port 1789 --verbose 2>&1 | tee -a "$LOG_FILE"
