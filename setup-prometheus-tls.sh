#!/bin/bash
# Setup TLS for Prometheus

set -e

echo "🔐 Setting up TLS for Prometheus..."
echo ""

# Configuration
CERT_DIR="/etc/prometheus/tls"
CERT_VALIDITY_DAYS=3650  # 10 years
PROMETHEUS_USER="prometheus"

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
    -keyout "$CERT_DIR/prometheus.key" \
    -out "$CERT_DIR/prometheus.crt" \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=prometheus.local" \
    -addext "subjectAltName=DNS:localhost,DNS:prometheus.local,IP:127.0.0.1"

echo -e "   ${GREEN}✓${NC} Certificate generated"
echo "   - Certificate: $CERT_DIR/prometheus.crt"
echo "   - Private Key: $CERT_DIR/prometheus.key"
echo "   - Validity: $CERT_VALIDITY_DAYS days"

echo ""
echo "3. Setting correct permissions..."
sudo chown -R $PROMETHEUS_USER:$PROMETHEUS_USER "$CERT_DIR"
sudo chmod 600 "$CERT_DIR/prometheus.key"
sudo chmod 644 "$CERT_DIR/prometheus.crt"
echo -e "   ${GREEN}✓${NC} Permissions set"

echo ""
echo "4. Creating Prometheus web configuration..."
sudo tee "$CERT_DIR/web-config.yml" > /dev/null <<EOF
tls_server_config:
  cert_file: $CERT_DIR/prometheus.crt
  key_file: $CERT_DIR/prometheus.key
  # Minimum TLS version
  min_version: TLS12
  # Preferred cipher suites
  cipher_suites:
    - TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
    - TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
    - TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
    - TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
  # Client authentication (optional - disabled for now)
  # client_auth_type: RequireAndVerifyClientCert
  # client_ca_file: $CERT_DIR/ca.crt
EOF

sudo chown $PROMETHEUS_USER:$PROMETHEUS_USER "$CERT_DIR/web-config.yml"
sudo chmod 644 "$CERT_DIR/web-config.yml"
echo -e "   ${GREEN}✓${NC} Web config created: $CERT_DIR/web-config.yml"

echo ""
echo "5. Updating Prometheus systemd service..."
sudo cp /etc/systemd/system/prometheus.service /etc/systemd/system/prometheus.service.backup-tls

# Update the service file to include web.config.file
sudo sed -i '/^ExecStart=/s/$/ --web.config.file=\/etc\/prometheus\/tls\/web-config.yml/' /etc/systemd/system/prometheus.service

echo -e "   ${GREEN}✓${NC} Service file updated"
echo "   Backup: /etc/systemd/system/prometheus.service.backup-tls"

echo ""
echo "6. Reloading systemd and restarting Prometheus..."
sudo systemctl daemon-reload
sudo systemctl restart prometheus

sleep 3

echo ""
echo "7. Checking Prometheus status..."
if systemctl is-active --quiet prometheus; then
    echo -e "   ${GREEN}✓${NC} Prometheus is running with TLS"
else
    echo -e "   ${YELLOW}⚠${NC} Prometheus status:"
    systemctl status prometheus --no-pager | head -15
    exit 1
fi

echo ""
echo "8. Testing HTTPS endpoint..."
if curl -k -s https://localhost:9090/-/healthy > /dev/null 2>&1; then
    echo -e "   ${GREEN}✓${NC} HTTPS endpoint is responding"
else
    echo -e "   ${YELLOW}⚠${NC} HTTPS endpoint test failed"
fi

echo ""
echo "9. Testing HTTP endpoint (should fail or redirect)..."
if curl -s http://localhost:9090/-/healthy > /dev/null 2>&1; then
    echo -e "   ${YELLOW}⚠${NC} HTTP still accessible (expected to fail)"
else
    echo -e "   ${GREEN}✓${NC} HTTP disabled (as expected)"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo -e "${GREEN}✅ TLS Setup Complete!${NC}"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "📊 Prometheus is now accessible via HTTPS:"
echo "   URL: https://localhost:9090"
echo ""
echo "🔐 Certificate Details:"
echo "   Certificate: $CERT_DIR/prometheus.crt"
echo "   Private Key: $CERT_DIR/prometheus.key"
echo "   Web Config:  $CERT_DIR/web-config.yml"
echo ""
echo "⚠️  Note: This is a self-signed certificate."
echo "   Browsers will show a security warning."
echo "   Use -k flag with curl: curl -k https://localhost:9090"
echo ""
echo "📝 Next Steps:"
echo "   1. Update scrape configs to use HTTPS"
echo "   2. Update Grafana datasource to use HTTPS"
echo "   3. (Optional) Install certificate in browser"
echo ""
