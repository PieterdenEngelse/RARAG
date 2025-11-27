#!/bin/bash
# Complete Distributed Tracing Setup - Final Steps

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Completing Distributed Tracing Setup                    ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Step 1: Update Tempo configuration
echo "[1/6] Updating Tempo configuration..."
echo "Backing up current config..."
sudo cp /etc/tempo/config.yml /etc/tempo/config.yml.backup-before-no-tls

echo "Installing new config (TLS removed from OTLP receiver)..."
sudo cp /tmp/tempo-config-no-tls.yml /etc/tempo/config.yml

if [ $? -eq 0 ]; then
    echo "✓ Tempo configuration updated"
else
    echo "✗ Failed to update Tempo configuration"
    exit 1
fi
echo ""

# Step 2: Restart Tempo
echo "[2/6] Restarting Tempo service..."
sudo systemctl restart tempo
sleep 3

if sudo systemctl is-active --quiet tempo; then
    echo "✓ Tempo is running"
else
    echo "✗ Tempo failed to start"
    echo "Check logs: sudo journalctl -u tempo -n 50"
    exit 1
fi
echo ""

# Step 3: Update AG Backend .env
echo "[3/6] Updating AG Backend configuration..."
sed -i 's|OTEL_EXPORTER_OTLP_ENDPOINT=https://localhost:4317|OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317|' /home/pde/ag/.env

if grep -q "OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317" /home/pde/ag/.env; then
    echo "✓ AG Backend .env updated (using http://)"
else
    echo "✗ Failed to update AG Backend .env"
    exit 1
fi
echo ""

# Step 4: Restart AG Backend
echo "[4/6] Restarting AG Backend..."
pkill -9 -f "target/release/ag"
sleep 3
tmux send-keys -t main:5 "cd /home/pde/ag && ./target/release/ag" C-m
sleep 5

if pgrep -f "target/release/ag" > /dev/null; then
    echo "✓ AG Backend is running"
else
    echo "✗ AG Backend failed to start"
    exit 1
fi
echo ""

# Step 5: Generate test traces
echo "[5/6] Generating test traces..."
for i in {1..5}; do
    curl -s http://localhost:3010/monitoring/health > /dev/null && echo "  ✓ Request $i sent"
    sleep 1
done
echo ""

echo "Waiting 10 seconds for traces to be batched and exported..."
sleep 10
echo ""

# Step 6: Verify traces are flowing
echo "[6/6] Verifying traces are flowing to Tempo..."
TRACES=$(curl -sk https://localhost:3200/metrics 2>/dev/null | grep 'tempo_ingester_traces_created_total{tenant="single-tenant"}' | awk '{print $2}')

if [ -n "$TRACES" ] && [ "$TRACES" -gt 0 ] 2>/dev/null; then
    echo "✅ SUCCESS! Traces are flowing to Tempo!"
    echo "   Traces created: $TRACES"
else
    echo "⚠️  Traces not detected yet. Checking status..."
    TRACES_LINE=$(curl -sk https://localhost:3200/metrics 2>/dev/null | grep tempo_ingester_traces_created_total | head -1)
    echo "   Metric: $TRACES_LINE"
fi
echo ""

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Setup Complete!                                          ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

if [ -n "$TRACES" ] && [ "$TRACES" -gt 0 ] 2>/dev/null; then
    echo "🎉 Congratulations! Distributed tracing is now operational!"
    echo ""
    echo "Architecture:"
    echo "  AG Backend (port 3010)"
    echo "       ↓ gRPC (no TLS)"
    echo "  Tempo OTLP Receiver (port 4317)"
    echo "       ↓"
    echo "  Tempo Storage"
    echo ""
    echo "Next steps:"
    echo "1. Add Tempo datasource in Grafana:"
    echo "   URL: https://localhost:3200"
    echo "   Skip TLS Verify: Yes"
    echo ""
    echo "2. Explore traces in Grafana:"
    echo "   - Go to Explore"
    echo "   - Select Tempo datasource"
    echo "   - Search for service.name = \"ag-backend\""
    echo ""
    echo "3. Make more requests to generate traces:"
    echo "   curl http://localhost:3010/monitoring/health"
    echo "   curl http://localhost:3010/documents"
else
    echo "⚠️  Traces may still be initializing. Try:"
    echo "1. Make more requests:"
    echo "   curl http://localhost:3010/monitoring/health"
    echo ""
    echo "2. Wait 10 more seconds and check again:"
    echo "   curl -sk https://localhost:3200/metrics | grep tempo_ingester_traces_created_total"
    echo ""
    echo "3. Check AG Backend logs:"
    echo "   tmux capture-pane -t main:5 -p | tail -n 20"
    echo ""
    echo "4. Check Tempo logs:"
    echo "   sudo journalctl -u tempo -n 20 --no-pager"
fi
