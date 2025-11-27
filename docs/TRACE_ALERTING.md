# Trace-Based Alerting

## Overview

The trace-based alerting system queries Tempo (distributed tracing backend) every 30 seconds to detect anomalies in application traces. It provides real-time monitoring of trace health and sends alerts when issues are detected.

## Features

- **Automated Anomaly Detection**: Continuously monitors traces for issues
- **Multiple Anomaly Types**: Detects high latency, errors, and high error rates
- **Performance Optimized**: ~100ms query timeout, non-blocking execution
- **Configurable Thresholds**: Customize latency and error rate thresholds
- **Webhook Alerts**: Send notifications to external systems
- **Background Task**: Runs independently without blocking the main application

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Application Startup                                │
│  ├─ Initialize OpenTelemetry (sends traces)         │
│  ├─ Start Trace Alerting Task (queries traces)      │
│  └─ Start API Server                                │
└─────────────────────────────────────────────────────┘
         │
         ├─ Every 30s ──────────────────────┐
         │                                   │
         ▼                                   ▼
┌──────────────────────┐          ┌──────────────────────┐
│  Tempo Query         │          │  Anomaly Detection   │
│  (~100ms timeout)    │  ──────> │  - High latency      │
│  /api/search         │          │  - Error status      │
│  Time: last 60s      │          │  - High error rate   │
└──────────────────────┘          └──────────────────────┘
                                           │
                                           ▼
                                  ┌──────────────────────┐
                                  │  Send Alert          │
                                  │  (webhook)           │
                                  └──────────────────────┘
```

## Configuration

All configuration is done via environment variables. Add these to your `.env` file:

### Basic Configuration

```bash
# Enable trace-based alerting
TEMPO_ENABLED=true

# Tempo API endpoint (default: http://127.0.0.1:3200)
TEMPO_URL=http://localhost:3200
```

### Alert Frequency

```bash
# Alert check interval in seconds (default: 30)
# Every 30s = 2 queries/min
TEMPO_ALERT_INTERVAL_SECS=30

# Lookback window in seconds (default: 60)
# How far back to query for traces
TEMPO_LOOKBACK_WINDOW_SECS=60
```

### Anomaly Thresholds

```bash
# Latency threshold in milliseconds (default: 1000)
# Alert if span duration exceeds this value
TEMPO_LATENCY_THRESHOLD_MS=1000

# Error rate threshold (0.0-1.0, default: 0.05 = 5%)
# Alert if error rate exceeds this percentage
TEMPO_ERROR_RATE_THRESHOLD=0.05
```

### Alert Delivery

```bash
# Webhook URL for trace anomaly alerts (optional)
TEMPO_ALERT_WEBHOOK_URL=https://example.com/alerts/traces
```

## Anomaly Types

### 1. High Latency

Detects traces where the duration exceeds the configured threshold.

**Alert Payload:**
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

### 2. Error Status

Detects traces with error status codes using TraceQL queries.

**Alert Payload:**
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

### 3. High Error Rate

Detects when the percentage of error traces exceeds the threshold.

**Alert Payload:**
```json
{
  "anomaly_type": "high_error_rate",
  "affected_traces": 10,
  "total_traces": 100,
  "timestamp": 1705420800
}
```

## Tempo API Integration

The system uses Tempo's REST API to query traces:

### Primary Query: `/api/search`

Retrieves recent traces within a time window:

```
GET /api/search?start={unix_timestamp}&end={unix_timestamp}&limit=100
```

**Performance:**
- Timeout: 100ms
- Limit: 100 traces per query
- Time range: Last 60 seconds (configurable)

### Error Detection: TraceQL Query

Uses TraceQL to find traces with error status:

```
GET /api/search?start={unix_timestamp}&end={unix_timestamp}&q={status=error}&limit=100
```

**Fallback:**
If TraceQL is not supported or fails, error detection is skipped gracefully.

## Performance Characteristics

- **Query Frequency**: Every 30 seconds (2 queries/min)
- **Query Timeout**: ~100ms per check
- **Non-Blocking**: Runs in background task, doesn't block main application
- **Async Alerts**: Webhook sends are spawned as separate tasks
- **Memory Efficient**: Processes traces in streaming fashion

## Usage

### Starting the Application

The trace alerting system starts automatically when enabled:

```bash
# Set environment variables
export TEMPO_ENABLED=true
export TEMPO_URL=http://localhost:3200
export TEMPO_ALERT_WEBHOOK_URL=https://example.com/alerts

# Start the application
cargo run
```

**Startup Log:**
```
🔍 OpenTelemetry initialized
🔔 Trace-based alerting started
```

### Monitoring Logs

The system logs its activity at different levels:

**INFO**: Anomalies detected and alerts sent
```
Detected trace anomalies count=3
Trace anomaly alert sent successfully anomaly_type="high_latency"
```

**DEBUG**: Regular checks and queries
```
Running trace anomaly check...
Querying Tempo for traces url="http://localhost:3200/api/search"
Retrieved traces from Tempo total_traces=42
No anomalies detected
```

**WARN**: Failures (non-fatal)
```
Failed to check for trace anomalies error="Tempo API error: 503"
Failed to send trace anomaly alert (non-fatal)
```

### Webhook Integration

When an anomaly is detected, a POST request is sent to the configured webhook URL:

**Request:**
```http
POST https://example.com/alerts/traces
Content-Type: application/json

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

**Timeout**: 5 seconds

**Error Handling**: Failures are logged as warnings but don't stop the alerting system.

## Testing

### Unit Tests

Run the included unit tests:

```bash
cargo test trace_alerting
```

**Test Coverage:**
- Configuration loading
- Event creation (high latency, error status, high error rate)
- JSON serialization
- Enable/disable logic

### Integration Testing

1. **Start Tempo**:
   ```bash
   docker run -d -p 3200:3200 grafana/tempo:latest
   ```

2. **Enable Alerting**:
   ```bash
   export TEMPO_ENABLED=true
   export TEMPO_URL=http://localhost:3200
   ```

3. **Generate Traces**:
   ```bash
   # Make API requests to generate traces
   curl http://localhost:3010/search?q=test
   ```

4. **Monitor Logs**:
   ```bash
   # Watch for anomaly detection
   tail -f logs/app.log | grep "trace anomaly"
   ```

### Webhook Testing

Use a webhook testing service like webhook.site:

```bash
export TEMPO_ALERT_WEBHOOK_URL=https://webhook.site/your-unique-id
```

Then trigger anomalies and check the webhook.site dashboard for received alerts.

## Troubleshooting

### No Alerts Being Sent

**Check 1: Is alerting enabled?**
```bash
# Look for this log message
grep "Trace-based alerting started" logs/app.log
```

**Check 2: Is Tempo accessible?**
```bash
curl http://localhost:3200/api/search?start=0&end=9999999999&limit=1
```

**Check 3: Are there traces in Tempo?**
```bash
# Check if traces are being sent
grep "OpenTelemetry initialized" logs/app.log
```

### Tempo Connection Errors

**Error**: `Failed to check for trace anomalies error="Tempo API error: Connection refused"`

**Solution**: Ensure Tempo is running and accessible:
```bash
docker ps | grep tempo
curl http://localhost:3200/ready
```

### Webhook Failures

**Error**: `Failed to send trace anomaly alert (non-fatal)`

**Solution**: Check webhook URL and network connectivity:
```bash
curl -X POST https://your-webhook-url/test -d '{"test": true}'
```

### High Memory Usage

If you notice high memory usage, reduce the lookback window:

```bash
# Reduce from 60s to 30s
TEMPO_LOOKBACK_WINDOW_SECS=30
```

## Best Practices

### Production Deployment

1. **Set Appropriate Thresholds**:
   - Start with conservative thresholds (1000ms latency, 5% error rate)
   - Adjust based on your application's baseline performance

2. **Configure Webhook Alerts**:
   - Use a reliable alerting service (PagerDuty, Slack, etc.)
   - Implement rate limiting on the webhook endpoint

3. **Monitor the Alerting System**:
   - Watch for "Failed to check for trace anomalies" warnings
   - Set up alerts for the alerting system itself

4. **Tune Query Frequency**:
   - 30 seconds is a good default
   - Increase for less critical systems to reduce load
   - Decrease for critical systems requiring faster detection

### Development

1. **Disable in Development**:
   ```bash
   TEMPO_ENABLED=false
   ```

2. **Use Local Tempo**:
   ```bash
   docker-compose up tempo
   TEMPO_URL=http://localhost:3200
   ```

3. **Test with Webhook.site**:
   ```bash
   TEMPO_ALERT_WEBHOOK_URL=https://webhook.site/your-id
   ```

## Metrics

The trace alerting system integrates with the existing monitoring infrastructure:

- **Logs**: Structured logs via `tracing` crate
- **OpenTelemetry**: Traces are sent to Tempo for analysis
- **Prometheus**: Future enhancement to add alerting metrics

## Future Enhancements

Potential improvements for future versions:

1. **Prometheus Metrics**:
   - `trace_anomalies_total{type="high_latency|error_status|high_error_rate"}`
   - `trace_alert_checks_total{status="success|failure"}`
   - `trace_alert_query_duration_ms`

2. **Advanced Anomaly Detection**:
   - Statistical anomaly detection (z-score, moving average)
   - Machine learning-based anomaly detection
   - Pattern recognition (repeated errors, cascading failures)

3. **Alert Aggregation**:
   - Batch multiple anomalies into single alert
   - Deduplication of similar anomalies
   - Alert suppression during maintenance windows

4. **Multiple Backends**:
   - Support for Jaeger, Zipkin
   - Direct OpenTelemetry Collector integration

## References

- [Tempo Documentation](https://grafana.com/docs/tempo/latest/)
- [TraceQL Query Language](https://grafana.com/docs/tempo/latest/traceql/)
- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review logs in `logs/app.log`
3. Enable debug logging: `RUST_LOG=debug`
4. Open an issue on GitHub
