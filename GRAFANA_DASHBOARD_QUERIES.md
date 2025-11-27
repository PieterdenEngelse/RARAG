# Grafana Dashboard Queries for Multi-Source Logs

Vector is now collecting logs from multiple sources. Here are the queries you can use in Grafana to create different dashboards.

## 📊 Available Log Sources

### 1. **Systemd Journal Logs** (`job="systemd-journal"`)
All logs from systemd services (AG + monitoring stack)

#### Query All Systemd Logs
```
{job="systemd-journal"}
```

#### Query by Specific Service
```
{job="systemd-journal", systemd_unit="ag.service"}
{job="systemd-journal", systemd_unit="loki.service"}
{job="systemd-journal", systemd_unit="prometheus.service"}
{job="systemd-journal", systemd_unit="grafana-server.service"}
{job="systemd-journal", systemd_unit="otelcol.service"}
{job="systemd-journal", systemd_unit="alertmanager.service"}
```

#### AG Service Errors Only
```
{job="systemd-journal", systemd_unit="ag.service", is_error="true"}
```

#### AG Service Warnings
```
{job="systemd-journal", systemd_unit="ag.service", is_warning="true"}
```

#### AG Service by Log Level
```
{job="systemd-journal", systemd_unit="ag.service", level="ERROR"}
{job="systemd-journal", systemd_unit="ag.service", level="WARN"}
{job="systemd-journal", systemd_unit="ag.service", level="INFO"}
```

#### AG Service HTTP Errors (4xx, 5xx)
```
{job="systemd-journal", systemd_unit="ag.service", http_status=~"4..|5.."}
```

#### AG Service by HTTP Method
```
{job="systemd-journal", systemd_unit="ag.service", http_method="POST"}
{job="systemd-journal", systemd_unit="ag.service", http_method="GET"}
```

#### Monitoring Stack Errors
```
{job="systemd-journal", systemd_unit=~"loki.service|prometheus.service|grafana-server.service", is_error="true"}
```

---

### 2. **System Errors** (`job="system-errors"`)
Critical system errors (priority 0-3: emergency, alert, critical, error)

#### All Critical System Errors
```
{job="system-errors"}
```

#### System Errors by Service
```
{job="system-errors", systemd_unit="some-service.service"}
```

#### Emergency/Alert Level Only (priority 0-1)
```
{job="system-errors", priority=~"0|1"}
```

---

### 3. **Kernel Logs** (`job="kernel"`)
Kernel messages from /var/log/kern.log

#### All Kernel Logs
```
{job="kernel"}
```

#### Kernel Errors Only
```
{job="kernel", is_error="true"}
```

#### Kernel Warnings
```
{job="kernel", is_warning="true"}
```

---

### 4. **Authentication Logs** (`job="auth"`)
SSH, sudo, login attempts from /var/log/auth.log

#### All Auth Logs
```
{job="auth"}
```

#### Failed Login Attempts
```
{job="auth", auth_event="failed_password"}
```

#### Successful Logins
```
{job="auth", auth_event="accepted_password"}
```

#### Sudo Commands
```
{job="auth", auth_event="sudo"}
```

#### All Auth Failures
```
{job="auth", is_error="true"}
```

#### Session Events
```
{job="auth", auth_event=~"session_opened|session_closed"}
```

---

## 🎯 Cross-Source Queries

### All Errors Across All Sources
```
{is_error="true"}
```

### All Warnings Across All Sources
```
{is_warning="true"}
```

### All Logs from Localhost
```
{hostname="localhost"}
```

---

## 📈 Example Dashboard Panels

### Panel 1: AG Service Error Rate
**Query:**
```
sum(rate({job="systemd-journal", systemd_unit="ag.service", is_error="true"}[5m]))
```

### Panel 2: Failed Login Attempts Over Time
**Query:**
```
count_over_time({job="auth", auth_event="failed_password"}[1m])
```

### Panel 3: HTTP Status Code Distribution
**Query:**
```
sum by (http_status) (count_over_time({job="systemd-journal", systemd_unit="ag.service", http_status!=""}[5m]))
```

### Panel 4: System Error Count by Service
**Query:**
```
sum by (systemd_unit) (count_over_time({job="system-errors"}[5m]))
```

### Panel 5: Kernel Errors
**Query:**
```
{job="kernel", is_error="true"}
```

---

## 🔍 LogQL Tips

### Filter by Text Content
```
{job="systemd-journal"} |= "database"
{job="systemd-journal"} |~ "error|fail"
{job="systemd-journal"} != "health check"
```

### Extract Fields
```
{job="systemd-journal"} | json | line_format "{{.message}}"
```

### Rate Queries
```
rate({job="systemd-journal", systemd_unit="ag.service"}[5m])
```

### Count Over Time
```
count_over_time({job="auth", is_error="true"}[1h])
```

---

## 🚀 Quick Start

1. **Open Grafana** → http://localhost:3000
2. **Go to Explore** or **Create Dashboard**
3. **Select Loki** as data source
4. **Enter one of the queries above**
5. **Run Query** and visualize!

---

## 📝 Notes

- **job="systemd-journal"** is the main category for all systemd logs
- Use **systemd_unit** label to filter by specific service
- **is_error** and **is_warning** flags work across all sources
- File-based sources (kernel, auth) may take a moment to populate
- All timestamps are in UTC

---

## 🔧 Current Configuration

**Vector Config:** `/home/pde/.config/vector/vector.toml`

**Sources:**
- `systemd_all` → AG + monitoring services
- `system_errors` → Critical system errors (priority ≤ 3)
- `kernel_logs` → /var/log/kern.log
- `auth_logs` → /var/log/auth.log

**Loki Endpoint:** http://127.0.0.1:3100
**Vector Metrics:** http://127.0.0.1:8686/metrics
