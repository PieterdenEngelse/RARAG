# Loki + Vector Alerts and Tuning

This document describes the **Vector → Loki** alerting setup, why each alert exists, and how to tune it for a single-node dev laptop (and later, more production-like setups).

The core metrics used are exported by Vector:

- `vector_processing_errors_total{component_id="loki"}` – total processing errors in the Loki sink
- `vector_component_received_events_total{component_id="journald"}` – events received from journald
- `vector_component_sent_events_total{component_id="loki"}` – events sent to Loki

The main alerts are:

1. **VectorLokiProcessingErrors** – processing errors in Loki sink
2. **VectorLokiNoTraffic** – journald has logs, Loki does not
3. **VectorLokiIngestionSpike** – Loki ingestion spike
4. **How to tune thresholds – based on real traffic**
5. **Concrete recommended configuration for this setup**

---

## 1. VectorLokiProcessingErrors

**Rule:**

```yaml
- alert: VectorLokiProcessingErrors
  expr: rate(vector_processing_errors_total{component_id="loki"}[5m]) > 0
  for: 5m
  labels:
    severity: warning
    service: ag-backend
    component: vector-loki
  annotations:
    summary: "Vector → Loki: processing errors"
    description: |
      Vector is reporting processing errors for the Loki sink.
      Check vector.service logs and Loki status.
```

**What it means:**

- Any non-zero rate of `vector_processing_errors_total` over 5 min indicates that Vector repeatedly failed to send events to Loki.
- On a dev laptop, this usually means something is broken (Loki down, wrong URL, schema problem, etc.).

**Tuning advice:**

- For this setup: keep `> 0` and `for: 5m`. You want to know about any persistent errors.
- If you ever see transient noise (e.g. during restarts), you can:
  - Increase `for:` to `10m`, or
  - Use a higher threshold, e.g. `> 0.1`, if you expect occasional one-off errors.

---

## 2. VectorLokiNoTraffic

**Rule (original):**

```yaml
- alert: VectorLokiNoTraffic
  expr: |
    rate(vector_component_received_events_total{component_id="journald"}[5m]) > 0
    and rate(vector_component_sent_events_total{component_id="loki"}[5m]) < 0.01
  for: 10m
  labels:
    severity: warning
    service: ag-backend
    component: vector-loki
  annotations:
    summary: "Logs are not reaching Loki from Vector"
    description: |
      Vector is receiving logs from journald but is not sending them to Loki.
      Check vector.service and loki.service for errors.
```

**What it means:**

- Journald source is active: `rate(...journald...[5m]) > 0` (logs are being read).
- Loki sink is effectively idle: `rate(...loki...[5m]) < 0.01` (no logs sent).
- Condition holds for the entire `for:` window.

This catches the case where the log pipeline is **broken**, but the source is still producing logs.

**Tuning advice for the laptop:**

- If you want to reduce noise when you are not actively using `ag`, increase the `for:` duration:

  ```yaml
  for: 15m
  ```

- Threshold `< 0.01` events/sec is effectively "no traffic"; it is fine to keep.
- For a more production-like environment, you might shorten `for:` to `5m` to get faster detection.

---

## 3. VectorLokiIngestionSpike

**Rule (original):**

```yaml
- alert: VectorLokiIngestionSpike
  expr: rate(vector_component_sent_events_total{component_id="loki"}[5m]) > 1000
  for: 5m
  labels:
    severity: info
    service: ag-backend
    component: vector-loki
  annotations:
    summary: "High Loki ingestion rate via Vector"
    description: |
      Loki is receiving more than 1000 events per second from Vector.
      This may be expected during load tests, but review capacity and retention if sustained.
```

**What it means:**

- Loki is receiving > 1000 events per second via Vector for at least 5 minutes.
- It is informational (`severity: info`) and aimed at capacity planning.

**Tuning advice:**

- On a dev laptop, 1000 eps is probably never reached. You can:
  - Lower the threshold, e.g. to 100 eps:

    ```yaml
    expr: rate(vector_component_sent_events_total{component_id="loki"}[5m]) > 100
    ```

  - Or comment this rule out entirely if you don’t care about ingestion spikes on the laptop.

- In a future production setting, keep a higher threshold (1000+, 5000+) based on real traffic and possibly raise `severity` to `warning` if sustained spikes are costly.

---

## 4. How to tune thresholds based on real data

Before finalizing thresholds, look at real metrics in Prometheus:

```promql
# Errors in Loki sink
rate(vector_processing_errors_total{component_id="loki"}[5m])

# Throughput: journald → Vector
rate(vector_component_received_events_total{component_id="journald"}[5m])

# Throughput: Vector → Loki
rate(vector_component_sent_events_total{component_id="loki"}[5m])
```

Guidelines:

- Observe typical values during normal operation (e.g. journald rate around 1–5 eps, Loki rate similar).
- Set "no traffic" threshold (`< 0.01`) much lower than the usual Loki rate.
- Set ingestion spike threshold (e.g. `> 100`) high enough that it indicates an abnormal increase compared to your baseline.
- Adjust `for:` duration to balance sensitivity vs. noise:
  - Shorter `for:` detects issues faster but may trigger for transient blips.
  - Longer `for:` reduces noise but delays alerts.

---

## 5. Recommended configuration for this setup

For the current single-node dev laptop, a reasonable tuned configuration is:

```yaml
groups:
  - name: vector-loki.alerts
    interval: 30s
    rules:
      # 1) Processing errors – important, keep sensitive
      - alert: VectorLokiProcessingErrors
        expr: rate(vector_processing_errors_total{component_id="loki"}[5m]) > 0
        for: 5m
        labels:
          severity: warning
          service: ag-backend
          component: vector-loki
        annotations:
          summary: "Vector → Loki: processing errors"
          description: |
            Vector is reporting processing errors for the Loki sink.
            Check vector.service logs and Loki status.

      # 2) No logs to Loki while journald is still active – slightly more relaxed
      - alert: VectorLokiNoTraffic
        expr: |
          rate(vector_component_received_events_total{component_id="journald"}[5m]) > 0
          and rate(vector_component_sent_events_total{component_id="loki"}[5m]) < 0.01
        for: 15m
        labels:
          severity: warning
          service: ag-backend
          component: vector-loki
        annotations:
          summary: "Vector → Loki: no traffic, journald still active"
          description: |
            Vector is receiving logs from journald but is not sending them to Loki.
            Check vector.service and loki.service for connectivity and errors.

      # 3) Ingestion spike – optional for dev, threshold lowered to 100 eps
      - alert: VectorLokiIngestionSpike
        expr: rate(vector_component_sent_events_total{component_id="loki"}[5m]) > 100
        for: 5m
        labels:
          severity: info
          service: ag-backend
          component: vector-loki
        annotations:
          summary: "Vector → Loki: high ingestion rate"
          description: |
            Loki is receiving more than 100 events per second from Vector.
            This might be expected under load, but review capacity and retention if sustained.
```

This configuration:

- Alerts **quickly** on real errors pushing to Loki.
- Alerts after a **moderate delay** if logs stop reaching Loki while journald is still active.
- Optionally warns if log volume suddenly increases well above normal.

You can evolve these thresholds as you observe real traffic patterns and decide how noisy vs. sensitive you want your alerts to be.
