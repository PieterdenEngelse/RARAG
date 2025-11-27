# ✅ FINAL STEPS - Complete Distributed Tracing

## 🎯 What I've Done For You

1. ✅ **Created updated Tempo configuration** (at `/tmp/tempo-config-no-tls.yml`)
2. ✅ **Updated AG Backend .env** (changed to `http://localhost:4317`)
3. ✅ **Created all necessary scripts**

## 🚀 What You Need to Do (2 Commands)

### Step 1: Update Tempo Configuration

```bash
sudo cp /tmp/tempo-config-no-tls.yml /etc/tempo/config.yml && sudo systemctl restart tempo
```

**What this does:**
- Copies the new Tempo config (TLS removed from OTLP receiver)
- Restarts Tempo service

---

### Step 2: Restart AG Backend and Verify

```bash
cd /home/pde/ag && pkill -9 -f "target/release/ag" && sleep 3 && tmux send-keys -t main:5 "cd /home/pde/ag && ./target/release/ag" C-m && sleep 10 && curl http://localhost:3010/monitoring/health && sleep 10 && curl -sk https://localhost:3200/metrics | grep tempo_ingester_traces_created_total
```

**What this does:**
- Kills the current AG Backend
- Restarts it in tmux (picks up new config)
- Makes a test request
- Checks if traces are flowing to Tempo

---

## ✅ Expected Result

You should see output like:

```
tempo_ingester_traces_created_total{tenant="single-tenant"} 1
```

Or any number greater than 0!

---

## 🎉 If Successful

You'll have **full distributed tracing** operational:

```
AG Backend (port 3010)
     ↓ gRPC (no TLS)
Tempo OTLP Receiver (port 4317)
     ↓
Tempo Storage
```

### Next Steps:
1. **Add Tempo to Grafana:**
   - URL: `https://localhost:3200`
   - Skip TLS Verify: Yes

2. **Explore Traces:**
   - Go to Grafana → Explore
   - Select Tempo datasource
   - Search for `service.name = "ag-backend"`

3. **Generate More Traces:**
   ```bash
   curl http://localhost:3010/monitoring/health
   curl http://localhost:3010/documents
   ```

---

## 🔍 Troubleshooting

### If traces aren't showing:

**Check Tempo is running:**
```bash
sudo systemctl status tempo
```

**Check Tempo logs:**
```bash
sudo journalctl -u tempo -n 50 --no-pager
```

**Check AG Backend logs:**
```bash
tmux capture-pane -t main:5 -p | tail -n 30
```

**Verify Tempo config:**
```bash
grep -A 5 "distributor:" /etc/tempo/config.yml
```

Should show:
```yaml
distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: "0.0.0.0:4317"
          # TLS removed for OTLP receiver to allow plaintext connections
```

---

## 📁 Files Ready

- `/tmp/tempo-config-no-tls.yml` - New Tempo config (TLS removed)
- `/home/pde/ag/.env` - Updated (using http://)
- `/home/pde/ag/complete-distributed-tracing.sh` - Automated script (needs sudo)

---

## ⚡ Quick One-Liner

```bash
sudo cp /tmp/tempo-config-no-tls.yml /etc/tempo/config.yml && sudo systemctl restart tempo && cd /home/pde/ag && pkill -9 -f "target/release/ag" && sleep 3 && tmux send-keys -t main:5 "cd /home/pde/ag && ./target/release/ag" C-m && sleep 10 && curl http://localhost:3010/monitoring/health && sleep 10 && curl -sk https://localhost:3200/metrics | grep tempo_ingester_traces_created_total
```

---

## 🎯 Summary

**Status**: Ready to complete!
**Commands needed**: 1 (or 2 if you prefer step-by-step)
**Time**: 30 seconds
**Result**: Full distributed tracing! 🚀

Just run the commands above and distributed tracing will be operational!
