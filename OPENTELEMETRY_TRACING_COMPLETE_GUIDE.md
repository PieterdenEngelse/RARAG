# OpenTelemetry Tracing Setup - Complete Guide

## ✅ What Has Been Configured

### 1. AG Backend Configuration
- ✅ OpenTelemetry dependencies installed (v0.21.0)
- ✅ Distributed tracing code implemented
- ✅ Environment variables configured in `.env`:
  ```
  OTEL_TRACES_ENABLED=true
  OTEL_OTLP_EXPORT=true
  OTEL_CONSOLE_EXPORT=false
  OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
  OTEL_SERVICE_NAME=ag-backend
  OTEL_SERVICE_VERSION=13.1.2
  OTEL_ENVIRONMENT=development
  OTEL_EXPORTER_OTLP_INSECURE=true
  ```
- ✅ AG backend is sending traces to OpenTelemetry Collector on port 4318

### 2. OpenTelemetry Collector Configuration
- ✅ Collector is running and receiving traces from AG backend
- ✅ Collector is configured to forward traces to Tempo on port 4317
- ✅ Configuration file: `~/.config/otelcol/config.yaml`
  ```yaml
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: 127.0.0.1:4318
        http:
          endpoint: 127.0.0.1:4319

  processors:
    batch:
      send_batch_size: 512
      timeout: 5s

  exporters:
    otlp/tempo:
      endpoint: localhost:4317
      tls:
        insecure: true
    logging:
      loglevel: debug

  service:
    pipelines:
      traces:
        receivers: [otlp]
        processors: [batch]
        exporters: [otlp/tempo, logging]
  ```

### 3. Tempo Configuration
- ✅ Tempo is running and listening on:
  - Port 3200: HTTPS API
  - Port 4317: OTLP gRPC receiver
  - Port 9095: Internal gRPC (for distributor → ingester communication)
- ✅ Tempo is receiving traces from the OpenTelemetry Collector
- ✅ Metrics show traces are being received:
  ```
  tempo_distributor_bytes_received_total{tenant="single-tenant"} 487385
  tempo_distributor_spans_received_total{tenant="single-tenant"} 1126
  ```

## ⚠️ Current Issue

**Traces are being received but NOT stored!**

The problem is that Tempo's internal gRPC communication (distributor → ingester on port 9095) is failing due to TLS configuration mismatch:

```
tempo_distributor_ingester_append_failures_total{ingester="127.0.0.1:9095"} 1105
tempo_receiver_refused_spans{receiver="otlp/otlp_receiver",transport="grpc"} 1126
```

**Root Cause:**
- Tempo's internal gRPC server (port 9095) has TLS enabled via `grpc_tls_config`
- The distributor is trying to connect to the ingester without TLS
- This causes "connection reset by peer" errors

## 🔧 The Fix

You need to update Tempo's configuration to remove TLS from the internal gRPC server while keeping it for the external HTTP API.

### Step 1: Edit Tempo Configuration

Edit `/etc/tempo/config.yml` and **remove** the `grpc_tls_config` section:

```bash
sudo nano /etc/tempo/config.yml
```

**Before:**
```yaml
server:
  http_listen_port: 3200
  log_level: info
  http_tls_config:
    cert_file: /etc/tempo/tls/tempo.crt
    key_file: /etc/tempo/tls/tempo.key
  grpc_tls_config:              # ← REMOVE THIS SECTION
    cert_file: /etc/tempo/tls/tempo.crt
    key_file: /etc/tempo/tls/tempo.key
```

**After:**
```yaml
server:
  http_listen_port: 3200
  log_level: info
  http_tls_config:
    cert_file: /etc/tempo/tls/tempo.crt
    key_file: /etc/tempo/tls/tempo.key
  # Internal gRPC (port 9095) - NO TLS for internal communication
```

### Step 2: Restart Tempo

```bash
sudo systemctl restart tempo
```

### Step 3: Verify Tempo is Running

```bash
sudo systemctl status tempo
```

### Step 4: Generate Test Traces

Make some requests to the AG backend to generate traces:

```bash
for i in {1..10}; do
  curl -s http://127.0.0.1:3010/monitoring/health > /dev/null
  curl -s http://127.0.0.1:3010/monitoring/metrics > /dev/null
  sleep 1
done
```

### Step 5: Verify Traces are Being Stored

Check Tempo metrics to see if traces are now being ingested successfully:

```bash
curl -sk https://localhost:3200/metrics | grep -E 'tempo_ingester_traces_created_total|tempo_distributor_ingester_append_failures'
```

**Expected output (after fix):**
```
tempo_ingester_traces_created_total{tenant="single-tenant"} 10  # Non-zero!
tempo_distributor_ingester_append_failures_total{ingester="127.0.0.1:9095"} 0  # Should be 0
```

### Step 6: Search for Traces

Once traces are being stored, you can search for them:

```bash
curl -sk 'https://localhost:3200/api/search?tags=service.name%3Dag-backend' | jq
```

## 📊 Creating Grafana Dashboards

Once traces are flowing, you can create Tempo dashboards in Grafana:

### 1. Add Tempo Datasource

1. Open Grafana: `http://localhost:3001`
2. Go to **Configuration** → **Data Sources**
3. Click **Add data source**
4. Select **Tempo**
5. Configure:
   - **URL**: `https://localhost:3200`
   - **Skip TLS Verify**: ✅ (checked)
6. Click **Save & Test**

### 2. Explore Traces

1. Go to **Explore** in Grafana
2. Select **Tempo** datasource
3. Use **Search** tab:
   - **Service Name**: `ag-backend`
   - **Span Name**: (leave empty to see all)
4. Click **Run Query**

### 3. Create Dashboard Panels

Create panels for:

1. **Trace Search**
   - Panel type: Traces
   - Query: Search by service name

2. **Service Graph**
   - Panel type: Node Graph
   - Uses Tempo's service-graphs processor

3. **Request Rate (RED Metrics)**
   - Panel type: Time series
   - Query Prometheus for: `rate(tempo_distributor_spans_received_total[5m])`

4. **Latency Percentiles**
   - Panel type: Time series
   - Query Prometheus for span duration metrics

5. **Error Rate**
   - Panel type: Time series
   - Query for spans with error status

## 🔍 Troubleshooting

### Check OpenTelemetry Collector Logs

```bash
journalctl --user -u otelcol.service -f
```

### Check Tempo Logs

```bash
sudo journalctl -u tempo -f
```

### Check AG Backend Logs

```bash
journalctl _COMM=ag -f
```

### Verify Ports are Listening

```bash
ss -tlnp | grep -E '(4317|4318|4319|9095|3200)'
```

Expected output:
```
LISTEN 0      4096       127.0.0.1:4319       0.0.0.0:*    users:(("otelcol-contrib",pid=XXX,fd=10))
LISTEN 0      4096       127.0.0.1:4318       0.0.0.0:*    users:(("otelcol-contrib",pid=XXX,fd=9))
LISTEN 0      4096               *:9095             *:*    # Tempo internal gRPC
LISTEN 0      4096               *:4317             *:*    # Tempo OTLP receiver
LISTEN 0      4096               *:3200             *:*    # Tempo HTTP API
```

## 📝 Architecture Overview

```
┌─────────────┐
│ AG Backend  │ (port 3010)
│             │
│ OpenTelemetry
│ SDK v0.21   │
└──────┬──────┘
       │ gRPC (port 4318)
       ▼
┌──────────────────────┐
│ OpenTelemetry        │
│ Collector            │
│                      │
│ - Receives: 4318/4319│
│ - Batches traces     │
│ - Forwards to Tempo  │
└──────┬───────────────┘
       │ gRPC (port 4317)
       ▼
┌──────────────────────┐
│ Tempo                │
│                      │
│ Distributor (4317)   │
│       ↓              │
│ Ingester (9095)      │ ← Internal gRPC (plaintext)
│       ↓              │
│ Storage (/var/tempo) │
│                      │
│ HTTP API (3200)      │ ← HTTPS for external access
└──────────────────────┘
       │
       ▼
┌──────────────────────┐
│ Grafana              │ (port 3001)
│                      │
│ - Tempo datasource   │
│ - Trace visualization│
│ - Service graphs     │
└──────────────────────┘
```

## 🎯 Summary

**What's Working:**
- ✅ AG backend is instrumented with OpenTelemetry
- ✅ Traces are being sent to OpenTelemetry Collector
- ✅ Collector is forwarding traces to Tempo
- ✅ Tempo is receiving traces

**What Needs to be Fixed:**
- ⚠️ Tempo's internal gRPC TLS configuration
- ⚠️ Remove `grpc_tls_config` from `/etc/tempo/config.yml`
- ⚠️ Restart Tempo service

**After the Fix:**
- ✅ Traces will be stored in Tempo
- ✅ You can search and visualize traces in Grafana
- ✅ Service graphs and metrics will be generated
- ✅ Complete distributed tracing setup!

## 📚 Related Files

- `/home/pde/ag/.env` - AG backend environment configuration
- `/home/pde/ag/Cargo.toml` - OpenTelemetry dependencies
- `/home/pde/ag/src/monitoring/otel_config.rs` - OpenTelemetry initialization
- `/home/pde/ag/src/monitoring/distributed_tracing.rs` - Tracing implementation
- `~/.config/otelcol/config.yaml` - OpenTelemetry Collector configuration
- `/etc/tempo/config.yml` - Tempo configuration (needs fix)
- `/home/pde/ag/fix-otelcol-tempo-connection.sh` - Collector fix script (already applied)
- `/home/pde/ag/fix-tempo-internal-grpc.sh` - Tempo fix script (needs sudo to run)

---

**Last Updated**: $(date)
**Status**: ⚠️ Awaiting Tempo configuration fix
