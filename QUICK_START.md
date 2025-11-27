# Distributed Tracing - Quick Start Guide

## 🚀 Complete the Setup (One Command)

Run this single command to complete the entire distributed tracing setup:

```bash
cd /home/pde/ag
./complete-tracing-setup.sh
```

This will:
1. ✅ Update Tempo configuration to accept plaintext gRPC on port 4320
2. ✅ Restart Tempo service
3. ✅ Restart OpenTelemetry Collector with updated configuration
4. ✅ Verify traces are flowing to Tempo

**Note**: You'll be prompted for your sudo password to update Tempo configuration.

---

## 📋 Manual Step-by-Step (If Preferred)

If you prefer to run each step manually:

### Step 1: Update Tempo Configuration
```bash
cd /home/pde/ag
./update-tempo-config.sh
```

### Step 2: Restart OpenTelemetry Collector
```bash
./update-otelcol-config.sh
```

### Step 3: Verify Everything is Working
```bash
./final-verification.sh
```

---

## ✅ Expected Result

After running the setup, you should see:

```
✅ SUCCESS: Distributed tracing is fully operational!

🎉 Congratulations! Your distributed tracing pipeline is working:

  AG Backend (port 3010)
       ↓ gRPC (port 4318)
  OpenTelemetry Collector
       ↓ gRPC plaintext (port 4320)
  Tempo (port 4320)
```

---

## 🔍 What Changed

### Tempo Configuration (`/etc/tempo/config.yml`)
Added plaintext gRPC receiver:
```yaml
distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: "0.0.0.0:4317"  # Existing TLS endpoint
          tls:
            cert_file: /etc/tempo/tls/tempo.crt
            key_file: /etc/tempo/tls/tempo.key
        grpc_plaintext:  # NEW
          endpoint: "127.0.0.1:4320"
```

### OpenTelemetry Collector Configuration (`~/.config/otelcol/config.yaml`)
Changed endpoint from port 4317 to 4320:
```yaml
exporters:
  otlp/tempo:
    endpoint: 127.0.0.1:4320  # Changed from 4317
    tls:
      insecure: true  # Plaintext gRPC
```

---

## 📊 Next Steps: Grafana Integration

Once traces are flowing, integrate with Grafana:

### 1. Add Tempo Datasource
- Open Grafana: `http://localhost:3000`
- Go to Configuration → Data Sources → Add data source
- Select "Tempo"
- Configure:
  - **URL**: `https://localhost:3200`
  - **Skip TLS Verify**: ✅ Enabled
- Click "Save & Test"

### 2. Explore Traces
- Go to Explore
- Select Tempo datasource
- Search options:
  - **Service Name**: `ag-backend`
  - **Operation**: `GET /monitoring/health`
  - **Time Range**: Last 15 minutes

### 3. Create Dashboards
Create dashboards to visualize:
- Request latency (p50, p95, p99)
- Error rates by endpoint
- Request volume over time
- Service dependency graph

---

## 🛠️ Troubleshooting

### Traces Not Showing Up?

**Check Tempo is listening on port 4320:**
```bash
ss -tlnp | grep 4320
```

**Check Tempo logs:**
```bash
sudo journalctl -u tempo -n 50 --no-pager
```

**Check OpenTelemetry Collector logs:**
```bash
journalctl --user -u otelcol.service -n 50 --no-pager
```

**Verify configuration:**
```bash
grep -A 5 'grpc_plaintext' /etc/tempo/config.yml
```

**Re-run verification:**
```bash
cd /home/pde/ag
./final-verification.sh
```

---

## 📁 Files Created

All scripts are in `/home/pde/ag/`:
- `complete-tracing-setup.sh` - Master script (run this!)
- `update-tempo-config.sh` - Update Tempo configuration
- `update-otelcol-config.sh` - Restart OpenTelemetry Collector
- `final-verification.sh` - Verify traces are flowing
- `verify-tracing.sh` - Quick status check

Documentation:
- `QUICK_START.md` - This file
- `FINAL_STATUS.md` - Detailed status report
- `TRACING_TLS_SOLUTION.md` - TLS troubleshooting guide
- `AG_TRACING_SETUP_SUMMARY.md` - Complete setup documentation

---

## 🎯 Summary

**Current Status**: 99% Complete
**Remaining**: Run `./complete-tracing-setup.sh`
**Time Required**: ~2 minutes
**Result**: Full distributed tracing operational! 🚀
