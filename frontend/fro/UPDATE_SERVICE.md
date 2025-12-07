# Update Dioxus Service

To make the dioxus service consistent with other services in the `ba` tmux session, run these commands:

## 1. Update the systemd service file

```bash
sudo cp /home/pde/ag/frontend/fro/dioxus.service /etc/systemd/system/dioxus.service
```

## 2. Reload systemd and restart the service

```bash
sudo systemctl daemon-reload
sudo systemctl restart dioxus.service
```

## 3. Verify it's running

```bash
sudo systemctl status dioxus.service
```

## What this does:

- Uses the startup script with extensive logging (RUST_LOG=trace,dioxus=debug)
- Runs on port 1789 as configured
- Logs go to journald (viewable with `journalctl -u dioxus.service -f`)
- The tmux panes are already configured:
  - Pane 0 (top): Shows service status with `watch`
  - Pane 1 (bottom): Shows live logs with `journalctl -f`

This matches the setup of other services like prometheus, grafana, ag, etc. in the `ba` session.
