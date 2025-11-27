# Grafana Dashboard Import Guide

## 📊 Step-by-Step Instructions

### **Step 1: Open Grafana**

1. Open your browser and go to: **http://localhost:3000**
2. Login with your credentials (default: admin/admin)

---

### **Step 2: Verify Loki Data Source**

1. Click the **☰ menu** (top left)
2. Go to **Connections** → **Data sources**
3. Look for **Loki** in the list
4. Click on it and verify:
   - URL: `http://localhost:3100`
   - Click **Save & Test** - should show green checkmark ✅

If Loki is not configured:
1. Click **Add data source**
2. Select **Loki**
3. Set URL to: `http://localhost:3100`
4. Click **Save & Test**

---

### **Step 3: Import the Main Dashboard**

1. Click the **☰ menu** (top left)
2. Go to **Dashboards**
3. Click **New** → **Import**
4. Click **Upload JSON file**
5. Select: `grafana-multi-source-logs-dashboard.json`
6. Click **Load**
7. Select **Loki** as the data source
8. Click **Import**

✅ **You should now see the "Multi-Source Logs Overview" dashboard!**

---

### **Step 4: Explore the Dashboard**

The dashboard has 13 panels organized by source:

#### **Systemd Journal (AG Service)**
- Panel 1: AG Service Logs (all logs)
- Panel 2: AG Service Errors (errors only)
- Panel 3: Error Rate (time series)
- Panel 4: HTTP Status Distribution (pie chart)

#### **System Errors**
- Panel 5: System Errors (critical logs)
- Panel 6: System Error Count by Service (time series)

#### **Kernel Logs**
- Panel 7: Kernel Logs (all kernel logs)
- Panel 8: Kernel Errors (errors only)

#### **Authentication Logs**
- Panel 9: Authentication Logs (all auth logs)
- Panel 10: Failed Login Attempts (time series)
- Panel 11: Auth Events Distribution (bar gauge)

#### **Cross-Source**
- Panel 12: All Errors Across All Sources (stacked time series)
- Panel 13: Monitoring Stack Logs (Loki, Prometheus, Grafana)

---

### **Step 5: Customize the Dashboard**

#### **Adjust Time Range**
- Top right corner: Click the time picker
- Select: Last 15 minutes, Last 1 hour, Last 6 hours, etc.
- Or set a custom range

#### **Set Auto-Refresh**
- Top right corner: Click the refresh dropdown
- Select: 5s, 10s, 30s, 1m, etc.

#### **Filter Logs**
- Click on any log panel
- Use the search box to filter by text
- Click on labels to add filters

#### **Edit Panels**
- Hover over panel title
- Click the **⋮** menu
- Select **Edit**
- Modify query, visualization, or settings
- Click **Apply** to save

---

### **Step 6: Create Additional Dashboards**

You can create focused dashboards for specific use cases:

#### **Security Dashboard**
```
Query: {job="auth"}
Panels:
- Failed login attempts
- Successful logins
- Sudo commands
- Session activity
```

#### **AG Service Dashboard**
```
Query: {job="systemd-journal", systemd_unit="ag.service"}
Panels:
- Request rate
- Error rate
- HTTP status codes
- Response times
- Trace IDs
```

#### **System Health Dashboard**
```
Queries:
- {job="system-errors"}
- {job="kernel", is_error="true"}
- {job="systemd-journal", is_error="true"}
```

---

### **Step 7: Save Your Dashboard**

1. Click the **💾 Save dashboard** icon (top right)
2. Add a description (optional)
3. Click **Save**

---

## 🎨 Dashboard Customization Tips

### **Change Panel Type**
- Edit panel → Visualization dropdown
- Options: Logs, Time series, Bar chart, Pie chart, Table, Stat, Gauge, etc.

### **Add Variables**
1. Dashboard settings (⚙️ icon)
2. Variables → Add variable
3. Example: Create a `service` variable to filter by systemd_unit

### **Add Annotations**
1. Dashboard settings (⚙️ icon)
2. Annotations → Add annotation query
3. Mark important events on time series

### **Set Alerts**
1. Edit panel
2. Alert tab
3. Create alert rule
4. Example: Alert when error rate > 10/min

---

## 🔍 Useful LogQL Queries

### **Filter by Text**
```
{job="systemd-journal"} |= "error"
{job="systemd-journal"} |~ "error|fail"
{job="systemd-journal"} != "health check"
```

### **Extract Fields**
```
{job="systemd-journal"} | json | line_format "{{.message}}"
```

### **Rate Queries**
```
rate({job="systemd-journal", is_error="true"}[5m])
```

### **Count Over Time**
```
count_over_time({job="auth", auth_event="failed_password"}[1h])
```

### **Sum by Label**
```
sum by (systemd_unit) (count_over_time({job="systemd-journal"}[5m]))
```

---

## 📊 Panel Examples

### **Log Panel**
- Shows raw log lines
- Good for: Debugging, searching, viewing details

### **Time Series Panel**
- Shows metrics over time
- Good for: Trends, rates, counts

### **Pie Chart Panel**
- Shows distribution
- Good for: HTTP status codes, error types

### **Bar Gauge Panel**
- Shows current values
- Good for: Comparing categories

### **Stat Panel**
- Shows single number
- Good for: Total errors, current rate

---

## 🚨 Troubleshooting

### **No Data in Panels**
1. Check time range (expand to last 1 hour)
2. Verify Loki data source is working
3. Check Vector is running: `systemctl --user status vector`
4. Query Loki directly: `curl http://127.0.0.1:3100/loki/api/v1/label/job/values`

### **Query Errors**
1. Check LogQL syntax
2. Verify label names exist
3. Use Explore to test queries first

### **Slow Queries**
1. Reduce time range
2. Add more specific label filters
3. Use `|= "text"` to filter early in pipeline

---

## 📝 Next Steps

1. ✅ Import the main dashboard
2. ✅ Explore the panels
3. ✅ Customize to your needs
4. ✅ Create additional focused dashboards
5. ✅ Set up alerts for critical errors
6. ✅ Share dashboards with your team

---

## 🔗 Resources

- **Grafana Docs:** https://grafana.com/docs/grafana/latest/
- **LogQL Docs:** https://grafana.com/docs/loki/latest/logql/
- **Dashboard JSON:** `./grafana-multi-source-logs-dashboard.json`
- **Query Reference:** `./GRAFANA_DASHBOARD_QUERIES.md`

Enjoy your new dashboards! 🎉
