# Syslog Error Monitoring Setup Guide

## 🎯 What You're Adding

**Syslog errors** from `/var/log/syslog` - the main system log file that captures:
- System events
- Service messages
- Application logs
- Daemon messages
- Hardware events

---

## 📊 New Dashboard Panel

**Panel:** Syslog Errors
**Query:** `{job="syslog", is_error="true"}`
**Shows:** Only error messages from syslog (not all logs)

---

## 🚀 Deployment Steps

### **Step 1: Deploy Vector Configuration**

Run the deployment script:

```bash
./deploy-syslog-config.sh
```

This will:
1. ✅ Validate the new configuration
2. ✅ Backup your current config
3. ✅ Deploy the new config with syslog
4. ✅ Restart Vector
5. ✅ Verify syslog is collecting

---

### **Step 2: Wait for Data**

Wait 1-2 minutes for syslog data to start flowing to Loki.

---

### **Step 3: Import Updated Dashboard**

1. Go to: http://localhost:3001/dashboard/import
2. Upload: `/home/pde/ag/grafana-dashboard-with-syslog.json`
3. Select "Loki" as data source
4. Click "Import"

---

## 📋 What's Changed

### **Vector Configuration:**

**Added:**
```toml
[sources.syslog_file]
type = "file"
include = ["/var/log/syslog"]
read_from = "end"

[transforms.syslog_labels]
# Extracts process name
# Flags errors and warnings

[sinks.loki_syslog]
labels.job = "syslog"
labels.is_error = "{{ is_error }}"
```

---

### **Dashboard Panels (7 total):**

1. **AG Service Logs** - Your application
2. **System Errors (Critical)** - Priority 0-3 from systemd
3. **Kernel Errors** - Kernel-specific errors
4. **Authentication Logs** - SSH, sudo, logins
5. **Failed Logins** - Security monitoring
6. **Syslog Errors** ✨ **NEW** - System log errors
7. **All Errors** - Unified view

---

## 🔍 Syslog Queries

### **All syslog logs:**
```
{job="syslog"}
```

### **Syslog errors only:**
```
{job="syslog", is_error="true"}
```

### **Syslog warnings:**
```
{job="syslog", is_warning="true"}
```

### **Syslog by process:**
```
{job="syslog", process="systemd"}
{job="syslog", process="NetworkManager"}
```

---

## 📊 What Syslog Contains

**Example messages you'll see:**

**System events:**
```
systemd[1]: Started Session 123 of user pde
```

**Service messages:**
```
NetworkManager[1234]: <info> device (eth0): link connected
```

**Application logs:**
```
CRON[5678]: (root) CMD (run-parts /etc/cron.hourly)
```

**Errors:**
```
systemd[1]: Failed to start Some Service
kernel: Out of memory: Killed process 1234
```

---

## ⚡ Performance Impact

**Syslog file size:** ~36 MB (from your system)

**Expected impact:**
- **Minimal** - Only reading from end of file
- **Filtered** - Only errors flagged with `is_error="true"`
- **Efficient** - File source is lightweight

**Query performance:**
- `{job="syslog"}` - Fast (label-based)
- `{job="syslog", is_error="true"}` - Very fast (indexed labels)

---

## 🎯 Use Cases

### **System Monitoring:**
- Track system service failures
- Monitor daemon errors
- Catch system-wide issues

### **Troubleshooting:**
- Debug service startup failures
- Investigate system errors
- Correlate events across services

### **Security:**
- Monitor for suspicious activity
- Track system-level security events
- Audit system changes

---

## 🔧 Verification

### **Check Vector is collecting:**
```bash
systemctl --user status vector
journalctl --user -u vector -n 20
```

### **Check Loki has syslog data:**
```bash
curl -s http://127.0.0.1:3100/loki/api/v1/label/job/values | jq
```

Should show: `"syslog"` in the list

### **Query syslog errors:**
```bash
curl -s -G http://127.0.0.1:3100/loki/api/v1/query_range \
  --data-urlencode 'query={job="syslog", is_error="true"}' \
  --data-urlencode 'limit=5'
```

---

## 📝 Files Created

| File | Purpose |
|------|---------|
| `vector-with-syslog.toml` | Updated Vector config |
| `deploy-syslog-config.sh` | Deployment script |
| `grafana-dashboard-with-syslog.json` | Updated dashboard |
| `SYSLOG_SETUP_GUIDE.md` | This guide |

---

## 🆘 Troubleshooting

### **No syslog data appearing:**

1. **Check Vector is running:**
   ```bash
   systemctl --user status vector
   ```

2. **Check Vector logs:**
   ```bash
   journalctl --user -u vector -f
   ```

3. **Verify syslog file exists:**
   ```bash
   ls -lh /var/log/syslog
   ```

4. **Check file permissions:**
   ```bash
   # Vector needs read access to /var/log/syslog
   # Usually readable by 'adm' group
   ```

### **Permission denied error:**

If Vector can't read `/var/log/syslog`, you may need to add your user to the `adm` group:

```bash
sudo usermod -a -G adm $USER
# Then restart Vector
systemctl --user restart vector
```

---

## 🎉 Summary

**What you're getting:**
- ✅ Syslog error monitoring
- ✅ System-wide error visibility
- ✅ Process-level filtering
- ✅ Integrated with existing dashboards
- ✅ Minimal performance impact

**Total log sources: 5**
1. systemd-journal (AG + monitoring services)
2. system-errors (critical systemd errors)
3. kernel (kernel errors)
4. auth (authentication logs)
5. syslog (system log errors) ✨ **NEW**

---

**Ready to deploy? Run:**
```bash
./deploy-syslog-config.sh
```

Then import the new dashboard! 🚀
