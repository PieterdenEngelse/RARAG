# Repository Guidelines

## Project Structure & Module Organization

The Actix Web backend lives in `backend/src/` with feature-focused modules (`api/`, `retriever.rs`, `monitoring/`, `memory/`, `tools/`, etc.) driven by `backend/Cargo.toml`. Shared integration and regression tests sit in `backend/tests/` (e.g., `retriever_tests.rs`, `rate_limit.rs`, `trace_propagation.rs`). The Dioxus web client occupies `frontend/fro/` with its own Cargo package plus `package.json`, `tailwind.config.js`, and component tree under `src/`. Operational playbooks, installer notes, and observability references are collected in `docs/`, `docu/`, and `ops/`. Runtime artifacts such as `documents/`, `db/`, `tantivy_index/`, `cache/`, and `logs/` are created on demand; keep them out of version control.

## Build, Test, and Development Commands

```bash
# Backend development server with structured logs
cd backend && env RUST_LOG=info cargo run

# Backend release build and test suite
cd backend && cargo build --release && cargo test

# Frontend toolchain (Tailwind + Dioxus)
cd frontend/fro && npm install && npm run css:build && dx serve --platform web
```

## Coding Style & Naming Conventions

- **Indentation**: Rust, TOML, and JS/TS use 4 spaces (no tabs); Tailwind config follows Prettier defaults.
- **File naming**: Rust modules are `snake_case` (`retriever.rs`, `agent_memory.rs`); components and structs use `UpperCamelCase`; constants are `SCREAMING_SNAKE_CASE`.
- **Function/variable naming**: Prefer `snake_case` in Rust, camelCase inside frontend TypeScript/JS glue, and kebab-case for config filenames.
- **Linting**: Run `cargo fmt` plus `cargo clippy --all-targets -- -D warnings` before committing; the frontend relies on `rustfmt` (via `dx fmt`) and Tailwind’s generated CSS.

## Testing Guidelines

- **Framework**: Native Rust test harness with async support; integration targets live in `backend/tests/`.
- **Test files**: Mirrors runtime modules (`retriever_tests.rs`, `rate_limit_middleware_integration_test.rs`). Use `cargo test retriever_tests::search` to scope.
- **Running tests**: `cd backend && cargo test` exercises unit + integration suites; add `-- --ignored` when exercising long-running observability specs.
- **Coverage**: No enforced threshold, but CI docs in `docu/PLAN.md` expect smoke tests before release tagging.

## Commit & Pull Request Guidelines

- **Commit format**: Keep short imperative subjects (see `git log` entries such as `phase 17 completed` or `29-12-25`). Reference affected subsystems when helpful (e.g., `monitoring: tighten OTLP retry`).
- **PR process**: Reference the relevant design note (`docu/PLAN.md`, tracing guides, etc.), attach `cargo test` output, and describe any feature flags touched (`OTEL_*`, `RATE_LIMIT_*`).
- **Branch naming**: Follow the existing `feature/<scope>` pattern noted in `docu/AGENTS.md`; reserve `main` for release-ready code.

---

# Repository Tour

## 🎯 What This Repository Does

**ag** is a Rust-based agentic Retrieval-Augmented Generation platform exposing Actix Web APIs for document ingestion, search, and agent memory, plus a Dioxus/Tailwind frontend and an observability toolchain (Prometheus, Grafana, Tempo, Loki).

**Key responsibilities:**
- Accept, chunk, and index documents into Tantivy and vector stores.
- Serve low-latency semantic search, rerank, summarize, and agent-memory endpoints.
- Emit metrics/traces/logs for the included monitoring stack and scripts under `prometheus/`, `grafana-*.json`, and `tools/`.

---

## 🏗️ Architecture Overview

### System Context
```
[Dioxus Web UI / CLI clients]
          │ HTTP (CORS + rate limits)
          ▼
[Actix Web API (backend/src)] ──► [Tantivy index + rusqlite DB]
          │                               │
          ├─► [Redis L3 cache (optional)] │
          └─► [OTel exporter] ─► [OTel Collector] ─► [Tempo / Grafana]
```

### Key Components
- **Actix server (`backend/src/main.rs`)** – Boots config, tracing, database schema, retriever, Redis cache, and spawns non-blocking indexing.
- **API layer (`backend/src/api/`)** – Upload/search routes, async reindex jobs, chunk/LLM/hardware config endpoints, agent memory handlers, monitoring surfaces (`/monitoring/*`).
- **Retriever & indexing (`backend/src/retriever.rs`, `index.rs`, `chunker.rs`)** – Manage Tantivy writers, chunkers, cache tiers, background indexing, and metrics gauges.
- **Monitoring (`backend/src/monitoring/`)** – Prometheus histograms, OTLP configuration, trace alerting, rate-limit middleware, dashboards, and scripts.
- **Frontend (`frontend/fro/src/`)** – Dioxus 0.6 components, monitoring widgets, and Tailwind-generated styles that call the API through `src/api.rs` helpers.

### Data Flow
1. Client uploads or queries over HTTP; TraceMiddleware tags request IDs and captures spans.
2. Rate-limit middleware classifies routes using env/JSON rules before hitting handlers.
3. Handlers delegate to Retriever/Chunker or to the SQLite-backed agent memory layer; Tantivy writes commit once per batch.
4. Metrics are exported from `/monitoring/metrics`, traces stream via OTLP to Collector/Tempo, and optional webhooks fire on reindex completion.

---

## 📁 Project Structure [Partial Directory Tree]

```
.
├── backend/
│   ├── Cargo.toml              # Actix/LLM crate definition
│   ├── src/
│   │   ├── api/
│   │   ├── monitoring/
│   │   ├── memory/
│   │   └── retriever.rs
│   └── tests/                  # Integration + reliability suites
├── frontend/
│   └── fro/
│       ├── Cargo.toml          # Dioxus web app
│       ├── package.json        # Tailwind CLI scripts
│       └── src/
├── docs/                       # Targeted how-to guides
├── docu/                       # Living design/plan documents
├── prometheus/                 # Scrape configs & TLS helpers
├── tools/                      # Auxiliary apps (e.g., qodo_web)
├── scripts/                    # Installer/diagnostic shell scripts
├── docker-compose.observability.yml
└── AGENTS.md
```

### Key Files to Know

| File | Purpose | When You'd Touch It |
|------|---------|---------------------|
| `backend/src/main.rs` | Orchestrates startup (config, tracing, retriever, background indexing). | Changing boot order, logging, or global toggles. |
| `backend/src/api/mod.rs` | Defines every HTTP route, async reindex, monitoring endpoints, rate-limit wiring. | Adding new endpoints or tweaking middleware. |
| `backend/src/retriever.rs` | Tantivy writer settings, cache tiers, search/rerank logic. | Optimizing indexing/search performance. |
| `backend/src/monitoring/otel_config.rs` | OTLP exporter initialization + env parsing. | Pointing traces to new collectors/backends. |
| `backend/tests/retriever_tests.rs` | Regression tests for indexing/search consistency. | Safeguarding search changes. |
| `frontend/fro/src/main.rs` | Dioxus router/bootstrap. | Adding views/routes on the web UI. |
| `frontend/fro/package.json` | Tailwind build/watch scripts. | Adjusting CSS pipeline or dependencies. |
| `.env.example` | Documented runtime knobs (chunking, tracing, rate limits). | Creating new env templates or onboarding. |
| `docker-compose.observability.yml` | Spins up Prometheus + Grafana quick-start stack. | Local observability testing. |
| `scripts/setup-prometheus-tls.sh` | Automates Prometheus TLS hardening. | Rotating certs or rebuilding secure scrape endpoints. |
| `docu/PLAN.md` | Phase-by-phase implementation log. | Understanding historical trade-offs before refactors. |

---

## 🔧 Technology Stack

### Core Technologies
- **Language:** Rust 2021 edition (workspace root + backend/frontend crates) for backend correctness and shared types.
- **Backend Framework:** Actix Web 4.11 (async extractors, middleware) paired with Tokio 1.47 runtime.
- **Retrieval Layer:** Tantivy 0.24.2 for inverted index search plus custom chunkers; Rusqlite 0.37 for metadata and agent memory.
- **Frontend:** Dioxus 0.6.3 compiled to web/WASM, styled with Tailwind CLI 4.1.14 and DaisyUI.
- **Observability:** Tracing + OpenTelemetry SDK (0.21), Prometheus client 0.13, Grafana dashboards, Tempo for traces, Loki/Vector assets for logs.
- **Caching:** Optional Redis (`redis` crate 0.32) as L3 cache behind in-process L1/L2 caches.

### Key Libraries
- `llm 1.3.4` for embedding generation and local model hooks.
- `serde`/`serde_json` for all API payloads and config serialization.
- `reqwest 0.12` (backend) and `gloo-net`/`reqwest 0.11` (frontend) for outbound HTTP.
- `tracing-subscriber`, `tracing-opentelemetry`, and `prometheus` crates for instrumentation.

### Development Tools
- Cargo workspace with `cargo run|build|test`, `cargo fmt`, and `cargo clippy`.
- Dioxus CLI (`dx serve`) plus Tailwind CLI for frontend hot reload.
- Shell automation under `scripts/` for TLS setup, tracing verification, and collector restarts.

---

## 🌐 External Dependencies

### Required Services
- **Prometheus** (`prometheus/`, `update-prometheus-scrape-configs.sh`) – Scrapes `/monitoring/metrics`; TLS-ready configs provided.
- **Grafana** (`grafana-*.json`) – Imports dashboards for latency, rate limits, and trace-alerting.
- **Tempo** (`tempo.service.fixed`, `setup-tempo-tls.sh`) – Receives OTLP spans from the backend or collector.
- **Redis** (optional) – Configurable via `REDIS_ENABLED` and `REDIS_URL` for L3 cache.

### Optional Integrations
- **Loki + Vector** (`vector_*.toml`, `setup-loki-tls.sh`) – Structured log shipping; toggled via provided scripts.
- **Webhook Targets** (`REINDEX_WEBHOOK_URL`) – Notify external systems when indexing jobs finish.

---

### Environment Variables

```bash
# Core runtime
BACKEND_HOST=127.0.0.1
BACKEND_PORT=3010
SKIP_INITIAL_INDEXING=false
INDEX_IN_RAM=false

# Chunking & search tuning
CHUNKER_MODE=fixed|lightweight|semantic
CHUNK_TARGET_SIZE=384
CHUNKING_SNAPSHOT_LOGGING=true

# Rate limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_QPS=1.0
RATE_LIMIT_SEARCH_QPS=10
RATE_LIMIT_UPLOAD_QPS=2
RATE_LIMIT_LRU_CAPACITY=1024
TRUST_PROXY=false

# Tracing & observability
OTEL_TRACES_ENABLED=true
OTEL_OTLP_EXPORT=true
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4318
TEMPO_ENABLED=false
RESOURCE_ATTRIBUTION_ENABLED=true

# Caching
REDIS_ENABLED=true
REDIS_URL=redis://127.0.0.1:6379/
REDIS_TTL=3600
```

---

## 🔄 Common Workflows

### Document upload and ingestion
1. `curl -F "file=@docs/sample.txt" http://127.0.0.1:3010/upload`
2. Monitor background indexing via `/monitoring/metrics` (histograms + gauges) or `/monitoring/chunking/latest`.
3. If SKIP_INITIAL_INDEXING was set, kick off `curl -X POST http://127.0.0.1:3010/reindex/async` and poll `/reindex/status/{job}`.

**Code path:** `api/mod.rs::upload_document_inner` → `index::index_file` → `retriever::commit`.

### Search & agent workflows
1. Query search: `curl "http://127.0.0.1:3010/search?q=rust"` (rate limited as “search”).
2. Store agent memory: POST to `/memory/store_rag`; retrieve with `/memory/search_rag` (SQLite via `agent_memory.rs`).
3. The frontend reuses `frontend/fro/src/api.rs` helpers to call these endpoints with Dioxus hooks.

### Observability bootstrap
1. `docker-compose -f docker-compose.observability.yml up -d` to start Prometheus + Grafana (uses provided provisioning files).
2. Run `./complete-tracing-setup.sh` or `./update-tempo-config.sh` to align Collector ↔ Tempo TLS expectations.
3. Import `grafana-*.json` dashboards for latency, trace alerting, and multi-source logging views.

---

## 📈 Performance & Scale

- **Rate limiting**: Middleware-driven per-route token buckets (configurable via JSON/env) protect heavy endpoints (`/upload`, `/reindex`) and report drops through Prometheus counters.
- **Background indexing**: Startup spawns indexing on a dedicated task; `SKIP_INITIAL_INDEXING=true` keeps boot times low while manual reindex remains available.
- **Histogram tuning**: `SEARCH_HISTO_BUCKETS` and `REINDEX_HISTO_BUCKETS` accept comma-separated millisecond thresholds; invalid tokens fall back to defaults with warnings.
- **Caching tiers**: L1/L2 in-memory caches plus optional Redis L3 reduce Tantivy lookups; monitoring endpoints expose cache hit ratios.

### Monitoring
- Prometheus scrape target: `http://<host>:3010/monitoring/metrics`.
- Grafana dashboards in `grafana-*.json` visualize latency, error budgets, and rate-limit activity; alert rules live alongside under `docs/TRACE_ALERTING*.md`.

---

## 🚨 Things to Be Careful About

> **Repository hygiene**: Never assume files are backed up in git—treat the working tree as the only source of truth and copy anything critical before experimenting.

### 🔒 Security Considerations
- **Proxy awareness**: Leave `TRUST_PROXY=false` unless you terminate TLS behind a trusted reverse proxy; otherwise rate limiting and IP logging may be spoofed.
- **API keys storage**: `/config/api_keys` endpoints persist in SQLite; `.env` and `agent.db` backups exist in repo root—treat them as secrets.
- **Tracing TLS**: Scripts such as `setup-tempo-tls.sh` and `setup-prometheus-tls.sh` modify system configs; run them with care and capture backups (`*.bak` files already exist).
- **Observability credentials**: Dashboard JSONs assume local Grafana without auth; harden when exposing externally.

*Update to last commit: d549cdb6f6eb667c1783a36be6b35a7a91af16a2*
