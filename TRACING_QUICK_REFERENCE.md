# Tracing Quick Reference Card

## 🚀 Quick Start

### Restart AG Backend (Apply Changes)

```bash
# Option 1: Kill and restart
kill $(pgrep -f "target/release/ag")
cargo run --release

# Option 2: Rebuild and restart
cargo build --release
kill $(pgrep -f "target/release/ag")
./target/release/ag

# Option 3: Using systemd (if configured)
systemctl --user restart ag.service
```

### Verify Everything Works

```bash
# Run verification script
./verify-tracing-session.sh

# Or manually check metrics
curl http://127.0.0.1:3010/monitoring/metrics | grep -E "(trace_|process_|tracing_)"
```

---

## 📊 Metrics Reference

### Trace Alerting Metrics

```promql
# Alert check status
trace_alert_checks_total{status="ok"}      # Successful checks
trace_alert_checks_total{status="error"}   # Failed checks

# Anomalies detected
trace_anomalies_total{type="high_latency"}    # High latency traces
trace_anomalies_total{type="error_status"}    # Error traces
trace_anomalies_total{type="high_error_rate"} # High error rate
```

### Resource Attribution Metrics

```promql
# Memory metrics
process_memory_bytes                # Current memory (RSS)
process_memory_peak_bytes           # Peak memory
tracing_memory_overhead_bytes       # Estimated tracing overhead

# CPU metrics
process_cpu_percent                 # Current CPU usage (0-100)
tracing_cpu_overhead_percent        # Estimated tracing overhead (~0.5%)

# Overhead percentage
(tracing_memory_overhead_bytes / process_memory_bytes) * 100
```

---

## ⚙️ Configuration

### Trace Alerting

```bash
# Enable/disable
TEMPO_ENABLED=true

# Tempo endpoint
TEMPO_URL=https://localhost:3200

# TLS settings (for self-signed certs)
TEMPO_ALERT_INSECURE_TLS=true

# Alert thresholds
TEMPO_ALERT_INTERVAL_SECS=30           # Check every 30s
TEMPO_LATENCY_THRESHOLD_MS=1000        # Alert if >1000ms
TEMPO_ERROR_RATE_THRESHOLD=0.05        # Alert if >5% errors
TEMPO_LOOKBACK_WINDOW_SECS=60          # Look back 60s

# Webhook (optional)
TEMPO_ALERT_WEBHOOK_URL=https://your-webhook
```

### Resource Attribution

```bash
# Enable/disable (default: true)
RESOURCE_ATTRIBUTION_ENABLED=true

# Update interval (default: 60s)
RESOURCE_ATTRIBUTION_UPDATE_INTERVAL_SECS=60
```

---

## 🔍 Troubleshooting

### Trace Alerting Not Working

```bash
# Check if enabled
curl http://127.0.0.1:3010/monitoring/metrics | grep trace_alert_checks_total

# Should see both status="ok" and status="error"
# If only status="error", check TLS configuration

# Verify TLS setting
grep TEMPO_ALERT_INSECURE_TLS .env
# Should show: TEMPO_ALERT_INSECURE_TLS=true

# Check Tempo is running
systemctl status tempo.service
curl -k https://localhost:3200/api/search?start=0&end=9999999999&limit=1
```

### Resource Attribution Not Showing

```bash
# Check if enabled
grep RESOURCE_ATTRIBUTION_ENABLED .env

# Wait 60 seconds for first update
sleep 60

# Check metrics
curl http://127.0.0.1:3010/monitoring/metrics | grep process_memory_bytes

# If still not showing, check logs
journalctl --user -u ag.service | grep "Resource attribution"
# Should see: "📊 Resource attribution started"
```

### High Memory/CPU Overhead

```bash
# Check actual overhead
curl http://127.0.0.1:3010/monitoring/metrics | grep -E "(process_|tracing_)"

# Calculate overhead percentage
# Memory overhead should be ~1.5%
# CPU overhead should be ~0.5%

# If higher, consider:
# 1. Reduce trace sampling rate
# 2. Increase TEMPO_ALERT_INTERVAL_SECS
# 3. Increase RESOURCE_ATTRIBUTION_UPDATE_INTERVAL_SECS
```

---

## 📈 Grafana Dashboards

### Trace Alerting Panel

```promql
# Alert checks over time
rate(trace_alert_checks_total[5m])

# Anomalies by type
sum by (type) (trace_anomalies_total)

# Error rate
rate(trace_alert_checks_total{status="error"}[5m]) / 
rate(trace_alert_checks_total[5m])
```

### Resource Attribution Panel

```promql
# Memory usage
process_memory_bytes
process_memory_peak_bytes

# Memory overhead
tracing_memory_overhead_bytes

# Memory overhead percentage
(tracing_memory_overhead_bytes / process_memory_bytes) * 100

# CPU usage
process_cpu_percent

# CPU overhead
tracing_cpu_overhead_percent
```

---

## 🔗 Service URLs

| Service | URL | Purpose |
|---------|-----|---------|
| AG Backend | http://127.0.0.1:3010 | Main API |
| Health Check | http://127.0.0.1:3010/monitoring/health | Health status |
| Metrics | http://127.0.0.1:3010/monitoring/metrics | Prometheus metrics |
| Tempo API | https://localhost:3200 | Trace backend |
| OTEL Collector | 127.0.0.1:4318 | Trace ingestion |

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| `docs/TRACE_ALERTING.md` | Complete trace alerting guide |
| `docs/RESOURCE_ATTRIBUTION.md` | Complete resource attribution guide |
| `TRACE_ALERTING_IMPLEMENTATION.md` | Trace alerting implementation summary |
| `RESOURCE_ATTRIBUTION_IMPLEMENTATION.md` | Resource attribution implementation summary |
| `TRACING_SESSION_SUMMARY.md` | This session's work summary |
| `README_TRACING.md` | General tracing documentation |
| `OPENTELEMETRY_TRACING_COMPLETE_GUIDE.md` | Complete OTEL setup guide |

---

## 🧪 Testing

### Run Unit Tests

```bash
# Trace alerting tests
cargo test trace_alerting --lib

# Resource attribution tests
cargo test resource_attribution --lib

# All monitoring tests
cargo test monitoring --lib
```

### Manual Testing

```bash
# 1. Generate some traffic
for i in {1..10}; do
  curl http://127.0.0.1:3010/monitoring/health
  sleep 1
done

# 2. Wait for trace alerting check (30s)
sleep 30

# 3. Check for traces in Tempo
curl -k "https://localhost:3200/api/search?start=0&end=9999999999&limit=10"

# 4. Check metrics
curl http://127.0.0.1:3010/monitoring/metrics | grep trace_

# 5. Wait for resource attribution update (60s)
sleep 60

# 6. Check resource metrics
curl http://127.0.0.1:3010/monitoring/metrics | grep -E "(process_|tracing_)"
```

---

## 🎯 Expected Behavior

### After Restart

**Within 1 second:**
- ✅ "🔔 Trace-based alerting started" in logs
- ✅ "📊 Resource attribution started" in logs

**Within 30 seconds:**
- ✅ `trace_alert_checks_total{status="ok"}` starts incrementing
- ✅ No more TLS handshake errors in Tempo logs

**Within 60 seconds:**
- ✅ All resource attribution metrics appear
- ✅ `process_memory_bytes` shows current memory
- ✅ `process_cpu_percent` shows CPU usage

**Ongoing:**
- ✅ Trace checks every 30 seconds
- ✅ Resource updates every 60 seconds
- ✅ Metrics continuously updated

---

## 🚨 Common Issues

### Issue: `trace_alert_checks_total{status="error"}` keeps incrementing

**Cause**: TLS configuration not applied or Tempo not accessible

**Fix**:
```bash
# 1. Verify TLS setting
grep TEMPO_ALERT_INSECURE_TLS .env

# 2. Restart AG backend
kill $(pgrep -f "target/release/ag")
cargo run --release

# 3. Check Tempo is running
systemctl status tempo.service
```

### Issue: Resource metrics not appearing

**Cause**: Resource attribution not started or waiting for first update

**Fix**:
```bash
# 1. Check if enabled
grep RESOURCE_ATTRIBUTION_ENABLED .env

# 2. Wait 60 seconds
sleep 60

# 3. Check metrics again
curl http://127.0.0.1:3010/monitoring/metrics | grep process_memory_bytes

# 4. If still missing, check logs
journalctl --user -u ag.service | grep -i resource
```

### Issue: High overhead reported

**Cause**: Normal variation or high trace volume

**Fix**:
```bash
# 1. Check actual values
curl http://127.0.0.1:3010/monitoring/metrics | grep -E "(process_|tracing_)"

# 2. Monitor over time (not just one reading)

# 3. If consistently high, reduce trace sampling
# Edit OTEL configuration to sample fewer traces
```

---

## 💡 Tips

1. **Wait for metrics**: Resource attribution updates every 60s, trace alerting every 30s
2. **Check logs first**: Look for startup messages to confirm features are enabled
3. **Use verification script**: `./verify-tracing-session.sh` checks everything
4. **Monitor trends**: Single readings can be misleading, watch over time
5. **Grafana is your friend**: Visualize metrics for better insights

---

## 🔄 Quick Commands

```bash
# Restart AG backend
kill $(pgrep -f "target/release/ag") && cargo run --release

# Check all tracing metrics
curl -s http://127.0.0.1:3010/monitoring/metrics | grep -E "(trace_|process_|tracing_)"

# Watch metrics in real-time
watch -n 5 'curl -s http://127.0.0.1:3010/monitoring/metrics | grep -E "(trace_|process_|tracing_)"'

# Check Tempo connectivity
curl -k https://localhost:3200/api/search?start=0&end=9999999999&limit=1

# View recent logs
journalctl --user -u ag.service -n 50 -f

# Run verification
./verify-tracing-session.sh
```

---

**Last Updated**: 2025-11-26
**Version**: 13.1.2
**Status**: ✅ Ready for deployment
