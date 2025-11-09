# Repository Guidelines

i dont use docker.

## Project Structure & Module Organization

- Root Rust backend: ./src (Actix Web services, monitoring, retriever, config) with Cargo.toml defining dependencies and features. Supporting docs: README.md, INSTALLER.md, PLAN.md.
- Frontend (Dioxus/Tailwind): ./frontend/fro with Rust UI (src/), Dioxus.toml, Tailwind config, and Node-based tooling (package.json) plus assets/ and public/.
- Tests: ./tests for Rust unit/integration tests (see PLAN/INSTALLER notes and crate layout).
- Ops and scripts: ./install.sh (cross-platform installer entry), ./scripts (utility scripts), ops assets referenced in INSTALLER.md (systemd/windows) may live under ops/ when present.
- Data/runtime folders: ./data, ./db, ./cache, ./logs, tantivy_index (local artifacts), and target/ (build outputs).
- Qodo workspace: ./.qodo/{agents,workflows} for automation.

## Build, Test, and Development Commands

```bash
# Backend: run (dev)
cargo run

# Backend: run with profiling stubs enabled
cargo run --features profiling

# Backend: build release
cargo build --release

# Backend: tests
cargo test

# Frontend (Dioxus): serve for web
cd frontend/fro && dx serve --platform web

# Tailwind (if needed for CSS generation)
cd frontend/fro && npx tailwindcss -i ./assets/styling/input.css -o ./public/styles.css --watch
```

## Coding Style & Naming Conventions

- Indentation: 4 spaces for Rust and JS/TS files unless project-specific formatting is applied.
- File naming: Rust modules use snake_case (e.g., src/api/mod.rs, src/retriever.rs); JS config files in frontend/fro use kebab/camel as per ecosystem (tailwind.config.js).
- Functions/variables: snake_case for Rust, UpperCamelCase for types/structs, SCREAMING_SNAKE_CASE for consts; in JS config, camelCase.
- Linting/formatting: Use rustfmt and clippy where available. Frontend adheres to Tailwind and Dioxus conventions; no explicit ESLint/Prettier configs detected.

## Testing Guidelines

- Framework: Rust’s built-in test framework (cargo test). Some modules include integration tests under ./tests (see PLAN.md and INSTALLER.md references to test commands).
- Test files: Rust unit tests live alongside modules with #[cfg(test)], integration tests under tests/ directory.
- Running tests: cargo test
- Coverage: No explicit coverage thresholds configured in the repo.

## Commit & Pull Request Guidelines

- Commit format: Conventional commits not enforced; use clear, imperative messages. Examples from docs indicate versioned changes like "config.rs v2.0.0" and feature flags; include scope and impact (e.g., "feat: add rate limiting with metrics").
- PR process: Open PR with description of changes, include build/test results and any feature flags used. Ensure docs (README.md/INSTALLER.md/this AGENTS.md) are updated when behavior changes.
- Branch naming: Feature branches observed under refs/heads/feature/...; use feature/<short-name>.

---

# Repository Tour

## 🎯 What This Repository Does

ag (Agentic RAG) is a Rust-based Retrieval-Augmented Generation system with an Actix Web backend and a Dioxus (WASM) frontend, providing search, indexing, and monitoring endpoints.

Key responsibilities:
- Serve API endpoints for search, indexing, uploads, and monitoring
- Manage vector/text indices (tantivy) and persistence (rusqlite)
- Provide a Dioxus web UI with Tailwind styling

---

## 🏗️ Architecture Overview

### System Context
```
[Browser / Client] → [Actix Web backend (ag)] → [Rusqlite/Tantivy storage]
                             ↓
                    [Dioxus Web Frontend]
```

### Key Components
- Backend (./src): Actix Web app exposing routes like /search, /upload, /monitoring/* with tracing and metrics.
- Indexing/Retriever: Tantivy-based indexing/search with configurable writer heap; supports async reindex and rate limiting as documented.
- Monitoring: Prometheus metrics and OpenTelemetry tracing (Jaeger integration), plus health/ready/live endpoints.
- Frontend (./frontend/fro): Dioxus 0.6 app compiled to web, Tailwind v4-based styling.

### Data Flow
1. Client calls API endpoints on Actix Web (e.g., /search or /upload).
2. Middleware adds tracing, rate-limiting, and metrics; handlers validate and process.
3. Indexing/retrieval uses Tantivy and Rusqlite for data access and search.
4. Responses are serialized via serde and returned; metrics exposed at /monitoring/metrics.

---

## 📁 Project Structure [Partial Directory Tree]

```
./
├── Cargo.toml                 # Rust workspace crate for backend
├── README.md                  # Logging presets and profiling notes
├── INSTALLER.md               # Cross-platform installation & ops
├── install.sh                 # Installer entry (bash)
├── src/                       # Backend source (Actix, monitoring, retriever, config)
├── tests/                     # Rust integration tests
├── frontend/
│   └── fro/
│       ├── Cargo.toml         # Dioxus app dependencies
│       ├── Dioxus.toml        # Dioxus web app config
│       ├── package.json       # Tailwind tooling
│       ├── tailwind.config.js # Tailwind config (dark mode class)
│       └── src/               # Dioxus UI components/views
├── data/ db/ cache/ logs/     # Runtime data folders
├── target/                    # Build artifacts (Rust)
└── .qodo/agents,workflows     # Qodo automation
```

### Key Files to Know

| File | Purpose | When You'd Touch It |
|------|---------|---------------------|
| Cargo.toml | Backend dependencies, features (installer, profiling, full) | Add/change backend deps or features |
| src/main.rs (and src/api/mod.rs) | Backend entry and API routes | Add endpoints or middleware |
| src/monitoring/* | Metrics, tracing, health endpoints | Modify observability behavior |
| frontend/fro/Cargo.toml | Dioxus UI dependencies | Add UI libraries |
| frontend/fro/Dioxus.toml | Frontend app config | Change web build settings |
| frontend/fro/package.json | Node tooling (Tailwind) | Adjust scripts/tooling |
| frontend/fro/tailwind.config.js | Tailwind scanning/config | Extend styling/theme |
| install.sh | Installer flow and env | Change install behavior/paths |
| README.md | Logging presets and profiling flags | Update dev/ops guidance |
| INSTALLER.md | End-to-end install and ops docs | Update deployment details |

---

## 🔧 Technology Stack

### Core Technologies
- Language: Rust (edition 2021) from Cargo.toml
- Backend Framework: Actix Web 4.x
- Frontend Framework: Dioxus 0.6 (web, router)
- Search/Index: Tantivy 0.24
- Database/Storage: rusqlite 0.37, filesystem paths managed via env
- Async: Tokio 1.x, futures-util
- Serialization: serde/serde_json (optional serde_yaml via feature)
- Telemetry: tracing, opentelemetry (+jaeger), tracing-subscriber, Prometheus 0.13

### Key Libraries
- llm (1.3.4) for LLM interactions
- regex, rayon, lru for data processing and performance
- actix-cors, actix-multipart, actix-service for HTTP stack

### Development Tools
- cargo (build/test)
- dx (Dioxus CLI) for frontend serve/build
- Tailwind CLI 4.x via Node in frontend/fro

---

## 🌐 External Dependencies

- Optional Redis (redis 0.32 with async features) enabled by env as per INSTALLER and Cargo features.
- Jaeger (optional) for tracing export when TRACING_ENABLED/JAEGER_* envs are set.
- Prometheus scrapes /monitoring/metrics for metrics collection.

### Environment Variables

Common variables from README/INSTALLER:

```bash
RUST_LOG=info,tantivy=warn
SEARCH_HISTO_BUCKETS=1,2,5,10,20,50,100,250,500,1000
REINDEX_HISTO_BUCKETS=50,100,250,500,1000,2000,5000,10000
TRUST_PROXY=true|false
# Installer-generated
AG_HOME=~/.fro
BACKEND_PORT=3010
FRONTEND_PORT=3000
REDIS_URL=redis://127.0.0.1:6379/
```

---

## 🔄 Common Workflows

### Local development (backend)
1. Set logging preset (e.g., RUST_LOG=debug,tantivy=info)
2. cargo run
3. Verify health endpoints at /monitoring/{health,ready,live}

### Frontend development
1. cd frontend/fro
2. dx serve --platform web
3. Edit Dioxus components in src/ and Tailwind classes

### Reindex and performance testing
- Start backend, POST /reindex (async supported); compare segment file counts in index directory to validate reduction (see PLAN.md).

---

## 📈 Performance & Scale

- Rate limiting: per-IP token-bucket with LRU; tunable via env, exposes labeled metrics.
- Histogram buckets configurable via env; defaults used when unset or invalid.
- INDEX_IN_RAM optional for small datasets to improve latency (see PLAN.md).

---

## 🚨 Things to Be Careful About

### Security Considerations
- TRUST_PROXY should only be true behind a trusted reverse proxy; otherwise use real remote addr.
- Manage Redis and installer-generated .env securely; do not commit secrets.
- Large uploads and reindex endpoints can be expensive; ensure proper rate limits and auth if added.


Updated at: 2025-11-05

