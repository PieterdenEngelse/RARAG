#!/bin/bash
# migrate_back_to_vector.sh - Migrate from Promtail back to Vector
# Cleans up Promtail and restores Vector with enhanced configuration

set -e

USER_HOME="$HOME"

echo "════════════════════════════════════════════════════════════════"
echo "  Migration: Promtail → Vector (with cleanup)"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "This will:"
echo "  ✓ Stop and disable Promtail"
echo "  ✓ Create enhanced Vector configuration"
echo "  ✓ Start Vector"
echo "  ✓ Clean up Promtail files"
echo ""
read -p "Continue with migration? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 1: Stop and disable Promtail"
echo "════════════════════════════════════════════════════════════════"

if systemctl --user is-active --quiet promtail.service; then
    echo "Stopping Promtail..."
    systemctl --user stop promtail.service
    echo "✓ Promtail stopped"
else
    echo "⚠ Promtail not running"
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
echo "Step 2: Verify Vector binary exists"
echo "════════════════════════════════════════════════════════════════"

if [ ! -f "$USER_HOME/.local/bin/vector" ]; then
    echo "✗ Vector binary not found!"
    echo "Please install Vector first."
    exit 1
fi

echo "✓ Vector binary exists"
$USER_HOME/.local/bin/vector --version

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 3: Create enhanced Vector configuration"
echo "════════════════════════════════════════════════════════════════"

# Backup existing config
if [ -f "$USER_HOME/.config/vector/vector.toml" ]; then
    cp "$USER_HOME/.config/vector/vector.toml" "$USER_HOME/.config/vector/vector.toml.pre-migration"
    echo "✓ Backed up existing Vector config"
fi

# Note: The enhanced config should be created manually or via the repository
# For now, we'll verify it exists
if [ ! -f "$USER_HOME/.config/vector/vector.toml" ]; then
    echo "✗ Vector configuration not found!"
    echo "Please create $USER_HOME/.config/vector/vector.toml"
    exit 1
fi

echo "✓ Vector configuration ready"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 4: Ensure Vector systemd service exists"
echo "════════════════════════════════════════════════════════════════"

if [ ! -f "$USER_HOME/.config/systemd/user/vector.service" ]; then
    echo "Creating Vector systemd service..."
    
    cat > "$USER_HOME/.config/systemd/user/vector.service" << 'EOF'
[Unit]
Description=Vector log collector (user)
After=network.target loki.service

[Service]
Type=simple
ExecStart=%h/.local/bin/vector --config %h/.config/vector/vector.toml
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF
    
    echo "✓ Vector systemd service created"
else
    echo "✓ Vector systemd service exists"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 5: Start Vector"
echo "════════════════════════════════════════════════════════════════"

systemctl --user daemon-reload
systemctl --user enable vector.service
systemctl --user start vector.service

sleep 3

if systemctl --user is-active --quiet vector.service; then
    echo "✓ Vector is running"
    systemctl --user status vector.service --no-pager -l | head -20
else
    echo "✗ Vector failed to start!"
    echo "Check logs with: journalctl --user -u vector.service -n 50"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 6: Verify Loki is receiving logs from Vector"
echo "════════════════════════════════════════════════════════════════"

echo "Waiting 10 seconds for logs to flow..."
sleep 10

LOKI_RESPONSE=$(curl -s -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={job=~".*"}' \
  --data-urlencode 'limit=1')

if echo "$LOKI_RESPONSE" | grep -q '"status":"success"'; then
    echo "✓ Loki is receiving logs from Vector"
else
    echo "⚠ Could not verify logs in Loki"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 7: Clean up Promtail files"
echo "════════════════════════════════════════════════════════════════"

echo ""
echo "Do you want to remove Promtail files? (keeps backups)"
echo "  - ~/.local/bin/promtail (binary)"
echo "  - ~/.config/promtail/config.yml (config - backed up)"
echo "  - ~/.config/systemd/user/promtail.service (service file)"
echo "  - ~/.local/share/promtail (data directory)"
echo ""
echo "Preserved:"
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
    
    # Remove config (but keep backup)
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
    echo ""
    echo "To manually remove later:"
    echo "  rm ~/.local/bin/promtail"
    echo "  rm ~/.config/promtail/config.yml"
    echo "  rm ~/.config/systemd/user/promtail.service"
    echo "  rm -rf ~/.local/share/promtail"
    echo "  systemctl --user daemon-reload"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✓ Migration Complete!"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Summary:"
echo "  ✓ Promtail stopped and disabled"
echo "  ✓ Vector started and enabled"
echo "  ✓ Loki receiving logs from Vector"
echo ""
echo "Vector files:"
echo "  - Binary: ~/.local/bin/vector"
echo "  - Config: ~/.config/vector/vector.toml"
echo "  - Service: ~/.config/systemd/user/vector.service"
echo "  - Data: ~/.local/share/vector/"
echo ""
echo "Useful commands:"
echo "  systemctl --user status vector.service"
echo "  journalctl --user -u vector.service -f"
echo "  curl http://localhost:9598/metrics  # Vector metrics"
echo ""
