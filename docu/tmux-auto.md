.tmux_start.sh

#!/usr/bin/env bash
set -euo pipefail

SESSION="main"
PROJECT="$HOME/ag"

# Ensure tmux is available
if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux not found in PATH" >&2
  exit 1
fi

# Ensure project directory exists
if [ ! -d "$PROJECT" ]; then
  echo "Project directory not found: $PROJECT" >&2
  exit 1
fi

# Create session if it doesn't exist
if ! tmux has-session -t "$SESSION" 2>/dev/null; then
  # Window Q: Qodo GUI (start in ~/ag)
  tmux new-session -d -s "$SESSION" -n Q -c "$PROJECT"
  tmux send-keys -t "$SESSION:Q" "cd \"$PROJECT\"" Enter
  tmux send-keys -t "$SESSION:Q" "qodo --login" Enter
  sleep 1
  tmux send-keys -t "$SESSION:Q" "qodo --gui" Enter

  # Wait for GUI hint so Tab+U will be accepted; fallback to time-based wait
  READY=0
  for i in {1..30}; do
    if tmux capture-pane -pt "$SESSION:Q" -S -200 2>/dev/null | grep -q "Tab\+U"; then
      READY=1
      break
    fi
    sleep 0.5
  done
  if [ "$READY" -eq 0 ]; then
    sleep 2
  fi

  # Send Tab+U to open Qodo browser (no Enter)
  tmux send-keys -t "$SESSION:Q" Tab U

  # Window prome: watch Prometheus readiness (start in ~/ag)
  tmux new-window -t "$SESSION:" -n prome -c "$PROJECT"
  tmux send-keys -t "$SESSION:prome" 'watch -n 1 curl -fsS http://localhost:9090/-/ready || true' Enter

  # Window graf: follow Grafana logs (start in ~/ag)
  tmux new-window -t "$SESSION:" -n graf -c "$PROJECT"
  tmux send-keys -t "$SESSION:graf" 'journalctl -u grafana-server -f' Enter

  # Focus Q by default
  tmux select-window -t "$SESSION:Q"
fi

exit 0

.bash.rc

# Kill tmux when terminal closes (any signal)
kill_tmux_on_exit() {
    tmux kill-server 2>/dev/null
    exit
}

trap 'tmux kill-server 2>/dev/null' EXIT SIGHUP SIGTERM


.bash_profile

# ~/.bash_profile

# Source .bashrc if it exists
if [ -f ~/.bashrc ]; then
  source ~/.bashrc
fi


# Auto-start tmux and ensure services are running
if [ -z "$TMUX" ]; then
  sudo systemctl is-active --quiet prometheus || sudo systemctl start prometheus
  sudo systemctl is-active --quiet grafana-server || sudo systemctl start grafana-server
  
  if tmux has-session -t main 2>/dev/null; then
    tmux attach-session -t main
  else
    tmux new-session -s main -d
    tmux new-window -t main -n Q
    tmux new-window -t main -n prome
    tmux new-window -t main -n graf
    tmux attach-session -t main
  fi
fi


Thanks for sharing your .bash_profile. It works, but to ensure Qodo always finds the agent in ~/ag and to integrate your tmux_start.sh automation, I suggest these improvements:

What to change

    cd to ~/ag before creating/attaching the tmux session, so all windows start in the right project.
    Use tmux -c to set the working directory for each new window.
    Launch the Qodo automation script (~/ag/tmux_start.sh) in the Q window so it runs qodo --login, qodo --gui, and sends Tab+U automatically.
    Keep your Prometheus and Grafana service checks as-is.

Drop-in replacement for ~/.bash_profile
~/.bash_profile
Source .bashrc if it exists

if [ -f ~/.bashrc ]; then source ~/.bashrc fi
Auto-start tmux and ensure services are running

if [ -z "$TMUX" ]; then
Ensure services are running

sudo systemctl is-active --quiet prometheus || sudo systemctl start prometheus sudo systemctl is-active --quiet grafana-server || sudo systemctl start grafana-server
Operate from ~/ag so Qodo finds the agent

cd "$HOME/ag" || exit 1

if tmux has-session -t main 2>/dev/null; then tmux attach-session -t main else # Create session and windows with working directory set to ~/ag tmux new-session -s main -d -n Q -c "$HOME/ag" # Start Qodo automation in Q window (will send Tab+U) tmux send-keys -t main:Q "$HOME/ag/tmux_start.sh" Enter
unknown

fi fi

Why this helps

    Qodo agent discovery: Always running in ~/ag avoids copying or symlinking agent files to ~.
    Reliability: tmux -c ensures any new window started later also uses ~/ag.
    Automation: Your tmux_start.sh handles qodo --login/--gui and Tab+U timing reliably.

Optional extras

    If you don’t want sudo prompts on login, consider enabling linger and starting services via systemd --user or allowing these services to start at boot.
    If you want a quick wrapper to always run qodo from ~/ag:
        Create ~/bin/qodo-ag with: #!/usr/bin/env bash cd "$HOME/ag" && exec qodo "$@"
        chmod +x ~/bin/qodo-ag


.tmux.conf

set -g set-titles on
set -g set-titles-string "#W"
set -g mouse on
set-option -ga terminal-overrides ',xterm-256color:Tc'
set -g status-bg '#00afaf' 
# Optional: also kill on detach
set-hook -g client-detached 'kill-server'



