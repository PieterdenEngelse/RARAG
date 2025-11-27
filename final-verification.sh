#!/bin/bash
# Final verification script for distributed tracing

echo "═══════════════════════════════════════════════════════════"
echo "  Final Distributed Tracing Verification"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Step 1: Generate test traces
echo "[1/5] Generating test traces..."
echo "Making requests to AG Backend..."
for i in {1..5}; do
    curl -s http://localhost:3010/monitoring/health > /dev/null && echo "  ✓ Request $i sent"
    sleep 1
done
echo ""

# Step 2: Wait for batch export
echo "[2/5] Waiting for traces to be batched and exported (10 seconds)..."
sleep 10
echo "✓ Wait complete"
echo ""

# Step 3: Check Tempo port
echo "[3/5] Checking Tempo ports..."
echo "Port 4317 (gRPC with TLS):"
ss -tlnp 2>/dev/null | grep 4317 || echo "  Not listening"
echo "Port 4320 (gRPC plaintext):"
ss -tlnp 2>/dev/null | grep 4320 || echo "  Not listening"
echo ""

# Step 4: Check Tempo metrics
echo "[4/5] Checking Tempo trace ingestion metrics..."
TRACES=$(curl -sk https://localhost:3200/metrics 2>/dev/null | grep 'tempo_ingester_traces_created_total{tenant="single-tenant"}' | awk '{print $2}')

if [ -n "$TRACES" ]; then
    echo "Traces created: $TRACES"
    if [ "$TRACES" -gt 0 ] 2>/dev/null; then
        echo "✅ SUCCESS: Traces are being ingested by Tempo!"
    else
        echo "⚠️  No traces ingested yet"
    fi
else
    echo "⚠️  Could not retrieve trace metrics"
fi
echo ""

# Step 5: Check OpenTelemetry Collector logs
echo "[5/5] Checking OpenTelemetry Collector for errors..."
ERRORS=$(journalctl --user -u otelcol.service --since "1 minute ago" --no-pager 2>/dev/null | grep -i "error\|failed" | wc -l)
if [ "$ERRORS" -eq 0 ]; then
    echo "✓ No errors in OpenTelemetry Collector logs"
else
    echo "⚠️  Found $ERRORS error(s) in OpenTelemetry Collector logs"
    echo "Check with: journalctl --user -u otelcol.service -n 50"
fi
echo ""

# Final summary
echo "═══════════════════════════════════════════════════════════"
echo "  Summary"
echo "═══════════════════════════════════════════════════════════"
echo ""

if [ -n "$TRACES" ] && [ "$TRACES" -gt 0 ] 2>/dev/null; then
    echo "✅ SUCCESS: Distributed tracing is fully operational!"
    echo ""
    echo "🎉 Congratulations! Your distributed tracing pipeline is working:"
    echo ""
    echo "  AG Backend (port 3010)"
    echo "       ↓ gRPC (port 4318)"
    echo "  OpenTelemetry Collector"
    echo "       ↓ gRPC plaintext (port 4320)"
    echo "  Tempo (port 4320)"
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
    echo "3. Create dashboards to visualize:"
    echo "   - Request latency"
    echo "   - Error rates"
    echo "   - Service dependencies"
    echo ""
else
    echo "⚠️  Distributed tracing is not yet fully working"
    echo ""
    echo "Troubleshooting steps:"
    echo "1. Check if Tempo is listening on port 4320:"
    echo "   ss -tlnp | grep 4320"
    echo ""
    echo "2. Check Tempo logs:"
    echo "   sudo journalctl -u tempo -n 50"
    echo ""
    echo "3. Check OpenTelemetry Collector logs:"
    echo "   journalctl --user -u otelcol.service -n 50"
    echo ""
    echo "4. Verify Tempo configuration:"
    echo "   grep -A 5 'grpc_plaintext' /etc/tempo/config.yml"
    echo ""
fi
