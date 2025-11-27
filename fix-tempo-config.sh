#!/bin/bash
# Fix Tempo configuration - restore original and use different approach

echo "═══════════════════════════════════════════════════════════"
echo "  Fixing Tempo Configuration"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "[1/2] Restoring original Tempo configuration..."
sudo cp /etc/tempo/config.yml.backup-before-plaintext /etc/tempo/config.yml
if [ $? -eq 0 ]; then
    echo "✓ Original configuration restored"
else
    echo "✗ Failed to restore configuration"
    exit 1
fi
echo ""

echo "[2/2] Restarting Tempo service..."
sudo systemctl restart tempo
sleep 3

if sudo systemctl is-active --quiet tempo; then
    echo "✓ Tempo is running"
else
    echo "✗ Tempo failed to start"
    sudo journalctl -u tempo -n 20 --no-pager
    exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  Tempo Configuration Fixed!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Tempo is back to original configuration:"
echo "  - Port 4317 (gRPC with TLS)"
echo ""
echo "Next: We'll use a different approach - disable TLS in OpenTelemetry Collector"
echo "Run: bash /home/pde/ag/configure-otelcol-no-tls.sh"
