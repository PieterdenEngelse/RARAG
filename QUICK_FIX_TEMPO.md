# Quick Fix for Tempo Tracing

## The Problem
Traces are being received by Tempo but not stored due to internal gRPC TLS mismatch.

## The Solution (3 commands)

### 1. Edit Tempo Configuration
```bash
sudo nano /etc/tempo/config.yml
```

**Find this section:**
```yaml
server:
  http_listen_port: 3200
  log_level: info
  http_tls_config:
    cert_file: /etc/tempo/tls/tempo.crt
    key_file: /etc/tempo/tls/tempo.key
  grpc_tls_config:              # ← DELETE THESE 3 LINES
    cert_file: /etc/tempo/tls/tempo.crt
    key_file: /etc/tempo/tls/tempo.key
```

**Delete the `grpc_tls_config` section** (3 lines), save and exit (Ctrl+X, Y, Enter)

### 2. Restart Tempo
```bash
sudo systemctl restart tempo
sudo systemctl status tempo
```

### 3. Test It
```bash
# Generate some traces
for i in {1..5}; do curl -s http://127.0.0.1:3010/monitoring/health > /dev/null; sleep 1; done

# Check if traces are being stored (wait 10 seconds first)
sleep 10
curl -sk https://localhost:3200/metrics | grep tempo_ingester_traces_created_total
```

**Expected output:**
```
tempo_ingester_traces_created_total{tenant="single-tenant"} 5
```

If you see a number > 0, **it's working!** 🎉

## Verify in Grafana

1. Open Grafana: http://localhost:3001
2. Add Tempo datasource:
   - URL: `https://localhost:3200`
   - Skip TLS Verify: ✅
3. Go to Explore → Tempo
4. Search for service: `ag-backend`

---

**Full documentation**: See `OPENTELEMETRY_TRACING_COMPLETE_GUIDE.md`
