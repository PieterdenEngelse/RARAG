#!/bin/bash
# Fix tempo.service binary path issue

set -e

echo "🔧 Fixing tempo.service..."
echo ""

# Check if tempo binary exists
echo "1. Checking tempo binary location..."
if [ -f "/usr/local/bin/tempo" ]; then
    echo "   ✅ Found tempo at /usr/local/bin/tempo"
else
    echo "   ❌ Tempo binary not found at /usr/local/bin/tempo"
    echo "   Searching for tempo..."
    find /usr -name tempo 2>/dev/null || echo "   No tempo binary found"
    exit 1
fi

echo ""
echo "2. Backing up current service file..."
sudo cp /etc/systemd/system/tempo.service /etc/systemd/system/tempo.service.backup
echo "   ✅ Backup created"

echo ""
echo "3. Updating service file..."
sudo cp ./tempo.service.fixed /etc/systemd/system/tempo.service
echo "   ✅ Service file updated"

echo ""
echo "4. Reloading systemd daemon..."
sudo systemctl daemon-reload
echo "   ✅ Daemon reloaded"

echo ""
echo "5. Restarting tempo service..."
sudo systemctl restart tempo.service
sleep 2

echo ""
echo "6. Checking tempo status..."
if systemctl is-active --quiet tempo.service; then
    echo "   ✅ Tempo is running!"
    systemctl status tempo.service --no-pager | head -10
else
    echo "   ⚠️  Tempo status:"
    systemctl status tempo.service --no-pager | head -15
fi

echo ""
echo "✅ Fix complete!"
echo ""
echo "📊 Verify in Grafana:"
echo "   The system-errors panel should stop showing tempo errors"
echo ""
