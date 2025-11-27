#!/bin/bash

echo "═══════════════════════════════════════════════════════════"
echo "  AG Backend Distributed Tracing Verification"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "[1/6] Checking AG Backend Status..."
if pgrep -f "target/release/ag" > /dev/null; then
    echo "✓ AG Backend is running"
    AG_PID=$(pgrep -f "target/release/ag" | head -n 1)
    echo "  PID: $AG_PID"
else
    echo "✗ AG Backend is NOT running"
    exit 1
fi
echo ""

echo "[2/6] Checking OTEL Configuration..."
echo "Current .env settings:"
grep "^OTEL_" /home/pde/ag/.env
echo ""

echo "[3/6] Testing AG Backend Health..."
HEALTH_RESPONSE=$(curl -s http://localhost:3010/monitoring/health)
if [ $? -eq 0 ]; then
    echo "✓ AG Backend is responding"
    echo "  Response: $HEALTH_RESPONSE"
else
    echo "✗ AG Backend is not responding"
fi
echo ""

echo "[4/6] Checking Tempo Status..."
if systemctl is-active --quiet tempo; then
    echo "✓ Tempo is running"
else
    echo "✗ Tempo is NOT running"
fi
echo ""

echo "[5/6] Checking Tempo OTLP Ports..."
echo "Port 4317 (gRPC):"
ss -tlnp | grep 4317 || echo "  Not listening"
echo "Port 4318 (HTTP):"
ss -tlnp | grep 4318 || echo "  Not listening"
echo ""

echo "[6/6] Checking Tempo Trace Metrics..."
TRACES=$(curl -sk https://localhost:3200/metrics 2>/dev/null | grep "tempo_ingester_traces_created_total{tenant=\"single-tenant\"}" | awk '{print $2}')
if [ -n "$TRACES" ]; then
    echo "Traces created: $TRACES"
    if [ "$TRACES" -gt 0 ]; then
        echo "✓ Traces are being ingested!"
    else
        echo "⚠️ No traces ingested yet"
    fi
else
    echo "✗ Could not retrieve trace metrics"
fi
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "  Summary"
echo "═══════════════════════════════════════════════════════════"
echo ""
if [ "$TRACES" -gt 0 ] 2>/dev/null; then
    echo "✅ SUCCESS: Distributed tracing is working!"
    echo ""
    echo "Next steps:"
    echo "1. Add Tempo datasource in Grafana: https://localhost:3200"
    echo "2. Create Tempo dashboards to visualize traces"
    echo "3. Explore traces in Grafana Explore view"
else
    echo "⚠️ INCOMPLETE: Tracing is configured but not yet working"
    echo ""
    echo "Troubleshooting:"
    echo "1. Check AG_TRACING_SETUP_SUMMARY.md for detailed steps"
    echo "2. Try using HTTP endpoint (port 4318) instead of gRPC (4317)"
    echo "3. Check AG backend logs for OTLP errors:"
    echo "   journalctl _PID=$AG_PID -n 50 --no-pager | grep -i otlp"
fi
echo ""
