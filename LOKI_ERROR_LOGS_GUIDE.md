# Loki Enhanced Error Log Filtering Guide

## Overview

The `ag` service logs are now enhanced with structured labels in Loki, enabling powerful filtering and querying capabilities for error tracking, performance monitoring, and debugging.

## Available Labels

After the Promtail pipeline enhancement, the following labels are automatically extracted from logs:

| Label | Description | Example Values |
|-------|-------------|----------------|
| `level` | Log level | `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL` |
| `request_id` | Unique request identifier | `6afb77c9-91b2-495c-85a3-4f5b81edd78f` |
| `http_status` | HTTP response status code | `200`, `404`, `500` |
| `http_method` | HTTP request method | `GET`, `POST`, `PUT`, `DELETE` |
| `duration_ms` | Request duration in milliseconds | `0`, `15`, `250` |
| `systemd_unit` | Systemd service name | `ag.service` |
| `hostname` | Server hostname | `pde-BOHB-WAX9` |
| `priority` | Syslog priority | `6` (INFO), `3` (ERROR) |

## Query Examples

### 1. Filter by Log Level

**Get all ERROR logs:**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service",level="ERROR"}' \
  --data-urlencode 'limit=50'
```

**Get all WARN logs:**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service",level="WARN"}' \
  --data-urlencode 'limit=50'
```

### 2. Filter by HTTP Status

**Get all 5xx errors:**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service",http_status=~"5.."}' \
  --data-urlencode 'limit=50'
```

**Get all 4xx client errors:**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service",http_status=~"4.."}' \
  --data-urlencode 'limit=50'
```

**Get successful requests (200):**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service",http_status="200"}' \
  --data-urlencode 'limit=10'
```

### 3. Filter by Request ID

**Track a specific request through the system:**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service",request_id="6afb77c9-91b2-495c-85a3-4f5b81edd78f"}' \
  --data-urlencode 'limit=100'
```

### 4. Filter by HTTP Method

**Get all POST requests:**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service",http_method="POST"}' \
  --data-urlencode 'limit=50'
```

### 5. Performance Monitoring

**Find slow requests (duration > 100ms):**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service"} | duration_ms > 100' \
  --data-urlencode 'limit=50'
```

### 6. Combined Filters

**Get ERROR logs with HTTP status 500:**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service",level="ERROR",http_status="500"}' \
  --data-urlencode 'limit=50'
```

**Get WARN or ERROR logs:**
```bash
curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={systemd_unit="ag.service",level=~"WARN|ERROR"}' \
  --data-urlencode 'limit=50'
```

## Grafana Dashboard Queries

### Error Rate Panel
```logql
sum(rate({systemd_unit="ag.service",level="ERROR"}[5m]))
```

### HTTP Status Distribution
```logql
sum by (http_status) (rate({systemd_unit="ag.service",http_status!=""}[5m]))
```

### Request Duration Histogram
```logql
histogram_quantile(0.95, 
  sum(rate({systemd_unit="ag.service",duration_ms!=""}[5m])) by (le)
)
```

### Top Slow Endpoints
```logql
topk(10, 
  avg by (path) (duration_ms) 
  from {systemd_unit="ag.service",duration_ms!=""}
)
```

## Configuration Files

### ag Service Environment
**File:** `/home/pde/.config/ag/ag.env`

Key setting for clean logs (no ANSI color codes):
```bash
NO_COLOR=1
```

### Promtail Configuration
**File:** `/home/pde/.config/promtail/config.yml`

Pipeline stages extract labels from log lines using regex patterns:
- Log level extraction: `(?P<level>TRACE|DEBUG|INFO|WARN|ERROR|FATAL)`
- Request ID extraction: `request_id=(?P<request_id>[a-f0-9-]+)`
- HTTP status extraction: `status=(?P<http_status>\d{3})`
- HTTP method extraction: `method=(?P<http_method>GET|POST|...)`
- Duration extraction: `duration_ms=(?P<duration_ms>\d+)`

## Troubleshooting

### Labels Not Appearing

1. **Check ANSI codes are disabled:**
   ```bash
   journalctl -u ag.service -n 1 -o cat
   ```
   Should show clean output without `[2m`, `[32m` codes.

2. **Verify Promtail is running:**
   ```bash
   systemctl --user status promtail.service
   ```

3. **Check Promtail is sending logs:**
   ```bash
   curl -s http://localhost:9080/metrics | grep promtail_sent_entries_total
   ```

4. **Verify Loki is receiving logs:**
   ```bash
   curl -s "http://127.0.0.1:3100/loki/api/v1/labels"
   ```

### Restart Services

**Restart ag service (to apply env changes):**
```bash
sudo systemctl restart ag.service
```

**Restart Promtail (to apply config changes):**
```bash
systemctl --user restart promtail.service
```

**Restart Loki (if needed):**
```bash
systemctl --user restart loki.service
```

## Best Practices

1. **Use specific labels** for faster queries (e.g., `level="ERROR"` instead of regex)
2. **Limit time ranges** for better performance
3. **Use request_id** to trace requests across multiple log lines
4. **Set up alerts** in Grafana for ERROR and WARN levels
5. **Monitor http_status** for 4xx and 5xx patterns
6. **Track duration_ms** to identify performance degradation

## Example: Debugging a Failed Request

1. Find the error:
   ```bash
   curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
     --data-urlencode 'query={level="ERROR"}' \
     --data-urlencode 'limit=1'
   ```

2. Extract the request_id from the error log

3. Get all logs for that request:
   ```bash
   curl -G "http://127.0.0.1:3100/loki/api/v1/query" \
     --data-urlencode 'query={request_id="<extracted-id>"}' \
     --data-urlencode 'limit=100'
   ```

4. Analyze the full request lifecycle

## Summary

The enhanced Loki log filtering provides:
- ✅ **Error tracking** via `level` label
- ✅ **Request tracing** via `request_id` label
- ✅ **HTTP monitoring** via `http_status` and `http_method` labels
- ✅ **Performance analysis** via `duration_ms` label
- ✅ **Powerful querying** with LogQL
- ✅ **Grafana integration** for dashboards and alerts

All logs are automatically enriched with these labels through the Promtail pipeline, requiring no code changes to the `ag` service.
