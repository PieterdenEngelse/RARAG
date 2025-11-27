# Tempo TLS Setup Guide

## 🔐 Enable HTTPS on Tempo

**Goal:** Secure Tempo with HTTPS/TLS encryption

---

## 🚀 Quick Start

### **Step 1: Enable TLS on Tempo**

```bash
./setup-tempo-tls.sh
```

This will:
1. ✅ Generate self-signed TLS certificate
2. ✅ Update Tempo configuration with TLS
3. ✅ Enable TLS on HTTP API (port 3200)
4. ✅ Enable TLS on GRPC (port 4317)
5. ✅ Update Prometheus remote_write to use HTTPS
6. ✅ Restart Tempo with HTTPS
7. ✅ Verify HTTPS is working

### **Step 2: Update Grafana Datasource (if using Tempo)**

1. Open Grafana: http://localhost:3001
2. Go to: Connections → Data sources → Tempo
3. Change URL to: `https://localhost:3200`
4. Enable: "Skip TLS Verify"
5. Click: "Save & Test"

---

## 📋 What Gets Created

### **1. TLS Certificates**

| File | Location | Purpose |
|------|----------|---------|
| Certificate | `/etc/tempo/tls/tempo.crt` | Public certificate |
| Private Key | `/etc/tempo/tls/tempo.key` | Private key |

### **2. Certificate Details**

- **Type:** Self-signed X.509
- **Algorithm:** RSA 2048-bit
- **Validity:** 10 years (3650 days)
- **Subject:** CN=tempo.local
- **SANs:** localhost, tempo.local, 127.0.0.1
- **TLS Version:** TLS 1.2+

---

## 🔧 Configuration Changes

### **Tempo Configuration**

**Before:**
```yaml
server:
  http_listen_port: 3200
  log_level: info
```

**After:**
```yaml
server:
  http_listen_port: 3200
  log_level: info
  http_tls_config:
    cert_file: /etc/tempo/tls/tempo.crt
    key_file: /etc/tempo/tls/tempo.key
  grpc_tls_config:
    cert_file: /etc/tempo/tls/tempo.crt
    key_file: /etc/tempo/tls/tempo.key
```

### **OTLP Receiver (GRPC)**

**Before:**
```yaml
distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: "0.0.0.0:4317"
```

**After:**
```yaml
distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: "0.0.0.0:4317"
          tls:
            cert_file: /etc/tempo/tls/tempo.crt
            key_file: /etc/tempo/tls/tempo.key
```

### **Prometheus Remote Write**

**Before:**
```yaml
metrics_generator:
  storage:
    remote_write:
      - url: http://prometheus:9090/api/v1/write
```

**After:**
```yaml
metrics_generator:
  storage:
    remote_write:
      - url: https://localhost:9090/api/v1/write
        tls_config:
          insecure_skip_verify: true
```

---

## 🌐 Accessing Tempo

### **HTTP API**

```bash
# Health check
curl -k https://localhost:3200/ready

# Query traces (if you have trace data)
curl -k https://localhost:3200/api/search
```

### **GRPC (OTLP)**

For OpenTelemetry collectors, update to use TLS:

```yaml
exporters:
  otlp:
    endpoint: localhost:4317
    tls:
      insecure: false
      insecure_skip_verify: true
```

---

## 🔍 Verification

### **1. Check Tempo Status**

```bash
systemctl status tempo
```

Should show: `Active: active (running)`

### **2. Test HTTPS Endpoint**

```bash
curl -k https://localhost:3200/ready
```

Should return: `ready`

### **3. Test HTTP (Should Fail)**

```bash
curl http://localhost:3200/ready
```

Should fail (HTTP disabled)

### **4. Check Logs**

```bash
journalctl -u tempo -n 20 --no-pager
```

Should show no TLS errors

---

## 🔐 Security Features

### **Enabled:**
- ✅ TLS 1.2+ on HTTP API
- ✅ TLS on GRPC (OTLP receiver)
- ✅ 2048-bit RSA encryption
- ✅ HTTPS-only access
- ✅ Secure remote_write to Prometheus

---

## 📊 Impact on Services

### **Services That Need Updates:**

1. **Grafana** ⚠️ - Update Tempo datasource URL (if configured)
2. **OpenTelemetry Collectors** ⚠️ - Update OTLP exporter to use TLS
3. **Applications** ⚠️ - Update trace exporters to use TLS

### **Services Already Updated:**

- ✅ Prometheus remote_write (updated by script)

---

## 🆘 Troubleshooting

### **Tempo Won't Start**

```bash
# Check logs
journalctl -u tempo -n 50 --no-pager

# Common issues:
# - Certificate file permissions
# - Invalid config.yml syntax
# - Port already in use
```

**Fix permissions:**
```bash
sudo chown tempo:tempo /etc/tempo/tls/*
sudo chmod 600 /etc/tempo/tls/tempo.key
sudo chmod 644 /etc/tempo/tls/tempo.crt
```

### **OTLP Traces Not Working**

If you're sending traces via OTLP:

1. Update your OTLP exporter to use TLS
2. Set `insecure_skip_verify: true` for self-signed cert
3. Or use `insecure: true` to disable TLS (not recommended)

### **Rollback to HTTP**

```bash
# Restore backup
sudo cp /etc/tempo/config.yml.backup-before-tls /etc/tempo/config.yml

# Restart Tempo
sudo systemctl restart tempo
```

---

## ✅ Checklist

- [ ] Run `./setup-tempo-tls.sh`
- [ ] Verify HTTPS works: `curl -k https://localhost:3200/ready`
- [ ] Update Grafana Tempo datasource (if configured)
- [ ] Update OpenTelemetry collectors (if sending traces)
- [ ] Test trace ingestion
- [ ] Verify Prometheus remote_write working

---

## 📊 Files Created

| File | Purpose |
|------|---------|
| `setup-tempo-tls.sh` | TLS setup script |
| `TEMPO_TLS_SETUP_GUIDE.md` | This guide |

---

## 🎯 Summary

**Before:**
```
Applications → http://localhost:4317 → Tempo (OTLP)
Grafana → http://localhost:3200 → Tempo (API)
Tempo → http://prometheus:9090 → Prometheus (remote_write)
```

**After:**
```
Applications → https://localhost:4317 → Tempo (OTLP with TLS)
Grafana → https://localhost:3200 → Tempo (API with TLS)
Tempo → https://localhost:9090 → Prometheus (remote_write with TLS)
```

---

**Ready to enable TLS on Tempo? Run:**
```bash
./setup-tempo-tls.sh
```

🔐 **Secure your traces!**
