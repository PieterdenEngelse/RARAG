#!/bin/bash
# ~/ag/tmux_startup.sh
# Initialize tmux session "main" with standard windows
# Handles all tmux session/window management and auto-attach
set -euo pipefail

LOG_FILE="$HOME/.tmux_startup.log"

{
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting tmux session setup"
    
    SESSION="main"
    AG_HOME="$HOME/ag"
    SESSION_CREATED=0

    # Helper function to create a window only if it doesn't exist
    create_window_if_missing() {
        local window_name="$1"
        local window_path="$2"
        if tmux list-windows -t "$SESSION" -F "#{window_name}" 2>/dev/null | grep -q "^${window_name}$"; then
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] Window $window_name already exists"
            return 0
        else
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] Creating window $window_name"
            tmux new-window -t "$SESSION:" -n "$window_name" -c "$window_path" -d
        fi
    }

    # Create session if it doesn't exist
    if ! tmux has-session -t "$SESSION" 2>/dev/null; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Creating new tmux session $SESSION"
        tmux new-session -s "$SESSION" -d -c "$AG_HOME" -n "Q"
        SESSION_CREATED=1
    fi

    # Create named windows
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Creating windows..."
    create_window_if_missing "prome" "$AG_HOME"
    create_window_if_missing "graf" "$AG_HOME"
    create_window_if_missing "otlp" "$AG_HOME"
    create_window_if_missing "tmux" "$HOME"
    create_window_if_missing "ag" "$HOME"
    create_window_if_missing "bash3" "$HOME"
    create_window_if_missing "bash4" "$HOME"

    # Send commands to specific windows
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Setting up window commands"

    # Q window: auto-run qodo login/gui only once per new tmux session
    if tmux list-windows -t "$SESSION" | grep -q "Q"; then
        if [ "$SESSION_CREATED" -eq 1 ]; then
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting qodo in Q window (one-time for this session)"
            tmux send-keys -t "$SESSION:Q" "qodo login" C-m
            sleep 1
            tmux send-keys -t "$SESSION:Q" "qodo --gui" C-m
        else
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] Q window ready (session already existed; skipping auto qodo commands)"
        fi
    fi

    # prome window: prometheus status/logs
    if tmux list-windows -t "$SESSION" | grep -q "prome"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting prometheus logs in prome window"
        tmux send-keys -t "$SESSION:prome" "sudo journalctl -u prometheus -f" C-m
    fi

    # graf window: grafana status + logs
    if tmux list-windows -t "$SESSION" | grep -q "graf"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting grafana status and logs in graf window"
        tmux send-keys -t "$SESSION:graf" "sudo systemctl status grafana-server && sudo journalctl -u grafana-server -f" C-m
    fi

    # otlp window: start otelcol service and tail its journal
    if tmux list-windows -t "$SESSION" | grep -q "otlp"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting otelcol service and logs in otlp window"
        tmux send-keys -t "$SESSION:otlp" "systemctl --user start otelcol.service && journalctl --user -u otelcol.service -f" C-m
    fi

    # tmux window: tmux-session service status
    if tmux list-windows -t "$SESSION" | grep -q "tmux"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting tmux-session status watch in tmux window"
        tmux send-keys -t "$SESSION:tmux" "watch -n 5 'systemctl --user status tmux-session.service'" C-m
    fi

    # ag window: ag-service status and logs
    if tmux list-windows -t "$SESSION" | grep -q "ag"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting ag-service status and logs in ag window"
        sleep 1
        tmux send-keys -t "$SESSION:ag" "sudo systemctl status ag.service && sudo journalctl -u ag.service -f" C-m
    fi

    # Reload tmux config to apply any formatting
    tmux source-file ~/.tmux.conf 2>/dev/null || true
    
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Tmux session setup complete"

} >> "$LOG_FILE" 2>&1
