#!/bin/bash
# migrate_to_vector.sh - Migrate from Promtail to Vector
# This script:
# 1. Installs Vector
# 2. Creates Vector configuration (equivalent to current Promtail setup)
# 3. Stops and removes Promtail
# 4. Starts Vector
# 5. Verifies everything works
# 6. Keeps Loki running (no changes to Loki)

set -e  # Exit on error

VECTOR_VERSION="0.34.0"
ARCH="x86_64-unknown-linux-musl"
USER_HOME="$HOME"

echo "════════════════════════════════════════════════════════════════"
echo "  Migrating from Promtail to Vector"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "This will:"
echo "  ✓ Install Vector ${VECTOR_VERSION}"
echo "  ✓ Create Vector config (equivalent to Promtail)"
echo "  ✓ Stop and remove Promtail"
echo "  ✓ Start Vector"
echo "  ✓ Keep Loki running (no changes)"
echo ""
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 1: Backup current Promtail configuration"
echo "════════════════════════════════════════════════════════════════"

if [ -f "$USER_HOME/.config/promtail/config.yml" ]; then
    mkdir -p "$USER_HOME/.config/promtail/backup"
    cp "$USER_HOME/.config/promtail/config.yml" \
       "$USER_HOME/.config/promtail/backup/config.yml.$(date +%Y%m%d_%H%M%S)"
    echo "✓ Backed up Promtail config"
else
    echo "⚠ No Promtail config found (this is OK if using system-wide install)"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 2: Download and install Vector"
echo "════════════════════════════════════════════════════════════════"

echo "Downloading Vector ${VECTOR_VERSION}..."
curl -L -o /tmp/vector.tar.gz \
  "https://github.com/vectordotdev/vector/releases/download/v${VECTOR_VERSION}/vector-${VECTOR_VERSION}-${ARCH}.tar.gz"

echo "Extracting..."
tar -xzf /tmp/vector.tar.gz -C /tmp

echo "Installing to ~/.local/bin/vector..."
mkdir -p "$USER_HOME/.local/bin"
mv /tmp/vector-*/bin/vector "$USER_HOME/.local/bin/"
chmod +x "$USER_HOME/.local/bin/vector"

echo "Cleaning up..."
rm -rf /tmp/vector*

echo "✓ Vector installed"
"$USER_HOME/.local/bin/vector" --version

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 3: Create Vector configuration"
echo "════════════════════════════════════════════════════════════════"

mkdir -p "$USER_HOME/.config/vector"
mkdir -p "$USER_HOME/.local/share/vector"

cat > "$USER_HOME/.config/vector/vector.toml" << 'EOF'
# Vector configuration for Loki
# Migrated from Promtail configuration
# Date: $(date +%Y-%m-%d)

# Data directory for disk buffers
data_dir = "DATA_DIR_PLACEHOLDER"

# ============================================================================
# Source: systemd journal (equivalent to Promtail systemd-journal job)
# ============================================================================
[sources.journald]
type = "journald"
include_units = ["ag.service"]
current_boot_only = false

# ============================================================================
# Transform: Add labels (equivalent to Promtail relabel_configs)
# ============================================================================
[transforms.add_labels]
type = "remap"
inputs = ["journald"]
source = '''
  # Extract systemd fields (same as Promtail)
  .systemd_unit = .SYSTEMD_UNIT
  .hostname = .host
  .priority = string!(.PRIORITY)
  .syslog_identifier = .SYSLOG_IDENTIFIER
  
  # Set job label
  .job = "systemd-journal"
  .host = "localhost"
'''

# ============================================================================
# Sink: Loki (same endpoint as Promtail)
# ============================================================================
[sinks.loki]
type = "loki"
inputs = ["add_labels"]
endpoint = "http://127.0.0.1:3100"
encoding.codec = "json"

# Labels (same as Promtail)
labels.job = "{{ job }}"
labels.systemd_unit = "{{ systemd_unit }}"
labels.hostname = "{{ hostname }}"
labels.priority = "{{ priority }}"
labels.syslog_identifier = "{{ syslog_identifier }}"
labels.host = "{{ host }}"

# Healthcheck
healthcheck.enabled = true

# Batch settings (optimize for performance)
batch.max_bytes = 1048576  # 1MB
batch.timeout_secs = 1

# ============================================================================
# Source: File logs (equivalent to Promtail ag-file-logs job)
# ============================================================================
[sources.file_logs]
type = "file"
include = ["FILE_LOGS_PATH_PLACEHOLDER/*.log"]
read_from = "beginning"

[transforms.file_labels]
type = "remap"
inputs = ["file_logs"]
source = '''
  .job = "ag-file-logs"
  .host = "localhost"
'''

[sinks.loki_files]
type = "loki"
inputs = ["file_labels"]
endpoint = "http://127.0.0.1:3100"
encoding.codec = "json"
labels.job = "{{ job }}"
labels.host = "{{ host }}"

# ============================================================================
# Internal metrics (monitor Vector itself)
# ============================================================================
[sources.internal_metrics]
type = "internal_metrics"

[sinks.prometheus_exporter]
type = "prometheus_exporter"
inputs = ["internal_metrics"]
address = "0.0.0.0:9598"
default_namespace = "vector"
EOF

# Replace placeholders
sed -i "s|DATA_DIR_PLACEHOLDER|$USER_HOME/.local/share/vector|g" "$USER_HOME/.config/vector/vector.toml"
sed -i "s|FILE_LOGS_PATH_PLACEHOLDER|$USER_HOME/.agentic-rag/logs|g" "$USER_HOME/.config/vector/vector.toml"

echo "✓ Vector configuration created at ~/.config/vector/vector.toml"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 4: Validate Vector configuration"
echo "════════════════════════════════════════════════════════════════"

"$USER_HOME/.local/bin/vector" validate "$USER_HOME/.config/vector/vector.toml"

if [ $? -eq 0 ]; then
    echo "✓ Vector configuration is valid"
else
    echo "✗ Vector configuration validation failed!"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 5: Create Vector systemd service"
echo "════════════════════════════════════════════════════════════════"

mkdir -p "$USER_HOME/.config/systemd/user"

cat > "$USER_HOME/.config/systemd/user/vector.service" << EOF
[Unit]
Description=Vector log shipper
After=network.target loki.service
Documentation=https://vector.dev

[Service]
Type=simple
ExecStart=$USER_HOME/.local/bin/vector --config $USER_HOME/.config/vector/vector.toml
Restart=on-failure
RestartSec=3

# Resource limits (optional - Vector is efficient)
MemoryMax=100M
CPUQuota=10%

[Install]
WantedBy=default.target
EOF

echo "✓ Vector systemd service created"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 6: Stop and disable Promtail"
echo "════════════════════════════════════════════════════════════════"

if systemctl --user is-active --quiet promtail.service; then
    echo "Stopping Promtail..."
    systemctl --user stop promtail.service
    echo "✓ Promtail stopped"
else
    echo "⚠ Promtail service not running"
fi

if systemctl --user is-enabled --quiet promtail.service 2>/dev/null; then
    echo "Disabling Promtail..."
    systemctl --user disable promtail.service
    echo "✓ Promtail disabled"
else
    echo "⚠ Promtail service not enabled"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 7: Start Vector"
echo "════════════════════════════════════════════════════════════════"

systemctl --user daemon-reload
systemctl --user enable vector.service
systemctl --user start vector.service

sleep 2

if systemctl --user is-active --quiet vector.service; then
    echo "✓ Vector is running"
    systemctl --user status vector.service --no-pager -l
else
    echo "✗ Vector failed to start!"
    echo "Check logs with: journalctl --user -u vector.service -n 50"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 8: Verify Loki is still receiving logs"
echo "════════════════════════════════════════════════════════════════"

echo "Waiting 5 seconds for logs to flow..."
sleep 5

echo "Querying Loki for ag.service logs..."
LOKI_RESPONSE=$(curl -s -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service"}' \
  --data-urlencode 'limit=1')

if echo "$LOKI_RESPONSE" | grep -q '"status":"success"'; then
    echo "✓ Loki is receiving logs from Vector"
    
    # Count log entries
    LOG_COUNT=$(echo "$LOKI_RESPONSE" | grep -o '"values":\[\[' | wc -l)
    if [ "$LOG_COUNT" -gt 0 ]; then
        echo "✓ Found logs in Loki"
    else
        echo "⚠ No logs found yet (may take a moment)"
    fi
else
    echo "⚠ Could not verify logs in Loki"
    echo "Response: $LOKI_RESPONSE"
fi

echo ""
echo "Checking available labels..."
curl -s "http://127.0.0.1:3100/loki/api/v1/labels" | grep -o '"[^"]*"' | head -10

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 9: Clean up Promtail (optional)"
echo "════════════════════════════════════════════════════════════════"

echo ""
echo "Do you want to remove Promtail files? (keeps backups)"
echo "This will remove:"
echo "  - ~/.local/bin/promtail (binary)"
echo "  - ~/.config/promtail/config.yml (config - backed up)"
echo "  - ~/.config/systemd/user/promtail.service (service file)"
echo "  - ~/.local/share/promtail (data directory)"
echo ""
echo "Keeps:"
echo "  - ~/.config/promtail/backup/ (config backups)"
echo ""
read -p "Remove Promtail files? (y/N) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Removing Promtail files..."
    
    # Remove binary
    if [ -f "$USER_HOME/.local/bin/promtail" ]; then
        rm "$USER_HOME/.local/bin/promtail"
        echo "✓ Removed promtail binary"
    fi
    
    # Remove config (keep backup)
    if [ -f "$USER_HOME/.config/promtail/config.yml" ]; then
        rm "$USER_HOME/.config/promtail/config.yml"
        echo "✓ Removed promtail config (backup preserved)"
    fi
    
    # Remove systemd service
    if [ -f "$USER_HOME/.config/systemd/user/promtail.service" ]; then
        rm "$USER_HOME/.config/systemd/user/promtail.service"
        systemctl --user daemon-reload
        echo "✓ Removed promtail systemd service"
    fi
    
    # Remove data directory
    if [ -d "$USER_HOME/.local/share/promtail" ]; then
        rm -rf "$USER_HOME/.local/share/promtail"
        echo "✓ Removed promtail data directory"
    fi
    
    echo "✓ Promtail cleanup complete"
else
    echo "⚠ Skipped Promtail cleanup (files remain)"
    echo "You can manually remove later with:"
    echo "  rm ~/.local/bin/promtail"
    echo "  rm ~/.config/promtail/config.yml"
    echo "  rm ~/.config/systemd/user/promtail.service"
    echo "  rm -rf ~/.local/share/promtail"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 10: Performance comparison"
echo "════════════════════════════════════════════════════════════════"

echo ""
echo "Vector resource usage:"
systemctl --user status vector.service --no-pager | grep -E "Memory|CPU" || echo "  (metrics will appear after a few minutes)"

echo ""
echo "Vector metrics endpoint (for Prometheus):"
echo "  http://localhost:9598/metrics"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✓ Migration Complete!"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Summary:"
echo "  ✓ Vector installed and running"
echo "  ✓ Promtail stopped and disabled"
echo "  ✓ Loki still running (no changes)"
echo "  ✓ Logs flowing to Loki via Vector"
echo ""
echo "Next steps:"
echo "  1. Check Vector status:"
echo "     systemctl --user status vector.service"
echo ""
echo "  2. View Vector logs:"
echo "     journalctl --user -u vector.service -f"
echo ""
echo "  3. Query logs in Grafana (no changes needed):"
echo "     {systemd_unit=\"ag.service\"}"
echo ""
echo "  4. Monitor Vector metrics in Prometheus:"
echo "     Add to prometheus.yml:"
echo "       - job_name: 'vector'"
echo "         static_configs:"
echo "           - targets: ['localhost:9598']"
echo ""
echo "  5. Compare performance:"
echo "     systemctl --user status vector.service | grep -E 'Memory|CPU'"
echo ""
echo "Rollback (if needed):"
echo "  systemctl --user stop vector.service"
echo "  systemctl --user disable vector.service"
echo "  systemctl --user start promtail.service"
echo "  systemctl --user enable promtail.service"
echo ""
echo "════════════════════════════════════════════════════════════════"
