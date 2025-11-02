# Repository Guidelines

## Project Structure & Module Organization

- Backend (Rust) lives in src/ with modules declared in src/lib.rs:
  - src/main.rs – Actix Web bootstrap, monitoring, PathManager, Redis L3, indexing
  - src/api/ – HTTP routes (health/ready/metrics, upload/search, memory, cache, tools)
  - src/retriever.rs – Tantivy index, vector store, caches, metrics
  - src/config.rs – ApiConfig (env + PathManager)
  - src/monitoring/, src/cache/, src/memory/, src/db/, src/index/, src/tools/, src/agent/ – feature modules
- Tests in tests/ (e2e and module tests where present)
- Data/working dirs (created/used at runtime):
  - documents/ – uploaded or local files to index
  - tantivy_index/, test_tantivy_index/ – search index directories
  - vectors.json – vector storage; agent.db – SQLite (legacy memory)
- Frontend (Dioxus) at frontend/fro with assets/, public/, src/

## Build, Test, and Development Commands

```bash
# Backend: run (dev)
cargo run

# Backend: build release
cargo build --release

# Backend: run tests
cargo test

# Frontend: serve with Dioxus (hot reload)
cd frontend/fro && dx serve --platform web

# Frontend: build CSS (Tailwind v4 CLI)
cd frontend/fro && npx @tailwindcss/cli -i ./assets/styling/input.css -o ./public/styles.css
```

## Coding Style & Naming Conventions

- Indentation: 4 spaces (rustfmt default)
- Rust files and modules: snake_case (e.g., src/path_manager.rs, src/api/mod.rs)
- Types/structs: CamelCase; functions/variables: snake_case
- Lint/format:
  - cargo fmt --all
  - cargo clippy --all-targets -- -D warnings

## Testing Guidelines

- Framework: Rust built-in test harness
- Test files: tests/*.rs and inline #[cfg(test)] in modules
- Running tests: cargo test
- Coverage: Not specified

## Commit & Pull Request Guidelines

- Commit messages: concise, imperative (no enforced convention found)
  - Examples: "Initialize Redis L3 cache handlers"; "Fix health check vector mapping"
- PRs: include summary, affected modules (e.g., src/api/, src/retriever.rs), and test results
- Branch naming: not specified

---

# Repository Tour

## 🎯 What This Repository Does

ag (agentic-rag) is a Rust-based Agentic RAG system exposing an Actix Web API with Tantivy-backed keyword search, a simple vector store, multi-layer caching (L1/L2/optional L3 Redis), and a Dioxus frontend.

Key responsibilities:
- Index and search documents stored in documents/ with Tantivy and vectors
- Provide REST endpoints for upload, search, RAG memory, and cache operations
- Serve a Dioxus/Tailwind UI demonstrating queries against the backend

---

## 🏗️ Architecture Overview

### System Context
```
Browser (Dioxus UI) → Actix Web API → Tantivy index, vectors.json, agent.db
                               ↓
                          Optional Redis L3
```

### Key Components
- API Server (src/api/) – Actix routes: health/ready/metrics, upload/reindex/search, summarize/rerank, vector save; memory APIs; cache routes; tools/composer/decision-engine hooks
- Retriever (src/retriever.rs) – Tantivy index management, vector store, L1 LRU cache, L2 TTL memory cache, optional Redis L3; metrics and health/ready checks
- Config & Paths (src/config.rs + path_manager) – env config, centralized paths
- Monitoring (src/monitoring/) – tracing initialization, log file/console config
- Data storage – tantivy_index/ (index), vectors.json (vectors), agent.db (legacy RAG memory)
- Frontend (frontend/fro) – Dioxus app consuming the API

### Data Flow
1. Files placed in documents/ or POST /upload are reindexed via POST /reindex
2. Tantivy index and vectors.json are updated; mappings maintained in retriever
3. Clients call /search (and optionally rerank/summarize) to retrieve results
4. Caching: L1/L2 always available; L3 Redis used if enabled in env

---

## 📁 Project Structure [Partial Directory Tree]

```
.
├── Cargo.toml                 # Backend dependencies (actix-web, tantivy, rusqlite, redis, tracing)
├── src/
│   ├── main.rs               # Startup: monitoring, config, DB schema, retriever, API server
│   ├── lib.rs                # Module declarations
│   ├── api/                  # HTTP routes (health, search, memory, cache, tools)
│   ├── retriever.rs          # Tantivy + vectors + caches + metrics
│   ├── config.rs             # ApiConfig, env, PathManager wiring
│   ├── monitoring/           # Tracing/log setup (init_tracing, MonitoringConfig)
│   ├── db/                   # schema_init for agent.db
│   ├── memory/               # Vector store API (Phase 4)
│   ├── tools/                # Tool routes
│   ├── index/                # Document indexing helpers
│   └── cache/                # L2/L3 cache implementations
├── documents/                # Uploaded/ingested files
├── tantivy_index/            # Tantivy index data directory
├── vectors.json              # Vector storage file
├── agent.db                  # SQLite for legacy memory
├── tests/                    # e2e and module tests
└── frontend/
    └── fro/                  # Dioxus frontend (assets, public, src, package.json)
```

### Key Files to Know

| File | Purpose | When You'd Touch It |
|------|---------|---------------------|
| src/main.rs | Application bootstrap and server run | Change ports, startup indexing, monitoring |
| src/api/mod.rs | Defines routes/endpoints | Add new API endpoints or adjust handlers |
| src/retriever.rs | Search index, vectors, caches | Tune search/caching, fix health/metrics |
| src/config.rs | Env config + PathManager | Update env variables and path logic |
| Cargo.toml | Backend deps and features | Add/remove libraries |
| frontend/fro/Dioxus.toml | Frontend config | Static files and watcher paths |
| frontend/fro/package.json | Tailwind CLI scripts | Adjust CSS build or node deps |
| tests/*.rs | Tests | Add/modify test coverage |

---

## 🔧 Technology Stack

### Core Technologies
- Language: Rust 2021 (Cargo project)
- Backend: Actix Web
- Search: Tantivy
- Persistence: vectors.json (JSON), SQLite via rusqlite (agent.db for legacy memory)
- Async: Tokio; Parallelism: Rayon
- Caching: L1 LRU (lru), L2 TTL memory cache; optional L3 Redis (redis crate)
- Frontend: Dioxus (Rust/Web) + Tailwind CSS 4 CLI

### Key Libraries (from Cargo.toml)
- actix-web, actix-cors, actix-multipart – HTTP server, CORS, file uploads
- tantivy – indexing and search
- rusqlite – SQLite access for legacy memory
- serde/serde_json – serialization
- tracing, tracing-subscriber, tracing-appender – logging/metrics
- tokio, rayon – async/parallel compute
- redis – optional L3 cache

### Development Tools
- rustfmt, clippy – formatting and linting
- Dioxus CLI (dx) – frontend dev server
- Tailwind CLI – stylesheet build

---

## 🌐 External Dependencies

- Redis (optional): enable with REDIS_ENABLED=true and configure REDIS_URL; TTL via REDIS_TTL

### Environment Variables

```bash
BACKEND_HOST=127.0.0.1
BACKEND_PORT=3010
REDIS_ENABLED=false
REDIS_URL=redis://127.0.0.1:6379/
REDIS_TTL=3600
```

---

## 🔄 Common Workflows

- Index local documents
  - Place .txt/.pdf under documents/ → POST /reindex
  - Or POST /upload then POST /reindex
- Search
  - GET /search?q=your+query
- Health and readiness
  - GET /health, /ready, /metrics
- Cache management
  - GET /cache/stats, POST /cache/clear-l2, POST /cache/log
  - Redis: GET /cache/redis/status, /cache/redis/info; POST /cache/redis/ping
- Legacy RAG memory
  - POST /memory/store_rag, /memory/search_rag, /memory/recall_rag

Code path: src/api/mod.rs routes → src/retriever.rs operations → tantivy_index/ + vectors.json

---

## 📈 Performance & Scale

- Caching layers reduce repeated query cost (L1/L2 in-memory, optional Redis L3)
- Batch indexing supported via Retriever::begin_batch/end_batch for large corpora
- Metrics available through retriever.metrics and /metrics endpoint

---

## 🚨 Things to Be Careful About

- Ensure documents/ and tantivy_index/ are writable; vectors.json path must be writable
- Health check enforces vector/index consistency; run /reindex after manual file changes
- Set correct BACKEND_HOST/BACKEND_PORT; frontend CORS allows localhost:3011 by default
- Redis is optional; failures log warnings and continue without L3

## 🛠️ Troubleshooting (Backend Indexing & Reindex)

This section covers common issues encountered during document indexing and the atomic reindex flow.

1) POST /reindex returns "IO error: No such file or directory (os error 2)"
- Cause: Missing directories for data/index/locks or parent dir of vectors.json.
- Fixes implemented in code: reindex_atomic now ensures all required directories exist before writes (create_dir_all on PathManager dirs and vectors parent).
- What you can do:
  - Verify directories exist and are writable:
    - ~/.local/share/ag/data
    - ~/.local/share/ag/index (and subdirs)
    - ~/.local/share/ag/locks
  - Ensure the process runs with permissions to create and rename files in these paths.

2) Health shows unhealthy_repairable: Manifest/file mismatch (manifest(v,m) vs file(v,m))
- Cause: Live vectors.json and manifest.json counts diverged, often after an interrupted reindex.
- Fix: Run POST /reindex to rebuild into temp files and atomically swap live artifacts.
- Notes: Pre-swap validation ensures vectors == mappings; manifest.next.json is generated and then moved to manifest.json during swap.

3) Vector/mapping mismatch in memory (vectors != doc_id_to_vector_idx)
- Cause: Historical writes without mapping or interrupted saves.
- Behavior: On save, a parity repair runs to add default IDs for unmapped vectors, then persists.
- What to do: POST /reindex to rebuild clean state. Health will fail if on-disk or in-memory mismatches exist until you reindex.

4) Where do logs go and how to increase verbosity?
- Tracing is enabled; typical logs print to stdout/stderr. If you run via tmux or a system service, redirect output to a file, e.g., /tmp/ag.out.
- To increase verbosity: set environment variable RUST_LOG=info (or debug) before running cargo run.
- Key checkpoints during reindex:
  - /reindex: START/SUCCESS/ERROR/END
  - index_all_documents: considering file=…, indexed file=…
  - Pre-swap: Temp vectors saved/read; Pre-swap validation OK: vectors=N, mappings=N; manifest.next.json written
  - Swap: renaming tmp/live index, vectors, and manifest

5) Tantivy warning: Merge cancelled in tantivy.tmp after swap
- Symptom: WARN about NotFound in tantivy.tmp during a merge right after the swap.
- Explanation: During atomic swap, tmp index dir is moved to live; ongoing background merge on the tmp path can emit a cancelled/NotFound warning.
- Action: Safe to ignore if /reindex succeeded and /health is healthy.

6) Server claims no documents or indexing appears to do nothing
- Verify working directory and UPLOAD_DIR alignment: the server expects documents/ relative to its working directory when started.
- Check /documents endpoint to confirm files discovered.
- Ensure files have supported extensions (.txt, .pdf; PDF parsing currently stubbed).

7) Still stuck?
- Run with detailed logs and capture reindex output:
  - RUST_LOG=info cargo run > /tmp/ag.out 2>&1
  - curl -s -X POST http://127.0.0.1:3010/reindex | jq .
  - tail -n 200 /tmp/ag.out
- Look for index_file and pre-swap logs to pinpoint the failure.


*Update to last commit: 5afa66a6c59d0dedde1bc8d7b4a9dd39088d20d0*
*Last updated: 2025-10-26*
