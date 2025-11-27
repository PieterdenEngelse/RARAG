#!/bin/bash
# Fix Tempo's internal gRPC TLS configuration to allow plaintext connections

echo "═══════════════════════════════════════════════════════════"
echo "  Fixing Tempo Internal gRPC Configuration"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "[1/3] Backing up current Tempo configuration..."
sudo cp /etc/tempo/config.yml /etc/tempo/config.yml.backup-internal-grpc-$(date +%Y%m%d-%H%M%S)
echo "✓ Backup created"
echo ""

echo "[2/3] Updating Tempo configuration..."
# The fix: Remove grpc_tls_config to allow plaintext internal gRPC connections
# Keep http_tls_config for external HTTPS API
sudo tee /etc/tempo/config.yml > /dev/null << 'EOF'
stream_over_http_enabled: true
server:
  http_listen_port: 3200
  log_level: info
  # TLS Configuration for HTTP API only
  http_tls_config:
    cert_file: /etc/tempo/tls/tempo.crt
    key_file: /etc/tempo/tls/tempo.key
  # Internal gRPC (port 9095) - NO TLS for internal communication

query_frontend:
  search:
    duration_slo: 5s
    throughput_bytes_slo: 1.073741824e+09
    metadata_slo:
        duration_slo: 5s
        throughput_bytes_slo: 1.073741824e+09
  trace_by_id:
    duration_slo: 5s

distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: "0.0.0.0:4317"
          # TLS removed for OTLP receiver to allow plaintext connections

metrics_generator:
  registry:
    external_labels:
      source: tempo
      cluster: docker-compose
  storage:
    path: /var/tempo/generator/wal
    remote_write:
      - url: https://localhost:9090/api/v1/write
        send_exemplars: true
        tls_config:
          insecure_skip_verify: true
  traces_storage:
    path: /var/tempo/generator/traces

storage:
  trace:
    backend: local
    wal:
      path: /var/tempo/wal
    local:
      path: /var/tempo/blocks

overrides:
  defaults:
    metrics_generator:
      processors: [service-graphs, span-metrics, local-blocks]
      generate_native_histograms: both
EOF

echo "✓ Configuration updated"
echo ""

echo "[3/3] Restarting Tempo..."
sudo systemctl restart tempo
sleep 5

if sudo systemctl is-active --quiet tempo; then
    echo "✓ Tempo is running"
else
    echo "✗ Tempo failed to start"
    echo "Checking logs..."
    sudo journalctl -u tempo -n 20 --no-pager
    exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  Tempo Internal gRPC Configuration Fixed!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Configuration changes:"
echo "  - Removed grpc_tls_config from server section"
echo "  - Internal gRPC (port 9095) now uses plaintext"
echo "  - HTTP API (port 3200) still uses HTTPS"
echo "  - OTLP receiver (port 4317) uses plaintext"
echo ""
echo "Next steps:"
echo "  1. Make test requests to AG backend"
echo "  2. Check Tempo metrics for trace ingestion"
echo "  3. Verify traces are being stored"
echo ""
