#!/bin/bash
# Fix Loki configuration

set -e

echo "🔧 Fixing Loki configuration..."
echo ""

LOKI_CONFIG="$HOME/.config/loki/config.yml"
CERT_DIR="$HOME/.config/loki/tls"

echo "1. Creating corrected Loki configuration..."
cat > "$LOKI_CONFIG" <<EOF
auth_enabled: false

server:
  http_listen_port: 3100
  grpc_listen_port: 0
  # TLS Configuration (simplified - min_version not supported)
  http_tls_config:
    cert_file: $CERT_DIR/loki.crt
    key_file: $CERT_DIR/loki.key

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

echo "   ✓ Configuration fixed (removed unsupported min_version)"

echo ""
echo "2. Restarting Loki..."
systemctl --user restart loki

sleep 3

echo ""
echo "3. Checking Loki status..."
if systemctl --user is-active --quiet loki; then
    echo "   ✓ Loki is running!"
    systemctl --user status loki --no-pager | head -10
else
    echo "   ✗ Loki failed to start"
    echo ""
    echo "Checking logs:"
    journalctl --user -u loki -n 20 --no-pager
    exit 1
fi

echo ""
echo "4. Testing HTTPS endpoint..."
sleep 2
if curl -k -s https://localhost:3100/ready > /dev/null 2>&1; then
    echo "   ✓ HTTPS endpoint is responding!"
    curl -k https://localhost:3100/ready
else
    echo "   ✗ HTTPS endpoint not responding"
fi

echo ""
echo "✅ Loki configuration fixed!"
echo ""
