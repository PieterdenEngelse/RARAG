# Repository Guidelines

_Last regenerated: 2025-02-14_

---

## Project Purpose

**ag** is a Rust-based agentic RAG (Retrieval-Augmented Generation) platform. It exposes an Actix Web API for document upload/search, agent memory, tool execution, and monitoring. The project also includes a Dioxus web frontend plus extensive observability tooling (Prometheus, Grafana, Tempo, Loki, Vector) and operational scripts for distributed tracing, TLS, and logging pipelines.

---

## High-Level Architecture

```
[Clients / Frontend] ──HTTP──> [Actix Web API (src/)] ──> [Tantivy Index + SQLite]
                                      │                         │
                                      │                         ├─ vectors.json / data/
                                      │
                                      ├──> Caching Layers (src/cache/, Redis optional)
                                      ├──> Agent Memory (src/memory/, SQLite db/)
                                      └──> Monitoring Stack (Prometheus metrics, OTEL traces)

[Frontend (frontend/fro)] <──> [Backend API]
[Observability Tooling (prometheus/, grafana-*.json, scripts/, tools/qodo_web)]
```

Key runtime directories (mutable at runtime): `documents/`, `tantivy_index/`, `data/`, `db/`, `cache/`, `logs/`.

---

## Source Layout Overview

| Path | Description |
|------|-------------|
| `src/` | Main backend source (Actix server, retriever, indexing, caching, monitoring, tools). Contains modules like `api/`, `retriever.rs`, `index.rs`, `memory/`, `tools/`, `monitoring/`, `cache/`, `security/`, etc. |
| `frontend/fro/` | Dioxus-based web UI with Tailwind CSS. Includes `src/` components, `package.json`, `Dioxus.toml`, Tailwind config. |
| `tests/` | Integration and component tests: retriever, cache layer, rate limiting, tracing propagation, metrics buckets, etc. |
| `scripts/` | Helper shell scripts (cargo/test wrappers, rate-limit test runners). |
| `docs/`, `docu/`, `ops/`, numerous `*.md` guides | Extensive documentation for observability, tracing, dashboards, installers, and operational playbooks. |
| `tools/qodo_web` | Web-based Qodo helper. |
| `prometheus/`, `grafana-*.json`, `vector_*.toml`, `tempo*.sh`, `loki*.md` | Observability stack configuration and troubleshooting assets. |
| `installers/`, `install*.sh`, `setup-*.sh` | Installation and environment automation scripts (prometheus, tempo, otelcol, TLS, syslog). |

Additional notable artifacts: backup copies of config files (`*.bak`), operational logs (`ag.log`, `server.log`), and workspace metadata (`ag.code-workspace`).

---

## Build & Run Commands

### Backend (Rust)
```bash
# Run dev server
env RUST_LOG=info cargo run

# Build release binary
cargo build --release

# Run all tests (unit + integration)
cargo test
```

### Frontend (Dioxus + Tailwind)
```bash
cd frontend/fro
npm install           # once
npm run css:build     # single Tailwind build
npm run css:watch     # watch mode during dev
dx serve --platform web   # Dioxus dev server
```

### Observability Stack (selected helpers)
```bash
./install_otelcol.sh                  # Install OpenTelemetry Collector
./setup-prometheus-tls.sh             # Configure Prometheus with TLS
./setup-tempo-tls.sh                  # Tempo TLS helper
./fix-prometheus-service.sh           # Troubleshoot Prometheus systemd unit
./update-prometheus-scrape-configs.sh # Refresh scrape targets
./migrate_to_vector.sh                # Switch logging pipeline to Vector
```
(Refer to the respective `*_GUIDE.md` files for step-by-step instructions.)

---

## Coding Conventions

- Rust 2021 edition; 4-space indents.
- Modules/files use `snake_case`; structs/enums `UpperCamelCase`; constants `SCREAMING_SNAKE_CASE`.
- Prefer `rustfmt` for formatting and `cargo clippy` for linting before commits.
- Follow existing Actix patterns (extractors, App configuration) and reuse helper functions in `src/api/`.
- Frontend uses Dioxus idioms, state stored in components, Tailwind utility classes with generated CSS.

---

## Key Backend Components

| Module | Highlights |
|--------|------------|
| `src/main.rs` | Application entrypoint: loads config, initializes tracing/metrics, builds Actix server (routes, middleware, static assets). |
| `src/api/` | Route handlers for upload/search, reindex, agent endpoints, decision engine, tool composition, monitoring endpoints. |
| `src/retriever.rs` | Core retriever: Tantivy index management, vector similarity, multi-level caching, metrics instrumentation. |
| `src/index.rs` | Document ingestion pipeline (file discovery, parsing, chunking, embedding). |
| `src/embedder.rs` | Embedding generation via `llm` crate; pluggable provider. |
| `src/memory/` | Agent memory system (chunking, vector store, agent state, decision engine). |
| `src/tools/` | Tool registry/execution (calculator, web search, URL fetch, composer). |
| `src/monitoring/` | Prometheus metrics definitions, OTEL config, rate-limit middleware, health/readiness checks. |
| `src/cache/` | L2 LRU cache, optional Redis L3 cache, invalidation helpers. |
| `src/security/` | Rate limiter helper utilities. |
| `src/path_manager.rs` | Centralized path resolution for data/index assets. |

---

## Tests

- `tests/` houses integration suites: `retriever_tests.rs`, `test_cache_layer.rs`, `rate_limit_middleware_integration_test.rs`, `trace_propagation.rs`, `w3c_trace_context.rs`, metrics & rate-limit helper tests, plus E2E scaffolding (`tests/e2e_test.rs`).
- Run via `cargo test`. Individual tests can target specific files (e.g., `cargo test retriever_tests::test_search`).

---

## Observability & Ops Assets

This repository includes a large number of operational documents and scripts. Key folders/files:

- `grafana-*.json`, `GRAFANA_*` guides – dashboards for logs, traces, alerting.
- `prometheus/`, `PROMETHEUS_TLS_SETUP_GUIDE.md`, `update-prometheus-*.sh` – Prometheus setup and TLS hardening.
- `TEMPO_*`, `tempo.service.fixed`, `fix-tempo-*.sh` – Tempo (tracing backend) deployment fixes.
- `LOKI_*`, `setup-loki-tls.sh`, `vector_*.toml` – Loki log aggregation and Vector agent configs.
- `tools/qodo_web` – minimal web UI for Qodo interactions.
- `scripts/` – wrappers/tests for rate limiting and cargo commands.

These resources are invaluable when troubleshooting distributed tracing, TLS, or logging pipelines.

---

## Common Workflows

1. **Document Upload & Index**
   ```bash
   curl -F "file=@docs/sample.txt" http://127.0.0.1:3010/upload
   curl -X POST http://127.0.0.1:3010/reindex      # sync
   curl -X POST http://127.0.0.1:3010/reindex/async # async + job polling
   ```

2. **Search**
   ```bash
   curl "http://127.0.0.1:3010/search?q=rust"
   # Inspect metrics
   curl http://127.0.0.1:3010/monitoring/metrics | grep search
   ```

3. **Agent Memory**
   ```bash
   curl -X POST http://127.0.0.1:3010/memory/store_rag \
     -H 'Content-Type: application/json' \
     -d '{"agent_id":"agent1","memory_type":"observation","content":"Prefers concise summaries"}'

   curl -X POST http://127.0.0.1:3010/memory/search_rag \
     -H 'Content-Type: application/json' \
     -d '{"agent_id":"agent1","query":"preferences","top_k":5}'
   ```

4. **Monitoring & Health**
   ```bash
   curl http://127.0.0.1:3010/monitoring/health
   curl http://127.0.0.1:3010/monitoring/ready
   curl http://127.0.0.1:3010/monitoring/metrics
   ```

5. **Observability Setup**
   - Use scripts under `./setup-*.sh`, `./fix-*.sh`, `./update-*.sh` when enabling TLS, adjusting collectors, or migrating between logging pipelines.
   - Reference `TRACE_ALERTING_IMPLEMENTATION.md`, `TRACING_STATUS_SUMMARY.md`, `MULTI_SOURCE_LOGGING_SUMMARY.md`, etc., for context-specific guidance.

---

## Configuration & Environment Variables

Key env vars (see `.env.example` and `src/config.rs` for defaults):

- `BACKEND_HOST`, `BACKEND_PORT`
- `SKIP_INITIAL_INDEXING`, `INDEX_IN_RAM`
- Rate limiting knobs: `RATE_LIMIT_ENABLED`, `RATE_LIMIT_QPS`, `RATE_LIMIT_BURST`, per-route overrides, `RATE_LIMIT_LRU_CAPACITY`, `TRUST_PROXY`
- Redis cache: `REDIS_ENABLED`, `REDIS_URL`, `REDIS_TTL`
- Monitoring: `RUST_LOG`, `SEARCH_HISTO_BUCKETS`, `REINDEX_HISTO_BUCKETS`
- Alerting: `REINDEX_WEBHOOK_URL`
- OpenTelemetry: `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_SERVICE_NAME`

Secrets should stay outside version control; copy `.env.example` to `.env` and customize locally.

---

## Development Tips

- Use `scripts/cargo_wrap.sh` or `scripts/cmd_wrap.sh` when running commands in instrumented environments.
- Backups (`*.bak`) exist for many critical files; review before deleting.
- The repository contains numerous helper notes (`FINAL_FIX.md`, `PHASE_18_COMPLETE.md`, etc.) summarizing past debugging efforts—excellent references when repeating procedures.
- Observability data directories (`grafana-data`, `prometheus/`, `tempo.service.fixed`, `vector_*.toml`) may require elevated permissions or services running externally.
- When editing `AGENTS.md`, keep summaries concise but actionable—this file doubles as a quick-start for new contributors.

---

If more detailed or component-specific documentation is needed, check `docs/`, `docu/`, and the various `*_GUIDE.md` files sprinkled throughout the repository.
