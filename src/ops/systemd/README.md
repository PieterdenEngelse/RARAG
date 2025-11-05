# systemd unit for ag

This directory provides a sample systemd unit and environment file for running the Agentic RAG API server as a service.

## Files
- `ag.service`: Sample unit file. Install to `/etc/systemd/system/ag.service`.
- `ag.env.example`: Example environment file. Copy to `/etc/default/ag` (Debian/Ubuntu) or `/etc/sysconfig/ag` (RHEL/CentOS) and edit values.

## Install
```bash
# Copy unit
sudo cp ops/systemd/ag.service /etc/systemd/system/ag.service

# Create config dir for app-specific files
sudo mkdir -p /etc/ag

# Copy environment (Debian/Ubuntu)
sudo cp ops/systemd/ag.env.example /etc/default/ag
# or (RHEL/CentOS)
# sudo cp ops/systemd/ag.env.example /etc/sysconfig/ag

# Provide rate-limit rules file (JSON or YAML)
sudo cp src/monitoring/dashboards/sample_rate_limit_routes.json /etc/ag/rl-routes.json
# For YAML, ensure binary built with `--features rl_yaml`
# sudo cp path/to/rl-routes.yaml /etc/ag/rl-routes.yaml

# Reload units and enable service
sudo systemctl daemon-reload
sudo systemctl enable --now ag
```

## Override configuration without editing the unit
Use `systemctl edit ag` to create `/etc/systemd/system/ag.service.d/override.conf`:
```ini
[Service]
Environment=RUST_LOG=info
Environment=RATE_LIMIT_ROUTES_FILE=/etc/ag/rl-routes.yaml
```
Then reload and restart:
```bash
sudo systemctl daemon-reload
sudo systemctl restart ag
```

## Paths
- Binary: `/usr/local/bin/ag` (adjust in unit if you install elsewhere)
- WorkingDirectory: `/opt/ag` (set to your deployed path)
- EnvironmentFile: `/etc/default/ag` or `/etc/sysconfig/ag`
- Rate limit rules file: `/etc/ag/rl-routes.json` or `/etc/ag/rl-routes.yaml`

## Logs
- Journald: `journalctl -u ag -f`
- Application logs may also be written based on RUST_LOG and your tracing configuration.

## Notes
- If you use YAML rules, build the binary with `--features rl_yaml` (or `--features full`).
- TRUST_PROXY should be set to `true` only if a trusted reverse proxy injects `X-Forwarded-For`/`Forwarded` headers.
- Consider running as a dedicated `ag` user and group, and adjust file permissions for `/etc/ag` accordingly.
