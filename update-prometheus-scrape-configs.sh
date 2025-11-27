#!/bin/bash
# Update Prometheus scrape configs to use HTTPS

set -e

echo "🔧 Updating Prometheus scrape configurations for TLS..."
echo ""

PROM_CONFIG="/etc/prometheus/prometheus.yml"
BACKUP_FILE="/etc/prometheus/prometheus.yml.backup-before-tls"

echo "1. Backing up current configuration..."
sudo cp "$PROM_CONFIG" "$BACKUP_FILE"
echo "   ✓ Backup created: $BACKUP_FILE"

echo ""
echo "2. Creating new configuration with HTTPS..."
sudo tee "$PROM_CONFIG" > /dev/null <<'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  # Prometheus itself (now with HTTPS)
  - job_name: 'prometheus'
    scheme: https
    tls_config:
      insecure_skip_verify: true  # Self-signed cert
    static_configs:
      - targets: ['localhost:9090']

  # Node exporter (HTTP for now)
  - job_name: 'node'
    static_configs:
      - targets: ['localhost:9100']

  # Agentic RAG backend (HTTP)
  - job_name: 'agentic-rag'
    static_configs:
      - targets: ['localhost:3010']
    metrics_path: '/monitoring/metrics'
    scrape_interval: 5s

  # OpenTelemetry Collector (HTTP)
  - job_name: 'otelcol'
    static_configs:
      - targets: ['localhost:8888']
    scrape_interval: 10s

  # Vector (HTTP)
  - job_name: 'vector'
    static_configs:
      - targets: ['localhost:9598']
    scrape_interval: 15s

  # Loki (HTTP)
  - job_name: 'loki'
    static_configs:
      - targets: ['localhost:3100']
    metrics_path: '/metrics'
    scrape_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['localhost:9093']

rule_files:
  - "/etc/prometheus/alerts/*.yaml"
EOF

echo "   ✓ Configuration updated"

echo ""
echo "3. Validating configuration..."
if sudo promtool check config "$PROM_CONFIG" 2>&1 | grep -q "SUCCESS"; then
    echo "   ✓ Configuration is valid"
else
    echo "   ✗ Configuration validation failed!"
    echo "   Restoring backup..."
    sudo cp "$BACKUP_FILE" "$PROM_CONFIG"
    exit 1
fi

echo ""
echo "4. Reloading Prometheus..."
sudo systemctl reload prometheus || sudo systemctl restart prometheus

sleep 2

echo ""
echo "5. Checking Prometheus status..."
if systemctl is-active --quiet prometheus; then
    echo "   ✓ Prometheus is running"
else
    echo "   ✗ Prometheus failed to start!"
    systemctl status prometheus --no-pager
    exit 1
fi

echo ""
echo "✅ Scrape configurations updated!"
echo ""
echo "📊 Prometheus is now scraping itself via HTTPS"
echo "   Other targets remain on HTTP (can be updated later)"
echo ""
