# Loki Multi-Source Log Collection Guide

## Overview

Promtail is now configured to collect logs from **multiple sources** across your system, providing comprehensive visibility into your entire monitoring stack and system errors.

## Available Log Sources

### 1. **AG Service Logs** (`systemd-journal`)
- **Source:** ag.service via systemd journald
- **Format:** Text with structured fields
- **Labels:** `level`, `request_id`, `http_status`, `http_method`, `duration_ms`, `is_error`, `is_warning`
- **Use Case:** Main application logs, request tracking, performance monitoring

### 2. **AG File Logs** (`ag-file-logs`)
- **Source:** `/home/pde/.agentic-rag/logs/backend.log.*`
- **Format:** JSON
- **Labels:** `level`, `request_id`, `http_status`, `is_error`, `is_warning`
- **Use Case:** Persistent logs, non-systemd runs (cargo run, manual execution)

### 3. **Monitoring Stack Logs** (`systemd-monitoring`)
- **Source:** Monitoring services via systemd journald
  - loki.service
  - otelcol.service
  - prometheus.service
  - grafana-server.service
  - alertmanager.service
  - promtail.service
- **Format:** Text with level extraction
- **Labels:** `level`, `systemd_unit`, `is_error`, `is_warning`
- **Use Case:** Monitor the monitoring stack itself, detect infrastructure issues

### 4. **System Errors** (`system-errors`)
- **Source:** All systemd services with priority 0-3 (emergency, alert, critical, error)
- **Format:** Raw error messages
- **Labels:** `systemd_unit`, `priority`, `is_error`
- **Use Case:** System-wide error detection, catch critical failures

## Query Examples

### AG Service Queries

```bash
# All AG logs (both sources)
{job=~"systemd-journal|ag-file-logs"}

# Only errors from AG
{job=~"systemd-journal|ag-file-logs", level="ERROR"}

# Slow requests (>100ms)
{job="systemd-journal"} | duration_ms > 100

# Specific request trace
{job=~"systemd-journal|ag-file-logs", request_id="<id>"}

# 5xx errors
{job="systemd-journal", http_status=~"5.."}
```

### Monitoring Stack Queries

```bash
# All monitoring stack logs
{job="systemd-monitoring"}

# Loki errors
{job="systemd-monitoring", systemd_unit="loki.service", level="error"}

# Prometheus warnings
{job="systemd-monitoring", systemd_unit="prometheus.service", level="warn"}

# OTEL Collector issues
{job="systemd-monitoring", systemd_unit="otelcol.service"}

# Grafana logs
{job="systemd-monitoring", systemd_unit="grafana-server.service"}
```

### System-Wide Error Queries

```bash
# All system errors
{job="system-errors"}

# Critical errors only (priority 0-2)
{job="system-errors", priority=~"[0-2]"}

# Errors from specific service
{job="system-errors", systemd_unit="tempo.service"}

# All errors across all sources
{is_error="true"}
```

### Combined Queries

```bash
# All errors from any source
{is_error="true"}

# All warnings from any source
{is_warning="true"}

# Errors from AG or monitoring stack
{job=~"systemd-journal|systemd-monitoring", level=~"ERROR|error"}

# Search for specific error message across all logs
{job=~".*"} |~ "(?i)connection refused"
```

## Grafana Dashboard Queries

### Error Rate Panel
```logql
# Total error rate across all sources
sum(rate({is_error="true"}[5m]))

# Error rate by job
sum by (job) (rate({is_error="true"}[5m]))

# Error rate by service
sum by (systemd_unit) (rate({is_error="true"}[5m]))
```

### Log Volume Panel
```logql
# Log volume by source
sum by (job) (rate({job=~".*"}[1m]))

# AG service log volume
sum(rate({job=~"systemd-journal|ag-file-logs"}[1m]))
```

### Top Errors Panel
```logql
# Top 10 error messages
topk(10, 
  sum by (systemd_unit) (count_over_time({is_error="true"}[1h]))
)
```

### Service Health Panel
```logql
# Services with errors in last hour
count by (systemd_unit) (
  count_over_time({is_error="true"}[1h])
) > 0
```

## Log Source Details

### Journald Priority Levels

| Priority | Name | Description |
|----------|------|-------------|
| 0 | emerg | System is unusable |
| 1 | alert | Action must be taken immediately |
| 2 | crit | Critical conditions |
| 3 | err | Error conditions |
| 4 | warning | Warning conditions |
| 5 | notice | Normal but significant |
| 6 | info | Informational |
| 7 | debug | Debug-level messages |

**Note:** `system-errors` job captures priority 0-3 only.

### Log Formats

**AG Journald Logs (systemd-journal):**
```
2025-11-23T10:08:39.219862Z  INFO http_request{...}: ag::monitoring::trace_middleware: ...
```

**AG File Logs (ag-file-logs):**
```json
{
  "timestamp": "2025-11-23T10:11:24.219541Z",
  "level": "INFO",
  "fields": {
    "message": "request completed",
    "method": "GET",
    "path": "/monitoring/metrics",
    "status": 200,
    "duration_ms": 0,
    "trace_id": "88932164-2049-45f3-9fc9-9b4c7ee644d2"
  },
  "target": "ag::monitoring::trace_middleware"
}
```

**Monitoring Stack Logs (systemd-monitoring):**
```
level=info ts=2025-11-23T10:20:19.476684386Z caller=roundtrip.go:348 org_id=fake ...
```

**System Errors (system-errors):**
```
tempo.service: Failed at step EXEC spawning /usr/bin/tempo: No such file or directory
```

## Configuration Files

### Promtail Config
**File:** `/home/pde/.config/promtail/config.yml`

**Key Sections:**
- `systemd-journal-ag` - AG service logs
- `systemd-journal-monitoring` - Monitoring stack logs
- `ag-file-logs` - AG file logs
- `systemd-journal-errors` - System-wide errors

### Service Files
- **Promtail:** `~/.config/systemd/user/promtail.service`
- **Loki:** `~/.config/systemd/user/loki.service`
- **OTEL Collector:** `~/.config/systemd/user/otelcol.service`
- **AG:** `/etc/systemd/system/ag.service`

## Troubleshooting

### No Logs from Monitoring Services

**Check if services are running:**
```bash
systemctl --user status loki.service
systemctl --user status otelcol.service
systemctl status prometheus.service
```

**Check journald has logs:**
```bash
journalctl --user -u loki.service -n 10
journalctl --user -u otelcol.service -n 10
```

**Verify Promtail is scraping:**
```bash
journalctl --user -u promtail.service | grep "Adding target"
```

### Missing Labels

**Check Promtail pipeline:**
```bash
# View Promtail config
cat ~/.config/promtail/config.yml

# Check for errors
journalctl --user -u promtail.service -p err
```

### High Log Volume

**Adjust max_age in Promtail config:**
```yaml
journal:
  max_age: 6h  # Reduce from 12h
```

**Filter out noisy services:**
```yaml
relabel_configs:
  - source_labels: ['__journal__systemd_unit']
    regex: '(noisy-service)\.service'
    action: drop
```

## Best Practices

1. **Use specific job filters** for faster queries
2. **Leverage `is_error` label** for cross-source error tracking
3. **Set up alerts** for system-errors job (critical issues)
4. **Monitor log volume** to detect anomalies
5. **Use `systemd_unit` label** to drill down to specific services
6. **Combine sources** for comprehensive debugging

## Example: Debugging a System Issue

**Step 1: Check for any errors**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={is_error="true"}' \
  --data-urlencode 'limit=20'
```

**Step 2: Identify affected service**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={job="system-errors"}' \
  --data-urlencode 'limit=10'
```

**Step 3: Get detailed logs from that service**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="tempo.service"}' \
  --data-urlencode 'limit=50'
```

**Step 4: Check related services**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={job="systemd-monitoring"}' \
  --data-urlencode 'limit=50'
```

## Summary

You now have **comprehensive log collection** from:
- ✅ AG application (journald + files)
- ✅ Monitoring stack (Loki, OTEL, Prometheus, Grafana, Alertmanager)
- ✅ System-wide errors (all critical failures)
- ✅ All logs enriched with labels for easy filtering
- ✅ Error and warning flags for quick identification

**Total visibility across your entire stack!** 🎉
