# Quick Reference - Grafana Loki Queries

## 🚀 Most Common Queries

### AG Service
```
{job="systemd-journal", systemd_unit="ag.service"}
```

### AG Errors
```
{job="systemd-journal", systemd_unit="ag.service", is_error="true"}
```

### Failed Logins
```
{job="auth", auth_event="failed_password"}
```

### All Errors
```
{is_error="true"}
```

### Kernel Errors
```
{job="kernel", is_error="true"}
```

### Critical System Errors
```
{job="system-errors"}
```

---

## 📊 Available Jobs

| Job | Description |
|-----|-------------|
| `systemd-journal` | All systemd services (AG, Loki, Prometheus, Grafana, etc.) |
| `system-errors` | Critical system errors (priority ≤ 3) |
| `kernel` | Kernel logs from /var/log/kern.log |
| `auth` | Authentication logs from /var/log/auth.log |

---

## 🏷️ Key Labels

| Label | Use Case |
|-------|----------|
| `systemd_unit` | Filter by specific service |
| `is_error` | Show only errors |
| `is_warning` | Show only warnings |
| `level` | Filter by log level (ERROR, WARN, INFO) |
| `http_status` | Filter by HTTP status code |
| `auth_event` | Filter by auth event type |

---

## 🔍 LogQL Operators

```
|=  "text"     # Contains text
|~ "regex"     # Matches regex
!= "text"      # Does not contain
!~ "regex"     # Does not match regex
```

---

## 📈 Rate & Count Queries

```
# Error rate over 5 minutes
rate({job="systemd-journal", is_error="true"}[5m])

# Count errors in last hour
count_over_time({job="auth", is_error="true"}[1h])

# Sum by service
sum by (systemd_unit) (count_over_time({job="systemd-journal"}[5m]))
```

---

## 🎯 Quick Examples

### Show AG HTTP 500 errors
```
{job="systemd-journal", systemd_unit="ag.service", http_status=~"5.."}
```

### Show all POST requests
```
{job="systemd-journal", systemd_unit="ag.service", http_method="POST"}
```

### Show Loki errors
```
{job="systemd-journal", systemd_unit="loki.service", is_error="true"}
```

### Show sudo commands
```
{job="auth", auth_event="sudo"}
```

---

## 🔧 Useful Commands

```bash
# Check Vector status
systemctl --user status vector

# Restart Vector
systemctl --user restart vector

# View Vector logs
journalctl --user -u vector -f

# Query Loki labels
curl -s http://127.0.0.1:3100/loki/api/v1/labels | jq

# Query job values
curl -s http://127.0.0.1:3100/loki/api/v1/label/job/values | jq
```

---

## 📍 Endpoints

- **Grafana:** http://localhost:3000
- **Loki API:** http://127.0.0.1:3100
- **Vector Metrics:** http://127.0.0.1:8686/metrics
- **AG Service:** http://127.0.0.1:3010

---

## 📚 Documentation

- **Full Query Guide:** `./GRAFANA_DASHBOARD_QUERIES.md`
- **Setup Summary:** `./MULTI_SOURCE_LOGGING_SUMMARY.md`
- **Vector Config:** `/home/pde/.config/vector/vector.toml`
