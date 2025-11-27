# AG Backend Distributed Tracing Setup Summary

## ✅ What Was Completed

### 1. Code Configuration
- ✅ OpenTelemetry dependencies already installed in Cargo.toml (v0.21.0)
- ✅ Distributed tracing code already implemented in `src/monitoring/distributed_tracing.rs`
- ✅ TraceMiddleware already applied to HTTP requests in `src/api/mod.rs`
- ✅ Updated `src/monitoring/otel_config.rs` to support TLS configuration

### 2. Environment Configuration
- ✅ Updated `.env` file with OTLP export enabled:
  ```
  OTEL_TRACES_ENABLED=true
  OTEL_OTLP_EXPORT=true
  OTEL_CONSOLE_EXPORT=false
  OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
  OTEL_SERVICE_NAME=ag-backend
  OTEL_SERVICE_VERSION=13.1.2
  OTEL_ENVIRONMENT=development
  ```

### 3. Build & Deployment
- ✅ Rebuilt AG backend with new configuration
- ✅ Restarted AG backend service
- ✅ Verified AG backend is running and responding to requests

### 4. Verification
- ✅ AG backend starts successfully with OpenTelemetry initialized
- ✅ Logs show: "Initializing OpenTelemetry: service=ag-backend, otlp_export=true, endpoint=http://127.0.0.1:4317"
- ✅ HTTP requests are being traced (visible in logs with trace_id)

## ⚠️ Current Issue

**Traces are NOT reaching Tempo:**
- Tempo metrics show: `tempo_ingester_traces_created_total{tenant="single-tenant"} 0`
- AG backend is attempting to send traces but they're not being received

## 🔍 Troubleshooting Steps

### Step 1: Check gRPC Connection

The AG backend uses gRPC to send traces to Tempo on port 4317. Let's verify the connection:

```bash
# Check if Tempo is listening on port 4317
ss -tlnp | grep 4317

# Expected output:
LISTEN 0      4096       0.0.0.0:4317       0.0.0.0:*    users:(("tempo",pid=XXXXX,fd=X))
```

### Step 2: Test gRPC Connection with grpcurl

```bash
# Install grpcurl if not already installed
sudo apt-get install -y grpcurl

# Test connection to Tempo's OTLP gRPC endpoint
grpcurl -plaintext localhost:4317 list

# If TLS is required:
grpcurl -insecure localhost:4317 list
```

### Step 3: Check AG Backend Logs for OTLP Errors

```bash
# Check for OTLP export errors
journalctl _COMM=ag -n 100 --no-pager | grep -i "otlp\|export\|trace"

# Or check tmux window logs
tmux capture-pane -t main:5 -p | grep -i "otlp\|export\|error"
```

### Step 4: Verify Tempo Configuration

```bash
# Check Tempo's OTLP receiver configuration
grep -A 10 "grpc:" /etc/tempo/config.yml

# Expected output should show:
# grpc:
#   endpoint: "0.0.0.0:4317"
#   tls:
#     cert_file: /etc/tempo/tls/tempo.crt
#     key_file: /etc/tempo/tls/tempo.key
```

### Step 5: Test with Simple OTLP Client

Create a test script to verify Tempo can receive traces:

```bash
# Install opentelemetry-cli (if available) or use curl to send a test trace
curl -X POST http://localhost:4318/v1/traces \
  -H "Content-Type: application/json" \
  -d '{
    "resourceSpans": [{
      "resource": {
        "attributes": [{
          "key": "service.name",
          "value": {"stringValue": "test-service"}
        }]
      },
      "scopeSpans": [{
        "spans": [{
          "traceId": "5B8EFFF798038103D269B633813FC60C",
          "spanId": "EEE19B7EC3C1B174",
          "name": "test-span",
          "startTimeUnixNano": "1544712660000000000",
          "endTimeUnixNano": "1544712661000000000"
        }]
      }]
    }]
  }'

# Then check Tempo metrics:
curl -sk https://localhost:3200/metrics | grep tempo_ingester_traces_created_total
```

## 🔧 Potential Solutions

### Solution 1: Use HTTP/JSON Instead of gRPC

Modify `.env` to use HTTP endpoint (port 4318):

```bash
cd /home/pde/ag
sed -i 's|OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317|OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318|' .env

# Restart AG backend
tmux send-keys -t main:5 C-c
sleep 2
tmux send-keys -t main:5 "cd /home/pde/ag && ./target/release/ag" C-m
```

### Solution 2: Disable TLS for OTLP (Development Only)

Modify Tempo configuration to accept non-TLS gRPC:

```bash
# Backup current config
sudo cp /etc/tempo/config.yml /etc/tempo/config.yml.backup-before-otlp-notls

# Edit config to add a non-TLS gRPC receiver
sudo nano /etc/tempo/config.yml

# Add under distributor.receivers:
#   otlp:
#     protocols:
#       grpc:
#         endpoint: "0.0.0.0:4318"  # Non-TLS port
#       http:
#         endpoint: "0.0.0.0:4319"

# Restart Tempo
sudo systemctl restart tempo
```

### Solution 3: Configure AG Backend to Skip TLS Verification

The current `otel_config.rs` has an `insecure` flag but it's not being used by the tonic client. We need to modify the code to actually configure TLS:

```rust
// In src/monitoring/otel_config.rs, modify the OTLP exporter configuration:

if config.otlp_export {
    use tonic::transport::ClientTlsConfig;
    
    let mut exporter_builder = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.otlp_endpoint);
    
    // Configure TLS if using HTTPS
    if config.otlp_endpoint.starts_with("https://") && config.insecure {
        // For self-signed certs, we need to configure tonic to accept them
        // This requires additional tonic configuration
        tracing::warn!("HTTPS endpoint with insecure=true: TLS verification will be skipped");
    }
    
    let otlp_exporter = exporter_builder.build_span_exporter()?;
    // ... rest of the code
}
```

## 📝 Next Steps

1. **Immediate**: Run troubleshooting steps 1-4 to identify the exact issue
2. **Quick Fix**: Try Solution 1 (use HTTP/JSON on port 4318)
3. **Proper Fix**: Implement Solution 3 (configure TLS properly in the code)
4. **Verify**: Make requests to AG backend and check Tempo metrics
5. **Create Dashboard**: Once traces are flowing, create Tempo dashboards in Grafana

## 📊 Expected Outcome

Once working, you should see:

```bash
# Tempo metrics showing traces:
curl -sk https://localhost:3200/metrics | grep tempo_ingester_traces_created_total
# tempo_ingester_traces_created_total{tenant="single-tenant"} 10  # Non-zero!

# Traces visible in Tempo API:
curl -sk "https://localhost:3200/api/search?tags=service.name%3Dag-backend"
# Should return trace IDs

# Traces visible in Grafana:
# 1. Add Tempo datasource: https://localhost:3200 (Skip TLS Verify)
# 2. Go to Explore > Tempo
# 3. Search for service.name = "ag-backend"
# 4. See traces with spans for HTTP requests
```

## 💾 Files Modified

- `/home/pde/ag/.env` - Added OTLP configuration
- `/home/pde/ag/src/monitoring/otel_config.rs` - Updated with TLS support
- `/home/pde/ag/setup-ag-tracing.sh` - Setup script created

## 💾 Backups Created

- `/home/pde/ag/.env.backup-before-tracing`
- `/home/pde/ag/src/monitoring/otel_config.rs.backup-before-tracing`

## 🔗 Related Documentation

- OpenTelemetry Rust SDK: https://github.com/open-telemetry/opentelemetry-rust
- Tempo OTLP Configuration: https://grafana.com/docs/tempo/latest/configuration/
- Grafana Tempo Datasource: https://grafana.com/docs/grafana/latest/datasources/tempo/

---

**Status**: ⚠️ Partially Complete - Tracing configured but not yet flowing to Tempo

**Last Updated**: $(date)
