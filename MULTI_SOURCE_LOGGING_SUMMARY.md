# Multi-Source Logging Setup - Summary

## ✅ What Was Done

You asked about changing `{job="systemd-journal"}` because you wanted dashboards for other log errors. I understood that you wanted to keep `job="systemd-journal"` as a **category** for systemd logs, and use other labels to differentiate between services.

### Solution Implemented

Vector is now configured to collect logs from **4 different sources**:

1. **Systemd Journal** (`job="systemd-journal"`)
   - AG service
   - Loki service
   - Prometheus service
   - Grafana service
   - OpenTelemetry Collector
   - Alertmanager
   
2. **System Errors** (`job="system-errors"`)
   - Critical system errors (priority 0-3)
   - Emergency, alert, critical, and error level messages
   
3. **Kernel Logs** (`job="kernel"`)
   - From `/var/log/kern.log`
   - Kernel errors, panics, oops, segfaults
   
4. **Authentication Logs** (`job="auth"`)
   - From `/var/log/auth.log`
   - SSH logins, sudo commands, authentication failures

---

## 🎯 Why `job="systemd-journal"` Makes Sense

The `job` label acts as a **category** or **source type**:

- `job="systemd-journal"` = "These logs came from systemd journal"
- `job="auth"` = "These logs came from auth.log"
- `job="kernel"` = "These logs came from kernel log"
- `job="system-errors"` = "These are critical system errors"

Then you use **other labels** to drill down:

- `systemd_unit="ag.service"` = "Show me AG service logs"
- `systemd_unit="loki.service"` = "Show me Loki logs"
- `is_error="true"` = "Show me only errors"
- `auth_event="failed_password"` = "Show me failed logins"

---

## 📊 How to Create Dashboards

### Dashboard 1: AG Service Monitoring
```
{job="systemd-journal", systemd_unit="ag.service"}
```

### Dashboard 2: AG Service Errors
```
{job="systemd-journal", systemd_unit="ag.service", is_error="true"}
```

### Dashboard 3: Monitoring Stack Health
```
{job="systemd-journal", systemd_unit=~"loki.service|prometheus.service|grafana-server.service"}
```

### Dashboard 4: System Security
```
{job="auth"}
```

### Dashboard 5: Failed Login Attempts
```
{job="auth", auth_event="failed_password"}
```

### Dashboard 6: Critical System Errors
```
{job="system-errors"}
```

### Dashboard 7: Kernel Issues
```
{job="kernel", is_error="true"}
```

### Dashboard 8: All Errors Across All Systems
```
{is_error="true"}
```

---

## 🔍 Available Labels

You can filter and group by these labels:

| Label | Description | Example Values |
|-------|-------------|----------------|
| `job` | Log source category | `systemd-journal`, `auth`, `kernel`, `system-errors` |
| `systemd_unit` | Systemd service name | `ag.service`, `loki.service`, `prometheus.service` |
| `hostname` | Server hostname | `pde-BOHB-WAX9` |
| `priority` | Syslog priority (0-7) | `0` (emergency), `3` (error), `6` (info) |
| `level` | Log level | `ERROR`, `WARN`, `INFO`, `DEBUG` |
| `is_error` | Error flag | `true` (only on errors) |
| `is_warning` | Warning flag | `true` (only on warnings) |
| `http_status` | HTTP status code | `200`, `404`, `500` |
| `http_method` | HTTP method | `GET`, `POST`, `PUT` |
| `trace_id` | Request trace ID | `c37c4c6f-d89b-4124-b9c1-3f0b2384397d` |
| `auth_event` | Auth event type | `failed_password`, `sudo`, `session_opened` |

---

## 📁 Files Created

1. **`./vector_multi_source_fixed.toml`** - The working Vector config
2. **`./GRAFANA_DASHBOARD_QUERIES.md`** - Complete query reference guide
3. **`./MULTI_SOURCE_LOGGING_SUMMARY.md`** - This summary document

---

## 🚀 Current Status

✅ Vector is running and collecting from all 4 sources
✅ Logs are flowing to Loki
✅ All labels are properly set
✅ Ready to create Grafana dashboards

### Verify It's Working

```bash
# Check Vector status
systemctl --user status vector

# Query Loki for available jobs
curl -s http://127.0.0.1:3100/loki/api/v1/label/job/values | jq

# Query Loki for systemd units
curl -s http://127.0.0.1:3100/loki/api/v1/label/systemd_unit/values | jq

# Sample AG service logs
curl -s -G http://127.0.0.1:3100/loki/api/v1/query_range \
  --data-urlencode 'query={job="systemd-journal", systemd_unit="ag.service"}' \
  --data-urlencode 'limit=10'
```

---

## 🎨 Next Steps

1. **Open Grafana** → http://localhost:3000
2. **Create a new dashboard**
3. **Add panels** using the queries from `GRAFANA_DASHBOARD_QUERIES.md`
4. **Save your dashboards**

### Suggested Dashboards

1. **AG Service Overview**
   - Request rate
   - Error rate
   - HTTP status distribution
   - Response time

2. **Security Dashboard**
   - Failed login attempts
   - Sudo commands
   - Authentication failures
   - Session activity

3. **System Health**
   - Critical system errors
   - Kernel errors
   - Service status
   - Error trends

4. **Monitoring Stack**
   - Loki errors
   - Prometheus errors
   - Grafana errors
   - OTEL collector status

---

## 🔧 Configuration Files

**Vector Config:** `/home/pde/.config/vector/vector.toml`
**Vector Service:** `~/.config/systemd/user/vector.service`
**Vector Metrics:** http://127.0.0.1:8686/metrics
**Loki API:** http://127.0.0.1:3100

---

## 💡 Key Insight

The beauty of this setup is that `job="systemd-journal"` is **not confusing** - it's a **category**. 

Think of it like this:
- **Category (job):** Where did the logs come from? (systemd, auth.log, kernel, etc.)
- **Service (systemd_unit):** Which specific service? (ag.service, loki.service, etc.)
- **Severity (is_error, level):** How important is it?
- **Context (http_status, auth_event):** What happened?

This gives you maximum flexibility to create dashboards for:
- Specific services (AG, Loki, Prometheus)
- Specific error types (HTTP 500s, failed logins, kernel panics)
- Cross-service views (all errors, all warnings)
- Security monitoring (auth events)
- System health (critical errors)

---

## 📝 Notes

- All timestamps are in UTC
- Logs are retained according to Loki's retention policy
- Vector metrics are available at port 8686
- File-based sources (kernel, auth) read from the end by default
- Systemd sources include all historical logs (current_boot_only = false)

Enjoy your multi-source logging setup! 🎉
