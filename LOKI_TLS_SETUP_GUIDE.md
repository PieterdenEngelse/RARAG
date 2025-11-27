# Loki TLS Setup Guide

## 🔐 Enable HTTPS on Loki

**Goal:** Secure Loki with HTTPS/TLS encryption

---

## 🚀 Quick Start

### **Step 1: Enable TLS on Loki**

```bash
./setup-loki-tls.sh
```

This will:
1. ✅ Generate self-signed TLS certificate
2. ✅ Update Loki configuration with TLS
3. ✅ Restart Loki with HTTPS
4. ✅ Verify HTTPS is working

### **Step 2: Update Vector to Use HTTPS**

```bash
./update-vector-for-loki-tls.sh
```

This will:
1. ✅ Update all Vector sinks to use HTTPS
2. ✅ Add TLS skip verify (for self-signed cert)
3. ✅ Restart Vector
4. ✅ Verify logs are flowing

### **Step 3: Update Grafana Datasource**

1. Open Grafana: http://localhost:3001
2. Go to: Connections → Data sources → Loki
3. Change URL to: `https://localhost:3100`
4. Enable: "Skip TLS Verify"
5. Click: "Save & Test"

---

## 📋 What Gets Created

### **1. TLS Certificates**

| File | Location | Purpose |
|------|----------|---------|
| Certificate | `~/.config/loki/tls/loki.crt` | Public certificate |
| Private Key | `~/.config/loki/tls/loki.key` | Private key |

### **2. Certificate Details**

- **Type:** Self-signed X.509
- **Algorithm:** RSA 2048-bit
- **Validity:** 10 years (3650 days)
- **Subject:** CN=loki.local
- **SANs:** localhost, loki.local, 127.0.0.1
- **TLS Version:** TLS 1.2 minimum

---

## 🔧 Configuration Changes

### **Loki Configuration**

**Before:**
```yaml
server:
  http_listen_port: 3100
  grpc_listen_port: 0
```

**After:**
```yaml
server:
  http_listen_port: 3100
  grpc_listen_port: 0
  http_tls_config:
    cert_file: ~/.config/loki/tls/loki.crt
    key_file: ~/.config/loki/tls/loki.key
    min_version: TLS12
```

### **Vector Configuration**

**Before:**
```toml
[sinks.loki_systemd]
type = "loki"
endpoint = "http://127.0.0.1:3100"
```

**After:**
```toml
[sinks.loki_systemd]
type = "loki"
endpoint = "https://127.0.0.1:3100"
tls.verify_certificate = false
```

---

## 🌐 Accessing Loki

### **API Endpoints**

```bash
# Health check
curl -k https://localhost:3100/ready

# Query logs
curl -k https://localhost:3100/loki/api/v1/query_range \
  --data-urlencode 'query={job="systemd-journal"}' \
  --data-urlencode 'limit=10'

# Get labels
curl -k https://localhost:3100/loki/api/v1/label/job/values
```

### **Grafana**

Update datasource:
1. URL: `https://localhost:3100`
2. Enable: "Skip TLS Verify (Insecure)"
3. Save & Test

---

## 🔍 Verification

### **1. Check Loki Status**

```bash
systemctl --user status loki
```

Should show: `Active: active (running)`

### **2. Test HTTPS Endpoint**

```bash
curl -k https://localhost:3100/ready
```

Should return: `ready`

### **3. Test HTTP (Should Fail)**

```bash
curl http://localhost:3100/ready
```

Should fail (HTTP disabled)

### **4. Verify Vector is Sending Logs**

```bash
curl -k https://localhost:3100/loki/api/v1/label/job/values
```

Should show all job labels

### **5. Check Vector Logs**

```bash
journalctl --user -u vector -n 20 --no-pager
```

Should show no TLS errors

---

## 🔐 Security Features

### **Enabled:**
- ✅ TLS 1.2 minimum
- ✅ 2048-bit RSA encryption
- ✅ HTTPS-only access
- ✅ Certificate-based encryption

### **Optional (Not Enabled):**
- ⚪ Client certificate authentication
- ⚪ Certificate pinning

---

## 📊 Impact on Services

### **Services That Need Updates:**

1. **Vector** ✅ - Updated by script
2. **Grafana** ⚠️ - Manual update required
3. **Prometheus** ⚠️ - If scraping Loki metrics

### **Services Not Affected:**

- ✅ Prometheus (separate service)
- ✅ AG backend (separate service)
- ✅ Tempo (separate service)

---

## 🔄 Updating Prometheus Scrape Config

If Prometheus scrapes Loki metrics:

```yaml
- job_name: 'loki'
  scheme: https
  tls_config:
    insecure_skip_verify: true
  static_configs:
    - targets: ['localhost:3100']
  metrics_path: '/metrics'
```

Then reload Prometheus:
```bash
sudo systemctl reload prometheus
```

---

## 🆘 Troubleshooting

### **Loki Won't Start**

```bash
# Check logs
journalctl --user -u loki -n 50 --no-pager

# Common issues:
# - Certificate file permissions
# - Invalid config.yml syntax
# - Port already in use
```

**Fix permissions:**
```bash
chmod 600 ~/.config/loki/tls/loki.key
chmod 644 ~/.config/loki/tls/loki.crt
```

### **Vector Can't Connect to Loki**

```bash
# Check Vector logs
journalctl --user -u vector -n 50 --no-pager

# Look for TLS errors
# Common issues:
# - Certificate verification failed
# - Connection refused
# - Timeout
```

**Fix:**
- Ensure `tls.verify_certificate = false` in Vector config
- Verify Loki is listening: `ss -tlnp | grep 3100`

### **No Logs in Grafana**

1. Check Grafana datasource settings
2. Verify "Skip TLS Verify" is enabled
3. Test connection in Grafana
4. Check Loki has data: `curl -k https://localhost:3100/loki/api/v1/label/job/values`

### **Rollback to HTTP**

```bash
# Restore Loki config
cp ~/.config/loki/config.yml.backup-before-tls ~/.config/loki/config.yml

# Restore Vector config
cp ~/.config/vector/vector.toml.backup-loki-tls ~/.config/vector/vector.toml

# Restart services
systemctl --user restart loki
systemctl --user restart vector
```

---

## 🔄 Renewing Certificates

Certificates expire after 10 years. To renew:

```bash
# Regenerate certificate
openssl req -new -newkey rsa:2048 -days 3650 -nodes -x509 \
    -keyout ~/.config/loki/tls/loki.key \
    -out ~/.config/loki/tls/loki.crt \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=loki.local" \
    -addext "subjectAltName=DNS:localhost,DNS:loki.local,IP:127.0.0.1"

# Fix permissions
chmod 600 ~/.config/loki/tls/loki.key
chmod 644 ~/.config/loki/tls/loki.crt

# Restart Loki
systemctl --user restart loki
```

---

## ✅ Checklist

- [ ] Run `./setup-loki-tls.sh`
- [ ] Verify HTTPS works: `curl -k https://localhost:3100/ready`
- [ ] Run `./update-vector-for-loki-tls.sh`
- [ ] Verify Vector is running: `systemctl --user status vector`
- [ ] Update Grafana datasource to HTTPS
- [ ] Test Grafana → Loki connection
- [ ] Update Prometheus scrape config (if applicable)
- [ ] Verify logs are flowing in Grafana dashboards

---

## 📊 Files Created

| File | Purpose |
|------|---------|
| `setup-loki-tls.sh` | TLS setup script |
| `update-vector-for-loki-tls.sh` | Update Vector for HTTPS |
| `LOKI_TLS_SETUP_GUIDE.md` | This guide |

---

## 🎯 Summary

**Before:**
```
Vector → http://localhost:3100 → Loki
Grafana → http://localhost:3100 → Loki
```

**After:**
```
Vector → https://localhost:3100 → Loki (TLS)
Grafana → https://localhost:3100 → Loki (TLS)
```

---

**Ready to enable TLS on Loki? Run:**
```bash
./setup-loki-tls.sh
```

🔐 **Secure your logs!**
