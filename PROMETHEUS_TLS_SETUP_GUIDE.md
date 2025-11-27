# Prometheus TLS Setup Guide

## 🔐 Phase 19: Enable TLS on Prometheus

**Goal:** Secure Prometheus with HTTPS/TLS encryption

---

## 🎯 What This Does

### **Before:**
```
http://localhost:9090  ← Unencrypted HTTP
```

### **After:**
```
https://localhost:9090  ← Encrypted HTTPS with TLS 1.2+
```

---

## 🚀 Quick Start

### **Step 1: Run TLS Setup**

```bash
./setup-prometheus-tls.sh
```

This will:
1. ✅ Generate self-signed TLS certificate
2. ✅ Create Prometheus web config
3. ✅ Update systemd service
4. ✅ Restart Prometheus with TLS
5. ✅ Verify HTTPS is working

### **Step 2: Update Scrape Configs**

```bash
./update-prometheus-scrape-configs.sh
```

This will:
1. ✅ Update Prometheus to scrape itself via HTTPS
2. ✅ Validate configuration
3. ✅ Reload Prometheus

### **Step 3: Verify**

```bash
# Test HTTPS endpoint
curl -k https://localhost:9090/-/healthy

# Should return: Prometheus Server is Healthy.
```

---

## 📋 What Gets Created

### **1. TLS Certificates**

| File | Location | Purpose |
|------|----------|---------|
| Certificate | `/etc/prometheus/tls/prometheus.crt` | Public certificate |
| Private Key | `/etc/prometheus/tls/prometheus.key` | Private key |
| Web Config | `/etc/prometheus/tls/web-config.yml` | TLS configuration |

### **2. Certificate Details**

- **Type:** Self-signed X.509
- **Algorithm:** RSA 2048-bit
- **Validity:** 10 years (3650 days)
- **Subject:** CN=prometheus.local
- **SANs:** localhost, prometheus.local, 127.0.0.1
- **TLS Version:** TLS 1.2 minimum

### **3. Cipher Suites**

Strong ciphers only:
- TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
- TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
- TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
- TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384

---

## 🔧 Configuration Changes

### **Systemd Service**

**Before:**
```ini
ExecStart=/usr/local/bin/prometheus \
  --config.file=/etc/prometheus/prometheus.yml \
  --storage.tsdb.path=/var/lib/prometheus/
```

**After:**
```ini
ExecStart=/usr/local/bin/prometheus \
  --config.file=/etc/prometheus/prometheus.yml \
  --storage.tsdb.path=/var/lib/prometheus/ \
  --web.config.file=/etc/prometheus/tls/web-config.yml
```

### **Scrape Config**

**Before:**
```yaml
- job_name: 'prometheus'
  static_configs:
    - targets: ['localhost:9090']
```

**After:**
```yaml
- job_name: 'prometheus'
  scheme: https
  tls_config:
    insecure_skip_verify: true
  static_configs:
    - targets: ['localhost:9090']
```

---

## 🌐 Accessing Prometheus

### **Web Browser**

```
https://localhost:9090
```

**Note:** You'll see a security warning because it's a self-signed certificate.

**To bypass:**
- Chrome/Edge: Click "Advanced" → "Proceed to localhost"
- Firefox: Click "Advanced" → "Accept the Risk and Continue"

### **curl**

```bash
# With -k flag (insecure, skips cert verification)
curl -k https://localhost:9090/api/v1/status/config

# Or specify the certificate
curl --cacert /etc/prometheus/tls/prometheus.crt \
     https://localhost:9090/api/v1/status/config
```

### **Grafana**

Update Prometheus datasource:
1. Go to: Configuration → Data Sources → Prometheus
2. Change URL to: `https://localhost:9090`
3. Enable: "Skip TLS Verify" (for self-signed cert)
4. Save & Test

---

## 🔍 Verification

### **1. Check Service Status**

```bash
systemctl status prometheus
```

Should show: `Active: active (running)`

### **2. Test HTTPS Endpoint**

```bash
curl -k https://localhost:9090/-/healthy
```

Should return: `Prometheus Server is Healthy.`

### **3. Test HTTP (Should Fail)**

```bash
curl http://localhost:9090/-/healthy
```

Should fail or show error (HTTP disabled)

### **4. Check Certificate**

```bash
openssl s_client -connect localhost:9090 -showcerts
```

Should show certificate details

### **5. Verify Scraping**

```bash
curl -k https://localhost:9090/api/v1/targets
```

Should show all targets and their status

---

## 🔐 Security Features

### **Enabled:**
- ✅ TLS 1.2 minimum (TLS 1.0/1.1 disabled)
- ✅ Strong cipher suites only
- ✅ 2048-bit RSA encryption
- ✅ Perfect Forward Secrecy (ECDHE)
- ✅ AES-GCM encryption

### **Optional (Not Enabled):**
- ⚪ Client certificate authentication
- ⚪ Certificate pinning
- ⚪ HSTS headers

---

## 📊 Impact on Other Services

### **Services That Need Updates:**

1. **Grafana** - Update Prometheus datasource URL
2. **Alertmanager** - Update Prometheus URL (if configured)
3. **Custom scripts** - Update any scripts using Prometheus API

### **Services Not Affected:**

- ✅ Loki (separate service)
- ✅ Vector (separate service)
- ✅ AG backend (separate service)
- ✅ Node exporter (separate service)

---

## 🔄 Updating Grafana Datasource

### **Manual Update:**

1. Open Grafana: http://localhost:3001
2. Go to: ☰ → Connections → Data sources
3. Click: Prometheus
4. Update URL: `https://localhost:9090`
5. Scroll down to "TLS/SSL Settings"
6. Enable: "Skip TLS Verify (Insecure)"
7. Click: "Save & Test"

### **Or via API:**

```bash
# Get datasource UID
curl -s http://localhost:3001/api/datasources | jq '.[] | select(.type=="prometheus") | .uid'

# Update datasource (replace UID)
curl -X PUT http://localhost:3001/api/datasources/uid/<UID> \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://localhost:9090",
    "jsonData": {
      "tlsSkipVerify": true
    }
  }'
```

---

## 🆘 Troubleshooting

### **Prometheus Won't Start**

```bash
# Check logs
journalctl -u prometheus -n 50 --no-pager

# Common issues:
# - Certificate file permissions
# - Invalid web-config.yml syntax
# - Port already in use
```

**Fix permissions:**
```bash
sudo chown prometheus:prometheus /etc/prometheus/tls/*
sudo chmod 600 /etc/prometheus/tls/prometheus.key
sudo chmod 644 /etc/prometheus/tls/prometheus.crt
```

### **HTTPS Not Working**

```bash
# Check if Prometheus is listening on 9090
ss -tlnp | grep 9090

# Test with verbose curl
curl -kv https://localhost:9090/-/healthy
```

### **Certificate Errors**

```bash
# Verify certificate
openssl x509 -in /etc/prometheus/tls/prometheus.crt -text -noout

# Check expiry
openssl x509 -in /etc/prometheus/tls/prometheus.crt -noout -dates
```

### **Rollback to HTTP**

```bash
# Restore backup
sudo cp /etc/systemd/system/prometheus.service.backup-tls \
       /etc/systemd/system/prometheus.service

# Reload and restart
sudo systemctl daemon-reload
sudo systemctl restart prometheus
```

---

## 🔄 Renewing Certificates

Self-signed certificates expire after 10 years. To renew:

```bash
# Regenerate certificate
sudo openssl req -new -newkey rsa:2048 -days 3650 -nodes -x509 \
    -keyout /etc/prometheus/tls/prometheus.key \
    -out /etc/prometheus/tls/prometheus.crt \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=prometheus.local" \
    -addext "subjectAltName=DNS:localhost,DNS:prometheus.local,IP:127.0.0.1"

# Fix permissions
sudo chown prometheus:prometheus /etc/prometheus/tls/*
sudo chmod 600 /etc/prometheus/tls/prometheus.key

# Restart Prometheus
sudo systemctl restart prometheus
```

---

## 📝 Next Steps (Optional)

### **1. Enable Client Certificate Authentication**

Add to `web-config.yml`:
```yaml
tls_server_config:
  client_auth_type: RequireAndVerifyClientCert
  client_ca_file: /etc/prometheus/tls/ca.crt
```

### **2. Use Production Certificates**

Replace self-signed with Let's Encrypt or internal CA:
```bash
# Example with certbot
sudo certbot certonly --standalone -d prometheus.yourdomain.com
```

### **3. Enable TLS on Other Services**

- Node Exporter
- Alertmanager
- Grafana
- Loki

---

## ✅ Checklist

- [ ] Run `./setup-prometheus-tls.sh`
- [ ] Verify HTTPS works: `curl -k https://localhost:9090/-/healthy`
- [ ] Run `./update-prometheus-scrape-configs.sh`
- [ ] Update Grafana datasource to HTTPS
- [ ] Test Grafana → Prometheus connection
- [ ] Update any custom scripts/tools
- [ ] Document certificate location for team
- [ ] Set calendar reminder for certificate renewal (10 years)

---

## 📊 Files Created

| File | Purpose |
|------|---------|
| `setup-prometheus-tls.sh` | TLS setup script |
| `update-prometheus-scrape-configs.sh` | Update scrape configs |
| `PROMETHEUS_TLS_SETUP_GUIDE.md` | This guide |

---

**Ready to enable TLS? Run:**
```bash
./setup-prometheus-tls.sh
```

🔐 **Secure your metrics!**
