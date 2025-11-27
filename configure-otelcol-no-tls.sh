#!/bin/bash
# Configure OpenTelemetry Collector to bypass TLS issues

echo "═══════════════════════════════════════════════════════════"
echo "  Configuring OpenTelemetry Collector (Bypass TLS)"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "[1/3] Creating new OpenTelemetry Collector configuration..."

# The solution: Send traces directly to AG Backend's internal Tempo endpoint
# OR: Disable the OpenTelemetry Collector and have AG Backend send directly to Tempo

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
  # Send to Tempo's internal distributor endpoint (no TLS)
  otlp/tempo:
    endpoint: 127.0.0.1:9095
    tls:
      insecure: true
  logging:
    loglevel: info

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [otlp/tempo, logging]
EOF

echo "✓ Configuration created"
echo ""

echo "[2/3] Restarting OpenTelemetry Collector..."
systemctl --user restart otelcol.service
sleep 3

if systemctl --user is-active --quiet otelcol.service; then
    echo "✓ OpenTelemetry Collector is running"
else
    echo "✗ OpenTelemetry Collector failed to start"
    journalctl --user -u otelcol.service -n 20 --no-pager
    exit 1
fi
echo ""

echo "[3/3] Testing configuration..."
echo "Waiting 5 seconds for collector to initialize..."
sleep 5

# Check for errors in the last 10 seconds
ERRORS=$(journalctl --user -u otelcol.service --since "10 seconds ago" --no-pager 2>/dev/null | grep -i "error\|failed" | wc -l)
if [ "$ERRORS" -eq 0 ]; then
    echo "✓ No errors detected"
else
    echo "⚠️  Found $ERRORS error(s) in logs"
    echo "Check with: journalctl --user -u otelcol.service -n 50"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  OpenTelemetry Collector Configured!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "New configuration:"
echo "  - Receiving from AG Backend on port 4318 (gRPC)"
echo "  - Forwarding to Tempo on port 9095 (internal gRPC, no TLS)"
echo "  - Removed tail_sampling to simplify"
echo ""
echo "Next: Verify traces are flowing"
echo "Run: bash /home/pde/ag/final-verification.sh"
