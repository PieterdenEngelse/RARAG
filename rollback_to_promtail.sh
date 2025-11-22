#!/bin/bash
# rollback_to_promtail.sh - Rollback from Vector to Promtail
# Use this if you need to revert the migration

set -e

USER_HOME="$HOME"

echo "════════════════════════════════════════════════════════════════"
echo "  Rollback: Vector → Promtail"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "This will:"
echo "  ✓ Stop and disable Vector"
echo "  ✓ Restore Promtail configuration from backup"
echo "  ✓ Start Promtail"
echo "  ✓ Keep Loki running (no changes)"
echo ""
read -p "Continue with rollback? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 1: Stop Vector"
echo "════════════════════════════════════════════════════════════════"

if systemctl --user is-active --quiet vector.service; then
    systemctl --user stop vector.service
    echo "✓ Vector stopped"
else
    echo "⚠ Vector not running"
fi

systemctl --user disable vector.service 2>/dev/null || true
echo "✓ Vector disabled"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 2: Check for Promtail backup"
echo "════════════════════════════════════════════════════════════════"

BACKUP_DIR="$USER_HOME/.config/promtail/backup"
if [ -d "$BACKUP_DIR" ]; then
    LATEST_BACKUP=$(ls -t "$BACKUP_DIR"/config.yml.* 2>/dev/null | head -1)
    if [ -n "$LATEST_BACKUP" ]; then
        echo "✓ Found backup: $LATEST_BACKUP"
        cp "$LATEST_BACKUP" "$USER_HOME/.config/promtail/config.yml"
        echo "✓ Restored Promtail configuration"
    else
        echo "✗ No backup found!"
        echo "You'll need to manually recreate Promtail config"
        exit 1
    fi
else
    echo "✗ No backup directory found!"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 3: Check Promtail binary"
echo "════════════════════════════════════════════════════════════════"

if [ ! -f "$USER_HOME/.local/bin/promtail" ]; then
    echo "⚠ Promtail binary not found - reinstalling..."
    
    PROMTAIL_VERSION="3.0.0"
    curl -L -o /tmp/promtail.zip \
      "https://github.com/grafana/loki/releases/download/v${PROMTAIL_VERSION}/promtail-linux-amd64.zip"
    
    unzip -o /tmp/promtail.zip -d /tmp
    chmod +x /tmp/promtail-linux-amd64
    mv /tmp/promtail-linux-amd64 "$USER_HOME/.local/bin/promtail"
    rm /tmp/promtail.zip
    
    echo "✓ Promtail binary reinstalled"
else
    echo "✓ Promtail binary exists"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 4: Restore Promtail systemd service"
echo "════════════════════════════════════════════════════════════════"

if [ ! -f "$USER_HOME/.config/systemd/user/promtail.service" ]; then
    echo "Creating Promtail systemd service..."
    
    cat > "$USER_HOME/.config/systemd/user/promtail.service" << EOF
[Unit]
Description=Grafana Promtail (user)
After=network.target loki.service

[Service]
Type=simple
ExecStart=$USER_HOME/.local/bin/promtail -config.file=$USER_HOME/.config/promtail/config.yml
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF
    
    echo "✓ Promtail systemd service created"
else
    echo "✓ Promtail systemd service exists"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 5: Start Promtail"
echo "════════════════════════════════════════════════════════════════"

systemctl --user daemon-reload
systemctl --user enable promtail.service
systemctl --user start promtail.service

sleep 2

if systemctl --user is-active --quiet promtail.service; then
    echo "✓ Promtail is running"
    systemctl --user status promtail.service --no-pager -l
else
    echo "✗ Promtail failed to start!"
    echo "Check logs with: journalctl --user -u promtail.service -n 50"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 6: Verify Loki is receiving logs"
echo "════════════════════════════════════════════════════════════════"

echo "Waiting 5 seconds for logs to flow..."
sleep 5

LOKI_RESPONSE=$(curl -s -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service"}' \
  --data-urlencode 'limit=1')

if echo "$LOKI_RESPONSE" | grep -q '"status":"success"'; then
    echo "✓ Loki is receiving logs from Promtail"
else
    echo "⚠ Could not verify logs in Loki"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✓ Rollback Complete!"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Summary:"
echo "  ✓ Promtail restored and running"
echo "  ✓ Vector stopped and disabled"
echo "  ✓ Loki still running (no changes)"
echo ""
echo "Vector files remain at:"
echo "  - ~/.local/bin/vector"
echo "  - ~/.config/vector/"
echo "  - ~/.config/systemd/user/vector.service"
echo ""
echo "To completely remove Vector:"
echo "  rm ~/.local/bin/vector"
echo "  rm -rf ~/.config/vector"
echo "  rm ~/.config/systemd/user/vector.service"
echo "  systemctl --user daemon-reload"
echo ""
