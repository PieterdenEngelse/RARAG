# systemd unit for ag

This directory provides a sample systemd unit and environment file for running the Agentic RAG API server as a service.

It consolidates the systemd guidance from the project root INSTALLER.md with additional operational notes.

## Files
- `ag.service`: Sample unit file. Install to `/etc/systemd/system/ag.service`.
- `ag.env.example`: Example environment file. Copy to `/etc/default/ag` (Debian/Ubuntu) or `/etc/sysconfig/ag` (RHEL/CentOS) and edit values.
- Optional rate-limit rules: `/etc/ag/rl-routes.json` or `/etc/ag/rl-routes.yaml` (YAML requires build with `--features rl_yaml` or `--features full`).

## Workstation user service (per-user)
Recommended for per-user installs with no root changes.

- ExecStart: `~/.local/bin/ag`
- WorkingDirectory: `~/.local/share/ag`
- EnvironmentFile: `~/.config/ag/ag.env`

Steps:
```bash
mkdir -p ~/.config/systemd/user
cp ops/systemd/ag.service ~/.config/systemd/user/ag.service
mkdir -p ~/.config/ag
cp ops/systemd/ag.env.example ~/.config/ag/ag.env  # edit as needed
# Optional rate-limit rules file
cp src/monitoring/dashboards/sample_rate_limit_routes.json ~/.config/ag/rl-routes.json
systemctl --user daemon-reload
systemctl --user enable --now ag
journalctl --user -u ag -f
```

## System-wide service (servers)
Use if the service must start at boot and be shared by all users.

Typical layout:
- ExecStart: `/usr/local/bin/ag`
- WorkingDirectory: `/var/lib/ag`
- EnvironmentFile: `/etc/default/ag` (Debian/Ubuntu) or `/etc/sysconfig/ag` (RHEL)
- Rules file: `/etc/ag/rl-routes.json` or `/etc/ag/rl-routes.yaml`

Steps:
```bash
sudo cp ops/systemd/ag.service /etc/systemd/system/ag.service  # adapt for system-wide
sudo mkdir -p /etc/ag /var/lib/ag
sudo cp ops/systemd/ag.env.example /etc/default/ag && sudoedit /etc/default/ag
sudo systemctl daemon-reload
sudo systemctl enable --now ag
journalctl -u ag -f
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
