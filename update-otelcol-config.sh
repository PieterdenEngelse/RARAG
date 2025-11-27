#!/bin/bash
# Script to restart OpenTelemetry Collector with updated configuration

echo "═══════════════════════════════════════════════════════════"
echo "  Restarting OpenTelemetry Collector"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "[1/2] Restarting OpenTelemetry Collector service..."
systemctl --user restart otelcol.service
sleep 3
echo "✓ OpenTelemetry Collector restarted"
echo ""

echo "[2/2] Checking service status..."
if systemctl --user is-active --quiet otelcol.service; then
    echo "✓ OpenTelemetry Collector is running"
else
    echo "✗ OpenTelemetry Collector failed to start"
    echo "Check logs with: journalctl --user -u otelcol.service -n 50"
    exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  OpenTelemetry Collector Configuration Complete!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Configuration:"
echo "  - Receiving traces from AG Backend on port 4318 (gRPC)"
echo "  - Forwarding traces to Tempo on port 4320 (plaintext gRPC)"
echo ""
echo "Next step: Verify distributed tracing is working"
echo "Run: bash /home/pde/ag/final-verification.sh"
