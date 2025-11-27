# Trace-Based Alerting - Quick Start Guide

## 5-Minute Setup

### Step 1: Start Tempo

```bash
# Using Docker
docker run -d \
  --name tempo \
  -p 3200:3200 \
  -p 4317:4317 \
  grafana/tempo:latest

# Verify Tempo is running
curl http://localhost:3200/ready
```

### Step 2: Configure Environment

Create or update your `.env` file:

```bash
# Enable OpenTelemetry (sends traces to Tempo)
OTEL_TRACES_ENABLED=true
OTEL_OTLP_EXPORT=true
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317

# Enable trace-based alerting
TEMPO_ENABLED=true
TEMPO_URL=http://127.0.0.1:3200

# Optional: Configure webhook for alerts
TEMPO_ALERT_WEBHOOK_URL=https://webhook.site/your-unique-id
```

### Step 3: Start Application

```bash
cargo run
```

**Expected Output:**
```
🔍 OpenTelemetry initialized
🔔 Trace-based alerting started
🚀 Starting API server on http://127.0.0.1:3010 ...
```

### Step 4: Generate Traces

```bash
# Make some API requests to generate traces
curl http://localhost:3010/search?q=test
curl http://localhost:3010/documents
curl http://localhost:3010/monitoring/health
```

### Step 5: Verify Alerting

Check logs for anomaly detection:

```bash
tail -f logs/app.log | grep "trace anomaly"
```

## Configuration Cheat Sheet

| Variable | Default | Description |
|----------|---------|-------------|
| `TEMPO_ENABLED` | `false` | Enable/disable trace alerting |
| `TEMPO_URL` | `http://127.0.0.1:3200` | Tempo API endpoint |
| `TEMPO_ALERT_INTERVAL_SECS` | `30` | Check interval (2 queries/min) |
| `TEMPO_LATENCY_THRESHOLD_MS` | `1000` | Alert if span > 1000ms |
| `TEMPO_ERROR_RATE_THRESHOLD` | `0.05` | Alert if error rate > 5% |
| `TEMPO_ALERT_WEBHOOK_URL` | - | Webhook for alerts (optional) |
| `TEMPO_LOOKBACK_WINDOW_SECS` | `60` | Query last 60 seconds |

## Common Use Cases

### 1. Monitor Production API Latency

```bash
# Alert on spans > 500ms
TEMPO_LATENCY_THRESHOLD_MS=500

# Check every 15 seconds
TEMPO_ALERT_INTERVAL_SECS=15
```

### 2. Detect Error Spikes

```bash
# Alert if error rate > 1%
TEMPO_ERROR_RATE_THRESHOLD=0.01

# Look back 2 minutes
TEMPO_LOOKBACK_WINDOW_SECS=120
```

### 3. Development Testing

```bash
# Disable alerting in dev
TEMPO_ENABLED=false

# Or use webhook.site for testing
TEMPO_ALERT_WEBHOOK_URL=https://webhook.site/your-id
```

## Alert Examples

### High Latency Alert

```json
{
  "anomaly_type": "high_latency",
  "trace_id": "abc123...",
  "span_name": "GET /api/search",
  "duration_ms": 1500,
  "affected_traces": 1,
  "total_traces": 50,
  "timestamp": 1705420800
}
```

### Error Status Alert

```json
{
  "anomaly_type": "error_status",
  "trace_id": "def456...",
  "span_name": "POST /upload",
  "error_message": "Trace contains error status",
  "affected_traces": 1,
  "total_traces": 50,
  "timestamp": 1705420800
}
```

### High Error Rate Alert

```json
{
  "anomaly_type": "high_error_rate",
  "affected_traces": 10,
  "total_traces": 100,
  "timestamp": 1705420800
}
```

## Troubleshooting

### No Traces in Tempo

**Problem**: Alerting runs but finds no traces

**Solution**:
1. Verify OpenTelemetry is enabled: `OTEL_TRACES_ENABLED=true`
2. Check OTLP endpoint: `OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317`
3. Make API requests to generate traces

### Connection Refused

**Problem**: `Failed to check for trace anomalies error="Connection refused"`

**Solution**:
1. Verify Tempo is running: `docker ps | grep tempo`
2. Check Tempo port: `curl http://localhost:3200/ready`
3. Update `TEMPO_URL` if using different port

### No Alerts Sent

**Problem**: Anomalies detected but no webhook alerts

**Solution**:
1. Verify webhook URL is set: `echo $TEMPO_ALERT_WEBHOOK_URL`
2. Test webhook manually: `curl -X POST $TEMPO_ALERT_WEBHOOK_URL -d '{"test":true}'`
3. Check logs for webhook errors

## Next Steps

- Read the [full documentation](TRACE_ALERTING.md)
- Configure [Prometheus metrics](../README.md#monitoring)
- Set up [production alerting](TRACE_ALERTING.md#production-deployment)
- Explore [Tempo UI](http://localhost:3200) for trace visualization

## Performance Notes

- **Query Frequency**: Every 30s = 2 queries/min
- **Query Timeout**: ~100ms per check
- **Memory Usage**: Minimal (processes 100 traces per check)
- **CPU Usage**: Negligible (background task)
- **Network**: ~2 HTTP requests per 30s

## Docker Compose Example

```yaml
version: '3.8'

services:
  tempo:
    image: grafana/tempo:latest
    ports:
      - "3200:3200"  # Tempo HTTP
      - "4317:4317"  # OTLP gRPC
    command: ["-config.file=/etc/tempo.yaml"]
    volumes:
      - ./tempo.yaml:/etc/tempo.yaml
      - tempo-data:/tmp/tempo

  ag:
    build: .
    ports:
      - "3010:3010"
    environment:
      - OTEL_TRACES_ENABLED=true
      - OTEL_OTLP_EXPORT=true
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://tempo:4317
      - TEMPO_ENABLED=true
      - TEMPO_URL=http://tempo:3200
    depends_on:
      - tempo

volumes:
  tempo-data:
```

## Support

For more help:
- Check logs: `tail -f logs/app.log`
- Enable debug: `RUST_LOG=debug cargo run`
- Review [full docs](TRACE_ALERTING.md)
