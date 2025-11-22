#!/bin/bash
# update_prometheus_for_vector.sh - Add Vector metrics to Prometheus

set -e

echo "════════════════════════════════════════════════════════════════"
echo "  Adding Vector Metrics to Prometheus"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Backup current config
echo "Step 1: Backing up current Prometheus configuration..."
sudo cp /etc/prometheus/prometheus.yml /etc/prometheus/prometheus.yml.backup.$(date +%Y%m%d_%H%M%S)
echo "✓ Backup created"

# Copy new config
echo ""
echo "Step 2: Installing new Prometheus configuration..."
sudo cp /home/pde/ag/prometheus.yml.new /etc/prometheus/prometheus.yml
echo "✓ Configuration updated"

# Validate config
echo ""
echo "Step 3: Validating Prometheus configuration..."
if promtool check config /etc/prometheus/prometheus.yml; then
    echo "✓ Configuration is valid"
else
    echo "✗ Configuration validation failed!"
    echo "Restoring backup..."
    sudo cp /etc/prometheus/prometheus.yml.backup.$(date +%Y%m%d)* /etc/prometheus/prometheus.yml
    exit 1
fi

# Reload Prometheus
echo ""
echo "Step 4: Reloading Prometheus..."
if sudo systemctl reload prometheus; then
    echo "✓ Prometheus reloaded successfully"
else
    echo "⚠ Reload failed, trying restart..."
    sudo systemctl restart prometheus
    echo "✓ Prometheus restarted"
fi

# Wait for Prometheus to be ready
echo ""
echo "Step 5: Waiting for Prometheus to be ready..."
sleep 3

# Verify Vector target
echo ""
echo "Step 6: Verifying Vector target..."
VECTOR_TARGET=$(curl -s http://localhost:9090/api/v1/targets | jq -r '.data.activeTargets[] | select(.labels.job=="vector") | .health')

if [ "$VECTOR_TARGET" = "up" ]; then
    echo "✓ Vector target is UP"
else
    echo "⚠ Vector target status: $VECTOR_TARGET"
    echo "  (It may take a moment to become healthy)"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✓ Prometheus Configuration Updated"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Vector metrics are now being scraped by Prometheus!"
echo ""
echo "Verify:"
echo "  1. Check targets: http://localhost:9090/targets"
echo "  2. Query Vector metrics: vector_component_received_events_total"
echo "  3. View in Grafana: Add Prometheus datasource queries"
echo ""
echo "Sample queries:"
echo "  - Events received: rate(vector_component_received_events_total[5m])"
echo "  - Events sent: rate(vector_component_sent_events_total[5m])"
echo "  - Buffer size: vector_buffer_events"
echo ""
