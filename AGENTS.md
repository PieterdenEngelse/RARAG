# Repository Guidelines

## Project Structure & Module Organization

- Backend (Rust): ./src – Actix Web API, retriever, indexing, monitoring, security, cache, etc. Key modules:
  - src/api (HTTP routes and handlers)
  - src/retriever.rs, src/index.rs, src/embedder.rs
  - src/monitoring (metrics, tracing, rate limiting middleware)
  - src/cache (L2 memory cache, Redis L3 cache)
  - src/config.rs, src/path_manager.rs
  - src/db/schema_init.rs (database bootstrapping)
- Frontend (Dioxus/Tailwind): ./frontend/fro with Rust UI in src/, Dioxus.toml, Tailwind config, and Node scripts.
- Tests: ./tests (integration, monitoring, rate-limit, cache tests).
- Runtime/data: ./documents (uploads), ./tantivy_index, ./data, ./db, ./cache, ./logs.
- Scripts and installers: ./install.sh, ./installers/, ./scripts/.

## Build, Test, and Development Commands

```bash
# Backend: run (dev)
cargo run

# Backend: build release
cargo build --release

# Backend: run tests
cargo test

# Frontend: build CSS once
cd frontend/fro && npm run css:build

# Frontend: watch CSS during dev
cd frontend/fro && npm run css:watch

# Frontend: serve Dioxus app (requires dx CLI)
cd frontend/fro && dx serve --platform web
```

## Coding Style & Naming Conventions

- Indentation: 4 spaces (Rust and JS).
- Rust style: modules and files in snake_case (e.g., src/path_manager.rs), types in UpperCamelCase, functions/variables in snake_case, consts in SCREAMING_SNAKE_CASE.
- Frontend config uses common JS conventions (camelCase identifiers; see tailwind.config.js).
- Lint/format: rustfmt and clippy recommended; no explicit config files present. Follow Actix/Dioxus idioms.

## Testing Guidelines

- Framework: Rust built-in test framework (cargo test).
- Locations: integration/system tests in ./tests (e.g., retriever_tests.rs, rate_limit_middleware_integration_test.rs). Unit tests may appear alongside modules with #[cfg(test)].
- Run: cargo test
- Coverage: No explicit coverage tooling or thresholds configured.

## Commit & Pull Request Guidelines

- Commit format: Not enforced; write clear, imperative messages describing the why and what. Reference modules when relevant (e.g., monitoring: add Prometheus endpoint).
- PR process: Include rationale, test results (cargo test), and any env or config changes affecting monitoring/rate limits. Update AGENTS.md when altering APIs or structure.
- Branches: Feature branches under feature/<short-name> are present in git refs; use descriptive names.

---

# Repository Tour

## 🎯 What This Repository Does

ag is a Rust-based Agentic RAG service that exposes an Actix Web API for document upload, indexing, search, simple rerank/summarize, and monitoring; it includes a Dioxus + Tailwind frontend.

Key responsibilities:
- Index and search text/PDF files using Tantivy and simple embeddings
- Serve HTTP endpoints for upload, search, reindex (sync/async), agent memory
- Expose metrics and tracing for observability; optional Redis cache layer

---

## 🏗️ Architecture Overview

### System Context
```
[Browser / Client] → [Actix Web backend (ag)] → [Rusqlite/Tantivy on disk]
                             ↓
                     [Dioxus Web Frontend]
```

### Key Components
- HTTP API (src/api): Upload, reindex (sync/async), search, memory, agent endpoints; global retriever handle.
- Retriever/Indexing (src/retriever.rs, src/index.rs): Tantivy index, vector storage JSON, hybrid utilities, atomic reindex.
- Monitoring (src/monitoring): metrics export (/monitoring/metrics), health/ready, OpenTelemetry setup, rate limit middleware.
- Caching (src/cache): L2 in-memory cache and optional L3 Redis cache.
- Config/Paths (src/config.rs, src/path_manager.rs): env-driven server, rate limits, and managed data/index paths.
- Frontend (frontend/fro): Dioxus app; Tailwind for styling and asset pipeline.

### Data Flow
1. Client uploads files to /upload; files stored under ./documents.
2. Reindex is triggered (background at startup unless SKIP_INITIAL_INDEXING=true, or via /reindex or /reindex/async).
3. src/index.rs parses supported files (txt/pdf placeholder), chunks lines, embeds, and writes to Tantivy + vectors.json.
4. /search queries Tantivy; optional caches (L2/L3) and metrics are updated; response returned as JSON.

---

## 📁 Project Structure [Partial Directory Tree]

```
./
├── Cargo.toml
├── src/
│   ├── api/                      # Actix routes, handlers, async reindex jobs
│   ├── cache/                    # L2 memory cache + Redis L3 cache
│   ├── monitoring/               # Metrics, OpenTelemetry, rate limit middleware
│   ├── db/schema_init.rs         # DB schema bootstrap (rusqlite)
│   ├── config.rs                 # ApiConfig from env
│   ├── retriever.rs              # Tantivy index + vector storage and metrics
│   ├── index.rs                  # Indexing pipeline over ./documents
│   ├── path_manager.rs           # Paths for data/index/vector files
│   └── lib.rs                    # Module declarations
├── tests/                        # Integration and monitoring tests
├── frontend/
│   └── fro/
│       ├── Dioxus.toml
│       ├── package.json          # Tailwind scripts (css:build, css:watch)
│       ├── tailwind.config.js
│       └── src/                  # Dioxus components/pages
├── documents/                    # Uploads (runtime)
├── tantivy_index/                # Local Tantivy index (runtime)
├── data/ db/ cache/ logs/        # Runtime data and logs
└── install.sh installers/ scripts/
```

### Key Files to Know

| File | Purpose | When You'd Touch It |
|------|---------|---------------------|
| src/main.rs | Backend entrypoint; sets up tracing, OTEL, DB, retriever; starts Actix server | Change startup behavior, background indexing, telemetry |
| src/api/mod.rs | HTTP routes for health/ready, upload, search, reindex (sync/async), memory, agent | Add endpoints or adjust rate limits/CORS |
| src/retriever.rs | Core search/index and vector store logic; metrics; caches; atomic reindex | Modify search behavior, vector IO, caching |
| src/index.rs | Iterates files in ./documents, extracts text, chunks, embeds, commits | Extend file types, chunking, embeddings |
| src/monitoring/metrics.rs | Prometheus metrics registry and exporters | Add/edit metrics |
| src/monitoring/rate_limit_middleware.rs | Middleware with per-route rules and labels | Tune rate limits and exempt routes |
| src/config.rs | ApiConfig from env (ports, rate limits, Redis) | Introduce new env flags or defaults |
| frontend/fro/package.json | Tailwind CSS scripts | Update CSS build/watch scripts |
| frontend/fro/Dioxus.toml | Dioxus web config and watcher | Adjust dev file watching |
| frontend/fro/tailwind.config.js | Tailwind scanning paths, dark mode | Add content paths and theme |

---

## 🔧 Technology Stack

### Core Technologies
- Language: Rust (edition 2021)
- Backend: Actix Web 4.x
- Search/Index: Tantivy 0.24.x
- Persistence: rusqlite 0.37 (SQLite)
- Async runtime: Tokio 1.x
- Telemetry: tracing, prometheus, OpenTelemetry 0.21 (+ OTLP via tonic)
- Frontend: Dioxus (Rust) with Tailwind CSS 4.x CLI

### Key Libraries
- llm for model interactions (placeholder usage in codebase)
- rayon and lru for performance and caching
- reqwest (rustls) for HTTP client utilities
- redis (optional) for L3 cache

### Development Tools
- cargo (build/test)
- dx (Dioxus CLI) for frontend serve
- Tailwind CLI via npm scripts (frontend/fro)

---

## 🌐 External Dependencies

- Prometheus (scrapes /monitoring/metrics)
- Optional: Redis at REDIS_URL for L3 cache
- Optional: OpenTelemetry collector via OTLP (see src/monitoring/* OTLP files)

### Environment Variables

```bash
# Server
BACKEND_HOST=127.0.0.1
BACKEND_PORT=3010

# Indexing & startup
SKIP_INITIAL_INDEXING=false
INDEX_IN_RAM=false

# Rate limiting
RATE_LIMIT_ENABLED=false
RATE_LIMIT_QPS=1.0
RATE_LIMIT_BURST=5
RATE_LIMIT_SEARCH_QPS=
RATE_LIMIT_SEARCH_BURST=
RATE_LIMIT_UPLOAD_QPS=
RATE_LIMIT_UPLOAD_BURST=
RATE_LIMIT_LRU_CAPACITY=1024
TRUST_PROXY=false

# Redis (optional)
REDIS_ENABLED=false
REDIS_URL=redis://127.0.0.1:6379/
REDIS_TTL=3600
```

---

## 🔄 Common Workflows

### Local backend development
1. Set env vars as needed (e.g., RUST_LOG, BACKEND_PORT).
2. cargo run
3. Hit http://127.0.0.1:3010/monitoring/health and /monitoring/metrics.

### Index local documents
1. Place .txt or .pdf files under ./documents
2. POST /reindex or POST /reindex/async
3. GET /index/info and /documents to verify

### Frontend workflow
1. cd frontend/fro
2. npm run css:watch (Tailwind)
3. dx serve --platform web

---

## 📈 Performance & Scale

- Batch indexing supported via begin_batch/end_batch to reduce commit overhead.
- L1/L2/L3 caching layers reduce query latency; metrics include cache hit/miss and search latency histograms.
- Histogram buckets configurable in monitoring code; Prometheus export is text format.

---

## 🚨 Things to Be Careful About

### 🔒 Security Considerations
- If TRUST_PROXY is false, remote IPs are taken from the socket; only enable behind trusted proxy.
- Upload endpoint accepts only .txt and .pdf; enforce size and auth if exposing publicly.
- Redis and any webhook URLs should be configured via env; avoid committing secrets.


*Updated at: 2025-11-15*
