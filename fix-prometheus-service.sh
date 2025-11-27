#!/bin/bash
# Fix Prometheus service file

set -e

echo "🔧 Fixing Prometheus service file..."
echo ""

echo "1. Creating corrected service file..."
sudo tee /etc/systemd/system/prometheus.service > /dev/null <<'EOF'
[Unit]
Description=Prometheus
Wants=network-online.target
After=network-online.target

[Service]
User=prometheus
Group=prometheus
Type=simple
ExecStart=/usr/local/bin/prometheus \
  --config.file=/etc/prometheus/prometheus.yml \
  --storage.tsdb.path=/var/lib/prometheus/ \
  --web.console.templates=/etc/prometheus/consoles \
  --web.console.libraries=/etc/prometheus/console_libraries \
  --web.config.file=/etc/prometheus/tls/web-config.yml

Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

echo "   ✓ Service file corrected"

echo ""
echo "2. Reloading systemd daemon..."
sudo systemctl daemon-reload
echo "   ✓ Daemon reloaded"

echo ""
echo "3. Restarting Prometheus..."
sudo systemctl restart prometheus

sleep 3

echo ""
echo "4. Checking Prometheus status..."
if systemctl is-active --quiet prometheus; then
    echo "   ✓ Prometheus is running!"
    systemctl status prometheus --no-pager | head -10
else
    echo "   ✗ Prometheus failed to start"
    echo ""
    echo "Checking logs:"
    journalctl -u prometheus -n 20 --no-pager
    exit 1
fi

echo ""
echo "5. Testing HTTPS endpoint..."
sleep 2
if curl -k -s https://localhost:9090/-/healthy > /dev/null 2>&1; then
    echo "   ✓ HTTPS endpoint is responding!"
    curl -k https://localhost:9090/-/healthy
else
    echo "   ✗ HTTPS endpoint not responding"
    echo "   Checking if port is open..."
    ss -tlnp | grep 9090 || echo "Port 9090 not listening"
fi

echo ""
echo "✅ Fix complete!"
echo ""
