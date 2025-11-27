#!/bin/bash
# Script to update Tempo configuration with plaintext gRPC receiver

echo "═══════════════════════════════════════════════════════════"
echo "  Updating Tempo Configuration for Distributed Tracing"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Backup original config
echo "[1/4] Backing up original Tempo configuration..."
sudo cp /etc/tempo/config.yml /etc/tempo/config.yml.backup-before-plaintext
echo "✓ Backup created: /etc/tempo/config.yml.backup-before-plaintext"
echo ""

# Copy updated config
echo "[2/4] Installing updated Tempo configuration..."
sudo cp /tmp/tempo-config-updated.yml /etc/tempo/config.yml
echo "✓ Configuration updated"
echo ""

# Restart Tempo
echo "[3/4] Restarting Tempo service..."
sudo systemctl restart tempo
sleep 3
echo "✓ Tempo restarted"
echo ""

# Check Tempo status
echo "[4/4] Checking Tempo status..."
if sudo systemctl is-active --quiet tempo; then
    echo "✓ Tempo is running"
else
    echo "✗ Tempo failed to start - check logs with: sudo journalctl -u tempo -n 50"
    exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  Tempo Configuration Complete!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Tempo is now listening on:"
echo "  - Port 4317 (gRPC with TLS)"
echo "  - Port 4320 (gRPC plaintext) ← NEW"
echo ""
echo "Next step: Update OpenTelemetry Collector configuration"
echo "Run: bash /home/pde/ag/update-otelcol-config.sh"
