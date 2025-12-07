You’re thinking in exactly the right direction: you want **two tiers** of observability:

1. **Built‑in monitoring** that “just works” with `cargo run` or the binary – **no Prometheus/Grafana needed**.
2. **Optional full stack** (Prometheus + Grafana + Loki + Tempo) for people who want deep history, logs, traces.

Here’s how to structure that so installation for other users stays simple.

---

## 1. Define two modes explicitly

### Mode A – “Self‑contained” (default)

- Requirements: only the `ag` binary + Dioxus frontend.
- Users get:
  - Monitor UI tiles (Requests, Cache, Index, Rate Limits, Logs, System Health)
  - Live summaries (current p95 latency, rates, counts, index size, status)
  - A tiny amount of in‑memory history if you want (e.g. last 5–15 minutes)

**No** Prometheus/Grafana install or config required.

### Mode B – “Observability integrated” (optional)

- Requirements: your existing stack (Prometheus, Grafana, Loki, Tempo).
- Users additionally get:
  - Long‑term history & multi‑service views in Grafana
  - Full log search in Loki
  - Trace exploration & anomaly dashboards in Tempo
  - Deep dashboards you already have under `src/monitoring/dashboards/ag/*`

This mode should be enabled by **simple flags/env vars** or an installer wizard, not mandatory.

---

## 2. What to put “inside the project” for Mode A

Everything that today goes into Prometheus gauges/counters can also be surfaced **directly by `ag`**:

### Backend: add UI‑oriented endpoints

In `src/monitoring/` (or a new `ui_metrics.rs`), add a few HTTP endpoints such as:

- `GET /monitoring/ui/requests`
- `GET /monitoring/ui/index`
- `GET /monitoring/ui/rate_limits`
- `GET /monitoring/ui/health`
- (optionally) `GET /monitoring/ui/logs-pipeline` (very high‑level status, not full logs)

Each returns **simple JSON** built from:

- The Prometheus client registry you already use for `/monitoring/metrics`, or
- Internal app state (index size, docs, reindex status, etc.)

Example for `/monitoring/ui/requests`:

```json
{
  "request_rate_rps": 42.3,
  "latency_p95_ms": 45.0,
  "error_rate_percent": 0.12
}
```

No Prometheus server is required here – you’re just reading the in‑process metrics.

### Frontend: Dioxus monitor pages talk only to the backend

In `frontend/fro/src/pages/monitor_*.rs`:

- Replace hardcoded numbers with fetches to `/monitoring/ui/...`.
- Show:
  - Loading state
  - “No data yet” if endpoint returns empty
  - Error state if `/monitoring/ui/...` is not available

Now, **any user** who just clones the repo and runs `cargo run` gets:

- A working Monitor section
- Real live metrics from `ag` itself
- No Prometheus/Grafana needed

This becomes the **installation story** you want.

---

## 3. How to plug in Prometheus/Grafana without making them mandatory

Prometheus/Grafana become **optional enhancers**, not requirements.

### Backend integration

Behind your `/monitoring/ui/...` endpoints, you have two options:

- **Simple mode (default)**:  
  Read from the Prometheus registry (counters, histograms) in memory. This uses only `prometheus` crate, no Prometheus server.

- **Advanced mode (when configured)**:  
  Optionally query Prometheus’ HTTP API for:
  - Longer windows (last 1h, 6h)
  - More complex rollups

Use a config flag like:

```rust
struct ObservabilityConfig {
    prometheus_url: Option<String>, // None = self-contained mode
}
```

If `prometheus_url` is `None`:

- `/monitoring/ui/...` serves “live” metrics from memory only.

If it’s `Some("http://localhost:9090")`:

- `/monitoring/ui/...` can also use PromQL queries to give slightly richer info (e.g., real rates over 5m windows).

The **endpoint shape** stays the same for the frontend; only the backend data source changes.

### Frontend integration

In Dioxus:

- Always talk only to `ag` (never directly to Prometheus/Grafana).
- On each tile, you can add a **“View in Grafana”** link *if* an env flag says Grafana is configured.

Example:

```rust
if grafana_enabled {
    a {
        href: "http://grafana.example.com/d/ag-latency",
        class: "text-xs text-teal-400 underline ml-2",
        "Open in Grafana"
    }
}
```

For users who never run Grafana:

- They still see all Dioxus tiles populated from `/monitoring/ui/...`.
- They just won’t see / won’t use the Grafana links.

---

## 4. What goes in your installer / docs

Given you already have an `installer` module, your installation flow can be:

### Quick start (self-contained)

- Requirements: Rust + `ag` binary.
- Steps:

  ```bash
  cargo run
  # or use the prebuilt binary
  ```

- Results:
  - Backend: `http://127.0.0.1:3010`
  - Frontend: `http://127.0.0.1:1789`
  - Monitor pages all working with live, in‑process metrics.

No Prometheus, no Grafana, no Loki, no Tempo.

### Advanced observability (optional)

- Optional step in installer or docs:

  ```bash
  ./scripts/install-observability.sh
  ```

  which might:
  - Start docker-compose for Prometheus/Grafana/Loki/Tempo
  - Apply `ag.yaml`, `datasources.yaml` under `/etc/grafana`
  - Set envs like `PROMETHEUS_URL=http://localhost:9090`, `GRAFANA_URL=http://localhost:3000`

- Then:
  - Dioxus still works as before
  - Monitor tiles might show a bit more history
  - Grafana dashboards become available via links

---

## 5. Summary relative to your goal

> “Getting as much as possible in project doesn’t need Prometheus/Grafana install/config needed for other users.”

You can achieve that by:

- **Designing the Dioxus Monitor UI and `/monitoring/ui/*` endpoints to depend only on `ag`’s own in‑process metrics**, not on external Prometheus/Grafana.
- Treating Prometheus/Grafana as a **layer on top**:
  - Extra history
  - Extra dashboards
  - Optional “Open in Grafana” buttons.

So for other users:

- A basic **install = just your binary + frontend**.
- **No external observability stack required** to get a useful Monitor experience.
