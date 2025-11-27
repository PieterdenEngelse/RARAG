#!/bin/bash
# Setup TLS for Tempo

set -e

echo "🔐 Setting up TLS for Tempo..."
echo ""

# Configuration
CERT_DIR="/etc/tempo/tls"
CERT_VALIDITY_DAYS=3650  # 10 years
TEMPO_CONFIG="/etc/tempo/config.yml"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "1. Creating TLS certificate directory..."
sudo mkdir -p "$CERT_DIR"
echo -e "   ${GREEN}✓${NC} Directory created: $CERT_DIR"

echo ""
echo "2. Generating self-signed TLS certificate..."
sudo openssl req -new -newkey rsa:2048 -days $CERT_VALIDITY_DAYS -nodes -x509 \
    -keyout "$CERT_DIR/tempo.key" \
    -out "$CERT_DIR/tempo.crt" \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=tempo.local" \
    -addext "subjectAltName=DNS:localhost,DNS:tempo.local,IP:127.0.0.1"

echo -e "   ${GREEN}✓${NC} Certificate generated"
echo "   - Certificate: $CERT_DIR/tempo.crt"
echo "   - Private Key: $CERT_DIR/tempo.key"
echo "   - Validity: $CERT_VALIDITY_DAYS days"

echo ""
echo "3. Setting correct permissions..."
sudo chown tempo:tempo "$CERT_DIR"/*.{crt,key} 2>/dev/null || true
sudo chmod 600 "$CERT_DIR/tempo.key"
sudo chmod 644 "$CERT_DIR/tempo.crt"
echo -e "   ${GREEN}✓${NC} Permissions set"

echo ""
echo "4. Backing up current Tempo configuration..."
sudo cp "$TEMPO_CONFIG" "$TEMPO_CONFIG.backup-before-tls"
echo -e "   ${GREEN}✓${NC} Backup created: $TEMPO_CONFIG.backup-before-tls"

echo ""
echo "5. Updating Tempo configuration with TLS..."
sudo tee "$TEMPO_CONFIG" > /dev/null <<'EOF'
stream_over_http_enabled: true
server:
  http_listen_port: 3200
  log_level: info
  # TLS Configuration
  http_tls_config:
    cert_file: /etc/tempo/tls/tempo.crt
    key_file: /etc/tempo/tls/tempo.key
  grpc_tls_config:
    cert_file: /etc/tempo/tls/tempo.crt
    key_file: /etc/tempo/tls/tempo.key

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
          tls:
            cert_file: /etc/tempo/tls/tempo.crt
            key_file: /etc/tempo/tls/tempo.key

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

echo -e "   ${GREEN}✓${NC} Configuration updated with TLS"

echo ""
echo "6. Restarting Tempo..."
sudo systemctl restart tempo

sleep 3

echo ""
echo "7. Checking Tempo status..."
if systemctl is-active --quiet tempo; then
    echo -e "   ${GREEN}✓${NC} Tempo is running with TLS"
else
    echo -e "   ${YELLOW}⚠${NC} Tempo status:"
    systemctl status tempo --no-pager | head -15
    exit 1
fi

echo ""
echo "8. Testing HTTPS endpoint..."
sleep 2
if curl -k -s https://localhost:3200/ready > /dev/null 2>&1; then
    echo -e "   ${GREEN}✓${NC} HTTPS endpoint is responding"
    curl -k https://localhost:3200/ready
else
    echo -e "   ${YELLOW}⚠${NC} HTTPS endpoint test failed"
    echo "   Checking logs..."
    journalctl -u tempo -n 20 --no-pager
fi

echo ""
echo "9. Testing HTTP endpoint (should fail)..."
if curl -s http://localhost:3200/ready > /dev/null 2>&1; then
    echo -e "   ${YELLOW}⚠${NC} HTTP still accessible"
else
    echo -e "   ${GREEN}✓${NC} HTTP disabled (as expected)"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo -e "${GREEN}✅ TLS Setup Complete!${NC}"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "📊 Tempo is now accessible via HTTPS:"
echo "   HTTP API: https://localhost:3200"
echo "   GRPC (OTLP): localhost:4317 (with TLS)"
echo ""
echo "🔐 Certificate Details:"
echo "   Certificate: $CERT_DIR/tempo.crt"
echo "   Private Key: $CERT_DIR/tempo.key"
echo ""
echo "⚠️  Note: This is a self-signed certificate."
echo "   Use -k flag with curl: curl -k https://localhost:3200"
echo ""
echo "📝 Next Steps:"
echo "   1. Update Grafana Tempo datasource to use HTTPS"
echo "   2. Update OpenTelemetry collectors to use TLS"
echo "   3. Update Prometheus remote_write URL (already done)"
echo ""
