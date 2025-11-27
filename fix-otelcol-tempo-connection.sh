#!/bin/bash
# Fix OpenTelemetry Collector to forward traces to Tempo's OTLP endpoint

echo "═══════════════════════════════════════════════════════════"
echo "  Fixing OpenTelemetry Collector → Tempo Connection"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "[1/4] Backing up current configuration..."
cp ~/.config/otelcol/config.yaml ~/.config/otelcol/config.yaml.backup-$(date +%Y%m%d-%H%M%S)
echo "✓ Backup created"
echo ""

echo "[2/4] Creating new configuration..."
# The fix: Change from port 9095 to port 4317 (Tempo's OTLP gRPC endpoint)
cat > ~/.config/otelcol/config.yaml << 'EOF'
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 127.0.0.1:4318
      http:
        endpoint: 127.0.0.1:4319

processors:
  batch:
    send_batch_size: 512
    timeout: 5s

exporters:
  # Send to Tempo's OTLP gRPC endpoint (port 4317)
  otlp/tempo:
    endpoint: localhost:4317
    tls:
      insecure: true
  logging:
    loglevel: debug

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [otlp/tempo, logging]
EOF

echo "✓ Configuration updated"
echo ""

echo "[3/4] Restarting OpenTelemetry Collector..."
systemctl --user restart otelcol.service
sleep 3

if systemctl --user is-active --quiet otelcol.service; then
    echo "✓ OpenTelemetry Collector is running"
else
    echo "✗ OpenTelemetry Collector failed to start"
    echo "Checking logs..."
    journalctl --user -u otelcol.service -n 20 --no-pager
    exit 1
fi
echo ""

echo "[4/4] Testing the connection..."
echo "Waiting 5 seconds for collector to initialize..."
sleep 5

# Make a test request to AG backend to generate traces
echo "Making test request to AG backend..."
curl -s http://127.0.0.1:3010/monitoring/health > /dev/null
sleep 2

# Check for errors in the last 10 seconds
ERRORS=$(journalctl --user -u otelcol.service --since "10 seconds ago" --no-pager 2>/dev/null | grep -i "error\|failed" | grep -v "reading server preface" | wc -l)
if [ "$ERRORS" -eq 0 ]; then
    echo "✓ No errors detected"
else
    echo "⚠️  Found $ERRORS error(s) in logs"
    echo "Recent logs:"
    journalctl --user -u otelcol.service -n 20 --no-pager
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  OpenTelemetry Collector Configuration Fixed!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Configuration changes:"
echo "  - Changed Tempo endpoint from port 9095 to port 4317"
echo "  - Port 4317 is Tempo's OTLP gRPC receiver"
echo "  - Enabled debug logging to see trace flow"
echo ""
echo "Next steps:"
echo "  1. Make some requests to AG backend to generate traces"
echo "  2. Check Tempo metrics: curl -sk https://localhost:3200/metrics | grep tempo_ingester_traces_created_total"
echo "  3. Search for traces: curl -sk 'https://localhost:3200/api/search?tags=service.name%3Dag-backend'"
echo ""
