#!/bin/bash
# Fix Tempo TLS certificate permissions

set -e

echo "🔧 Fixing Tempo TLS certificate permissions..."
echo ""

CERT_DIR="/etc/tempo/tls"

echo "1. Checking certificate files..."
if [ -f "$CERT_DIR/tempo.crt" ] && [ -f "$CERT_DIR/tempo.key" ]; then
    echo "   ✓ Certificate files exist"
else
    echo "   ✗ Certificate files not found!"
    exit 1
fi

echo ""
echo "2. Setting ownership to tempo user..."
sudo chown tempo:nogroup "$CERT_DIR/tempo.crt"
sudo chown tempo:nogroup "$CERT_DIR/tempo.key"
echo "   ✓ Ownership set to tempo:nogroup"

echo ""
echo "3. Setting correct permissions..."
sudo chmod 644 "$CERT_DIR/tempo.crt"
sudo chmod 600 "$CERT_DIR/tempo.key"
echo "   ✓ Permissions set (crt: 644, key: 600)"

echo ""
echo "4. Verifying permissions..."
ls -l "$CERT_DIR/"
echo ""

echo "5. Restarting Tempo..."
sudo systemctl restart tempo

sleep 3

echo ""
echo "6. Checking Tempo status..."
if systemctl is-active --quiet tempo; then
    echo "   ✓ Tempo is running!"
    systemctl status tempo --no-pager | head -10
else
    echo "   ✗ Tempo failed to start"
    echo ""
    echo "Checking logs:"
    journalctl -u tempo -n 20 --no-pager
    exit 1
fi

echo ""
echo "7. Testing HTTPS endpoint..."
sleep 2
if curl -k -s https://localhost:3200/ready > /dev/null 2>&1; then
    echo "   ✓ HTTPS endpoint is responding!"
    curl -k https://localhost:3200/ready
else
    echo "   ✗ HTTPS endpoint not responding"
fi

echo ""
echo "✅ Tempo TLS permissions fixed!"
echo ""
