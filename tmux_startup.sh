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
    create_window_if_missing "pr" "$AG_HOME"
    create_window_if_missing "gr" "$AG_HOME"
    create_window_if_missing "ot" "$AG_HOME"
    create_window_if_missing "tm" "$HOME"
    create_window_if_missing "ag" "$HOME"
    create_window_if_missing "te" "$HOME"
    create_window_if_missing "lo" "$HOME"
    create_window_if_missing "q2" "$HOME"

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
    if tmux list-windows -t "$SESSION" | grep -q "pr"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting prometheus logs in pr window"
        tmux send-keys -t "$SESSION:pr" "sudo journalctl -u prometheus -f" C-m
    fi

    # graf window: grafana status + logs
    if tmux list-windows -t "$SESSION" | grep -q "gr"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting grafana status and logs in gr window"
        tmux send-keys -t "$SESSION:gr" "sudo systemctl status grafana-server && sudo journalctl -u grafana-server -f" C-m
    fi

    # otlp window: start otelcol service and tail its journal
    if tmux list-windows -t "$SESSION" | grep -q "ot"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting otelcol service and logs in ot window"
        tmux send-keys -t "$SESSION:ot" "systemctl --user start otelcol.service && journalctl --user -u otelcol.service -f" C-m
    fi

    # tmux window: tmux-session service status 
    if tmux list-windows -t "$SESSION" | grep -q "tm"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting tmux-session status watch in tm window"
        tmux send-keys -t "$SESSION:tm" "watch -n 5 'systemctl --user status tmux-session.service'" C-m
    fi

    # ag window: ag-service status and logs
    if tmux list-windows -t "$SESSION" | grep -q "ag"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting ag-service status and logs in ag window"
        sleep 1
        tmux send-keys -t "$SESSION:ag" "sudo systemctl status ag.service && sudo journalctl -u ag.service -f" C-m
    fi

    # tempo window: tempo status and logs
    if tmux list-windows -t "$SESSION" | grep -q "te"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting temp-service status and logs in tem window"
        sleep 1
        tmux send-keys -t "$SESSION:tem" "sudo systemctl status tempo.service && sudo journalctl -u tempo.service -f" C-m
    fi

    # loki window: loki status and logs
    if tmux list-windows -t "$SESSION" | grep -q "lok"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting loki-service status and logs in lok window"
        sleep 1
        tmux send-keys -t "$SESSION:lok" "sudo systemctl status loki.service && sudo journalctl -u loki.service -f" C-m
    fi

    # qodo gui: qodo-gui.service status and logs
    if tmux list-windows -t "$SESSION" | grep -q "q2"; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting  qodo-gui.service status and logs in ag window"
        sleep 1
        tmux send-keys -t "$SESSION:q2" "systemctl --user status qodo-gui.service && sudo journalctl --user -u qodo-gui.service -f" C-m
    fi

    # Reload tmux config to apply any formatting
    tmux source-file ~/.tmux.conf 2>/dev/null || true
    
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Tmux session setup complete"

} >> "$LOG_FILE" 2>&1
