# TLS Verification Explained

## ❓ Why "Skip TLS Verify"?

You asked: **"Enable 'Skip TLS Verify' - why skipped?"**

Great question! Let me explain.

---

## 🔐 What is TLS Verification?

**TLS Verification** is the process where a client (like Grafana) checks if a server's certificate (like Prometheus) is:

1. ✅ **Signed by a trusted Certificate Authority (CA)**
2. ✅ **Not expired**
3. ✅ **Matches the hostname**
4. ✅ **Not revoked**

---

## 🏢 Certificate Types

### **1. Production Certificates (Trusted CA)**

**Example:** Let's Encrypt, DigiCert, Comodo

```
Your Browser → https://google.com
              ↓
         [Checks certificate]
              ↓
    Certificate signed by DigiCert (Trusted CA)
              ↓
         ✅ Connection trusted!
```

**Verification:** ✅ **Works automatically** - No "Skip TLS Verify" needed

---

### **2. Self-Signed Certificates (What We Created)**

**Example:** Our Prometheus and Loki certificates

```
Grafana → https://localhost:9090 (Prometheus)
         ↓
    [Checks certificate]
         ↓
    Certificate signed by... itself? (Self-signed)
         ↓
    ❌ Unknown authority!
         ↓
    Error: "certificate signed by unknown authority"
```

**Verification:** ❌ **Fails by default** - Need to skip or trust manually

---

## 🤔 Why Did We Create Self-Signed Certificates?

### **Reasons:**

1. **Free** - No cost for certificates
2. **Quick** - Generated in seconds
3. **Local development** - Perfect for localhost
4. **Learning** - Good for testing TLS setup
5. **No external dependencies** - No need for domain or CA

### **Trade-offs:**

- ⚠️ **Not trusted by default** - Browsers/clients show warnings
- ⚠️ **Manual trust required** - Need to skip verification or add to trust store
- ⚠️ **Not suitable for public internet** - Only for internal/local use

---

## 🔒 What "Skip TLS Verify" Does

When you enable "Skip TLS Verify" in Grafana:

```yaml
# Grafana datasource config
tls_skip_verify: true
```

**What it does:**
- ✅ Still uses HTTPS (encrypted connection)
- ✅ Still encrypts data in transit
- ⚠️ **Skips certificate validation**
- ⚠️ Trusts any certificate (even self-signed)

**What it doesn't do:**
- ❌ Doesn't disable encryption
- ❌ Doesn't use HTTP
- ❌ Doesn't expose data in plaintext

---

## 🛡️ Security Implications

### **With TLS Verification (Production):**

```
Grafana → Prometheus
   ↓
[Verifies certificate is from trusted CA]
   ↓
✅ Trusted connection
   ↓
🔒 Encrypted + Verified
```

**Security:** ⭐⭐⭐⭐⭐ **Excellent**
- Protected from man-in-the-middle attacks
- Certificate must be valid and trusted

---

### **With "Skip TLS Verify" (Our Setup):**

```
Grafana → Prometheus
   ↓
[Skips certificate verification]
   ↓
⚠️ Trusts any certificate
   ↓
🔒 Encrypted but not verified
```

**Security:** ⭐⭐⭐ **Good** (for local/internal use)
- Still encrypted (data not readable in transit)
- Vulnerable to man-in-the-middle IF attacker is on local network
- **Acceptable for:**
  - Local development
  - Internal networks
  - Trusted environments
  - Services on localhost

**Not acceptable for:**
- Public internet
- Untrusted networks
- Production with external access

---

## 🎯 Our Use Case

### **Current Setup:**

```
Grafana (localhost:3001)
   ↓ HTTPS (self-signed cert, skip verify)
Prometheus (localhost:9090)
   ↓ HTTPS (self-signed cert, skip verify)
Loki (localhost:3100)
```

**Why this is OK:**

1. ✅ **All on localhost** - Same machine, no network exposure
2. ✅ **Internal use** - Not exposed to internet
3. ✅ **Encrypted** - Data is still encrypted
4. ✅ **Learning/Development** - Perfect for testing
5. ✅ **No sensitive external data** - Monitoring your own system

**Risk level:** 🟢 **Low** (for this use case)

---

## 🔄 Alternatives to "Skip TLS Verify"

### **Option 1: Add Certificate to Trust Store**

Instead of skipping verification, add the self-signed cert to Grafana's trust store:

```bash
# Copy certificate
sudo cp ~/.config/loki/tls/loki.crt /usr/local/share/ca-certificates/loki.crt

# Update CA certificates
sudo update-ca-certificates

# Restart Grafana
sudo systemctl restart grafana-server
```

**Pros:**
- ✅ Proper certificate validation
- ✅ No "skip verify" needed

**Cons:**
- ⚠️ More complex setup
- ⚠️ Need to do for each service
- ⚠️ Need to update when cert expires

---

### **Option 2: Use Production Certificates**

Get real certificates from Let's Encrypt or internal CA:

```bash
# Example with certbot (requires domain)
sudo certbot certonly --standalone -d prometheus.yourdomain.com
```

**Pros:**
- ✅ Fully trusted
- ✅ No warnings
- ✅ Production-ready

**Cons:**
- ⚠️ Requires domain name
- ⚠️ Requires public DNS
- ⚠️ More complex setup
- ⚠️ Overkill for localhost

---

### **Option 3: Use Internal CA**

Create your own Certificate Authority:

```bash
# Create CA
openssl genrsa -out ca.key 2048
openssl req -x509 -new -nodes -key ca.key -days 3650 -out ca.crt

# Sign certificates with your CA
openssl x509 -req -in prometheus.csr -CA ca.crt -CAkey ca.key -out prometheus.crt
```

**Pros:**
- ✅ Proper certificate chain
- ✅ Can be trusted organization-wide

**Cons:**
- ⚠️ Complex setup
- ⚠️ Need to distribute CA cert
- ⚠️ Overkill for single machine

---

## 📊 Comparison

| Method | Security | Complexity | Cost | Best For |
|--------|----------|------------|------|----------|
| **Self-signed + Skip Verify** | ⭐⭐⭐ | 🟢 Easy | Free | Local/Dev |
| **Self-signed + Trust Store** | ⭐⭐⭐⭐ | 🟡 Medium | Free | Internal |
| **Let's Encrypt** | ⭐⭐⭐⭐⭐ | 🟡 Medium | Free | Public |
| **Internal CA** | ⭐⭐⭐⭐⭐ | 🔴 Hard | Free | Enterprise |
| **Commercial CA** | ⭐⭐⭐⭐⭐ | 🟡 Medium | $$$ | Production |

---

## ✅ Recommendation for Your Setup

**For localhost monitoring (your current use case):**

✅ **Use self-signed certificates + Skip TLS Verify**

**Why:**
- Perfect for local development
- Still provides encryption
- Simple to set up
- No external dependencies
- Acceptable security for localhost

**When to upgrade:**
- 🔄 If exposing services to network
- 🔄 If deploying to production
- 🔄 If handling sensitive external data
- 🔄 If compliance requires it

---

## 🎓 Summary

### **"Skip TLS Verify" means:**

✅ **Still encrypted** - HTTPS is used
✅ **Still secure** - For localhost/internal use
⚠️ **Trusts any certificate** - Including self-signed
⚠️ **Not for production internet** - Use real certs there

### **Why we use it:**

1. We created **self-signed certificates**
2. Self-signed certs are **not trusted by default**
3. We're on **localhost** (low risk)
4. It's **simpler** than managing CA trust stores
5. It's **appropriate** for development/internal use

### **The encryption is still there!**

```
Without TLS:
Grafana → [plaintext data] → Prometheus ❌

With TLS + Skip Verify:
Grafana → [encrypted data] → Prometheus ✅
          (just not verifying who signed the cert)
```

---

## 🔐 Bottom Line

**"Skip TLS Verify" doesn't mean "no security"**

It means:
- ✅ Encryption: **YES**
- ✅ HTTPS: **YES**
- ⚠️ Certificate validation: **NO**

For localhost monitoring, this is **perfectly acceptable**! 🎯

---

**Questions?** This is a great security question to ask! 👍
