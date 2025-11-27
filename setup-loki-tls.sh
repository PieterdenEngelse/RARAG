#!/bin/bash
# Setup TLS for Loki

set -e

echo "🔐 Setting up TLS for Loki..."
echo ""

# Configuration
CERT_DIR="$HOME/.config/loki/tls"
CERT_VALIDITY_DAYS=3650  # 10 years
LOKI_CONFIG="$HOME/.config/loki/config.yml"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "1. Creating TLS certificate directory..."
mkdir -p "$CERT_DIR"
echo -e "   ${GREEN}✓${NC} Directory created: $CERT_DIR"

echo ""
echo "2. Generating self-signed TLS certificate..."
openssl req -new -newkey rsa:2048 -days $CERT_VALIDITY_DAYS -nodes -x509 \
    -keyout "$CERT_DIR/loki.key" \
    -out "$CERT_DIR/loki.crt" \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=loki.local" \
    -addext "subjectAltName=DNS:localhost,DNS:loki.local,IP:127.0.0.1"

echo -e "   ${GREEN}✓${NC} Certificate generated"
echo "   - Certificate: $CERT_DIR/loki.crt"
echo "   - Private Key: $CERT_DIR/loki.key"
echo "   - Validity: $CERT_VALIDITY_DAYS days"

echo ""
echo "3. Setting correct permissions..."
chmod 600 "$CERT_DIR/loki.key"
chmod 644 "$CERT_DIR/loki.crt"
echo -e "   ${GREEN}✓${NC} Permissions set"

echo ""
echo "4. Backing up current Loki configuration..."
cp "$LOKI_CONFIG" "$LOKI_CONFIG.backup-before-tls"
echo -e "   ${GREEN}✓${NC} Backup created: $LOKI_CONFIG.backup-before-tls"

echo ""
echo "5. Updating Loki configuration with TLS..."
cat > "$LOKI_CONFIG" <<EOF
auth_enabled: false

server:
  http_listen_port: 3100
  grpc_listen_port: 0
  # TLS Configuration
  http_tls_config:
    cert_file: $CERT_DIR/loki.crt
    key_file: $CERT_DIR/loki.key
    # Minimum TLS version
    min_version: TLS12
    # Client authentication (optional - disabled for now)
    # client_auth_type: RequireAndVerifyClientCert
    # client_ca_file: $CERT_DIR/ca.crt

ingester:
  lifecycler:
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
  chunk_idle_period: 5m
  max_chunk_age: 1h

schema_config:
  configs:
    - from: 2023-01-01
      store: boltdb-shipper
      object_store: filesystem
      schema: v13
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: $HOME/.local/share/loki/index
    cache_location: $HOME/.local/share/loki/cache
    
  filesystem:
    directory: $HOME/.local/share/loki/chunks

compactor:
  working_directory: $HOME/.local/share/loki/compactor
  
limits_config:
  retention_period: 168h   # 7 days
  max_cache_freshness_per_query: 10m
  allow_structured_metadata: false

query_range:
  parallelise_shardable_queries: true

table_manager:
  retention_deletes_enabled: true
  retention_period: 168h
EOF

echo -e "   ${GREEN}✓${NC} Configuration updated with TLS"

echo ""
echo "6. Restarting Loki..."
systemctl --user restart loki

sleep 3

echo ""
echo "7. Checking Loki status..."
if systemctl --user is-active --quiet loki; then
    echo -e "   ${GREEN}✓${NC} Loki is running with TLS"
else
    echo -e "   ${YELLOW}⚠${NC} Loki status:"
    systemctl --user status loki --no-pager | head -15
    exit 1
fi

echo ""
echo "8. Testing HTTPS endpoint..."
sleep 2
if curl -k -s https://localhost:3100/ready > /dev/null 2>&1; then
    echo -e "   ${GREEN}✓${NC} HTTPS endpoint is responding"
    curl -k https://localhost:3100/ready
else
    echo -e "   ${YELLOW}⚠${NC} HTTPS endpoint test failed"
    echo "   Checking logs..."
    journalctl --user -u loki -n 20 --no-pager
fi

echo ""
echo "9. Testing HTTP endpoint (should fail)..."
if curl -s http://localhost:3100/ready > /dev/null 2>&1; then
    echo -e "   ${YELLOW}⚠${NC} HTTP still accessible"
else
    echo -e "   ${GREEN}✓${NC} HTTP disabled (as expected)"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo -e "${GREEN}✅ TLS Setup Complete!${NC}"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "📊 Loki is now accessible via HTTPS:"
echo "   URL: https://localhost:3100"
echo ""
echo "🔐 Certificate Details:"
echo "   Certificate: $CERT_DIR/loki.crt"
echo "   Private Key: $CERT_DIR/loki.key"
echo ""
echo "⚠️  Note: This is a self-signed certificate."
echo "   Use -k flag with curl: curl -k https://localhost:3100"
echo ""
echo "📝 Next Steps:"
echo "   1. Update Vector sinks to use HTTPS"
echo "   2. Update Grafana datasource to use HTTPS"
echo "   3. Update Prometheus scrape config"
echo ""
