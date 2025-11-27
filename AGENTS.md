# Repository Guidelines

## Project Structure & Module Organization

- **Backend (Rust)**: `./src` – Actix Web API with core modules:
  - `src/api/` – HTTP routes (upload, search, reindex, agent, memory, tools)
  - `src/retriever.rs`, `src/index.rs`, `src/embedder.rs` – Search and indexing engine
  - `src/monitoring/` – Metrics, tracing, OpenTelemetry, rate limiting
  - `src/cache/` – L2 memory cache and L3 Redis cache
  - `src/memory/`, `src/tools/` – Agent memory and tool execution
- **Frontend (Dioxus)**: `./frontend/fro` – Rust-based web UI with Tailwind CSS
- **Tests**: `./tests` – Integration tests for retriever, monitoring, rate limits, caching
- **Runtime data**: `./documents` (uploads), `./tantivy_index`, `./data`, `./db`, `./cache`, `./logs`

## Build, Test, and Development Commands

```bash
# Backend: run development server
cargo run

# Backend: build optimized release
cargo build --release

# Backend: run all tests
cargo test

# Frontend: build Tailwind CSS once
cd frontend/fro && npm run css:build

# Frontend: watch CSS during development
cd frontend/fro && npm run css:watch

# Frontend: serve Dioxus app (requires dx CLI)
cd frontend/fro && dx serve --platform web
```

## Coding Style & Naming Conventions

- **Indentation**: 4 spaces (Rust and JavaScript)
- **Rust naming**: 
  - Modules/files: `snake_case` (e.g., `path_manager.rs`)
  - Types/structs: `UpperCamelCase` (e.g., `ApiConfig`)
  - Functions/variables: `snake_case` (e.g., `index_all_documents`)
  - Constants: `SCREAMING_SNAKE_CASE` (e.g., `UPLOAD_DIR`)
- **Linting**: Use `rustfmt` and `clippy` for Rust code
- **Frontend**: Follow Dioxus component patterns; Tailwind utility classes for styling

## Testing Guidelines

- **Framework**: Rust built-in test framework (`cargo test`)
- **Test locations**: 
  - Integration tests: `./tests/*.rs`
  - Unit tests: Inline with `#[cfg(test)]` modules in source files
- **Running tests**: `cargo test` (runs all tests)
- **Test naming**: Use descriptive names like `test_cache_hit_miss`, `test_rate_limit_enforcement`
- **Coverage**: No explicit coverage requirements; focus on critical paths (retriever, API, monitoring)

## Commit & Pull Request Guidelines

- **Commit format**: Descriptive messages with context (e.g., "phase 17 completed", "Phase 15 Step 3: Async startup, concurrency guard")
- **Commit style**: Use imperative mood; reference phase/feature when applicable
- **PR requirements**: 
  - Ensure `cargo test` passes
  - Update documentation if adding new endpoints or features
  - Include rationale for architectural changes
- **Branch naming**: Use `feature/<name>` for new features (e.g., `feature/monitoring-improvements`)

---

# Repository Tour

## 🎯 What This Repository Does

**ag** is a Rust-based Agentic RAG (Retrieval-Augmented Generation) service that provides document indexing, semantic search, and agent memory capabilities through an Actix Web API, with a Dioxus-based web frontend.

**Key responsibilities:**
- Index and search text/PDF documents using Tantivy full-text search and vector embeddings
- Provide HTTP API for document upload, search, reindex (sync/async), and agent operations
- Expose comprehensive monitoring via Prometheus metrics, OpenTelemetry tracing, and health endpoints
- Support multi-layer caching (L2 in-memory, L3 Redis) for performance optimization
- Enable agent memory and tool execution for agentic workflows

---

## 🏗️ Architecture Overview

### System Context
```
[Browser/Client] → [Actix Web API (ag)] → [Tantivy Index + SQLite]
                          ↓                        ↓
                   [Dioxus Frontend]        [Redis Cache (optional)]
                          ↓
                   [Prometheus/OTLP Collector]
```

### Key Components

- **HTTP API (`src/api/`)** - Actix Web server exposing RESTful endpoints:
  - Document management: upload, list, delete
  - Search operations: full-text search, rerank, summarize
  - Indexing: synchronous and asynchronous reindex with job tracking
  - Agent operations: memory storage/retrieval, tool execution
  - Monitoring: health, readiness, Prometheus metrics

- **Retriever (`src/retriever.rs`)** - Core search engine:
  - Tantivy-based full-text indexing
  - Vector storage for semantic search (JSON-based)
  - Multi-layer caching (L1 query cache, L2 LRU memory, L3 Redis)
  - Metrics tracking (searches, cache hits/misses, latency)
  - Atomic reindex with batch operations

- **Indexing Pipeline (`src/index.rs`)** - Document processing:
  - File scanning and text extraction (txt, pdf)
  - Text chunking (line-based)
  - Embedding generation via `embedder.rs`
  - Batch indexing for performance

- **Monitoring (`src/monitoring/`)** - Observability stack:
  - Prometheus metrics export (`/monitoring/metrics`)
  - OpenTelemetry distributed tracing (OTLP exporter)
  - Health and readiness checks
  - Rate limiting middleware with per-route rules
  - Configurable histogram buckets for latency tracking

- **Caching (`src/cache/`)** - Performance optimization:
  - L2: In-memory LRU cache for search results
  - L3: Optional Redis cache with TTL
  - Cache invalidation on reindex

- **Agent System (`src/memory/`, `src/tools/`)** - Agentic capabilities:
  - Agent memory layer with SQLite persistence
  - Tool registry and execution framework
  - Decision engine for query planning
  - Multi-agent coordination support

- **Frontend (`frontend/fro/`)** - Dioxus web application:
  - Rust-based UI components
  - Tailwind CSS for styling
  - API client for backend communication

### Data Flow

1. **Document Upload**: Client uploads files via `POST /upload` → files stored in `./documents`
2. **Indexing**: Background task or explicit `POST /reindex` triggers indexing:
   - `index.rs` scans `./documents`, extracts text, chunks content
   - `embedder.rs` generates embeddings for each chunk
   - `retriever.rs` writes to Tantivy index and `vectors.json`
3. **Search**: Client queries via `GET /search?q=<query>`:
   - Check L2/L3 caches for cached results
   - Query Tantivy index for full-text matches
   - Optionally rerank by vector similarity
   - Update metrics and cache results
4. **Monitoring**: Prometheus scrapes `/monitoring/metrics` for metrics; OpenTelemetry traces sent to OTLP collector

---

## 📁 Project Structure [Partial Directory Tree]

```
ag/
├── src/                          # Main Rust backend source
│   ├── api/                      # HTTP route handlers
│   │   ├── mod.rs                # Main API server setup and core routes
│   │   ├── agent_routes.rs       # Agent goal/episode/reflection endpoints
│   │   ├── memory_routes.rs      # Memory chunk storage/search
│   │   ├── tool_routes.rs        # Tool selection and execution
│   │   ├── composer_routes.rs    # Tool chain composition
│   │   └── decision_engine_routes.rs  # Query planning and decision logic
│   ├── monitoring/               # Observability and metrics
│   │   ├── metrics.rs            # Prometheus metrics definitions
│   │   ├── tracing_config.rs     # Tracing/logging setup
│   │   ├── otel_config.rs        # OpenTelemetry configuration
│   │   ├── rate_limit_middleware.rs  # Rate limiting middleware
│   │   ├── health.rs             # Health check logic
│   │   └── alerting_hooks.rs     # Webhook alerts for reindex events
│   ├── cache/                    # Caching layers
│   │   ├── cache_layer.rs        # L2 in-memory LRU cache
│   │   ├── redis_cache.rs        # L3 Redis cache
│   │   └── invalidation.rs       # Cache invalidation logic
│   ├── memory/                   # Agent memory system
│   │   ├── agent.rs              # Agent with goals, tasks, episodes
│   │   ├── vector_store.rs       # Vector storage and similarity search
│   │   ├── chunker.rs            # Semantic chunking
│   │   ├── decision_engine.rs    # Query planning and execution
│   │   └── llm_provider.rs       # LLM integration (Ollama)
│   ├── tools/                    # Tool execution framework
│   │   ├── mod.rs                # Tool registry and base traits
│   │   ├── tool_executor.rs      # Tool execution engine
│   │   ├── tool_selector.rs      # Tool selection logic
│   │   ├── tool_composer.rs      # Tool chain composition
│   │   ├── calculator.rs         # Calculator tool
│   │   ├── web_search.rs         # Web search tool
│   │   └── url_fetch.rs          # URL fetching tool
│   ├── db/                       # Database initialization
│   │   └── schema_init.rs        # SQLite schema setup
│   ├── security/                 # Security utilities
│   │   └── rate_limiter.rs       # Token bucket rate limiter
│   ├── installer/                # Installation utilities
│   ├── main.rs                   # Application entry point
│   ├── lib.rs                    # Library module declarations
│   ├── config.rs                 # Configuration from environment
│   ├── retriever.rs              # Core search and indexing engine
│   ├── index.rs                  # Document indexing pipeline
│   ├── embedder.rs               # Text embedding generation
│   ├── path_manager.rs           # Path management for data/index files
│   ├── agent.rs                  # Simple agent implementation
│   └── agent_memory.rs           # Agent memory persistence
├── tests/                        # Integration tests
│   ├── retriever_tests.rs        # Retriever functionality tests
│   ├── rate_limit_middleware_integration_test.rs  # Rate limit tests
│   ├── test_cache_layer.rs       # Cache layer tests
│   ├── trace_propagation.rs      # Distributed tracing tests
│   └── w3c_trace_context.rs      # W3C trace context tests
├── frontend/fro/                 # Dioxus frontend
│   ├── src/                      # Frontend Rust source
│   ├── assets/                   # Static assets
│   ├── public/                   # Public files (CSS output)
│   ├── package.json              # npm scripts for Tailwind
│   ├── tailwind.config.js        # Tailwind configuration
│   ├── Dioxus.toml               # Dioxus build config
│   └── Cargo.toml                # Frontend dependencies
├── documents/                    # Uploaded documents (runtime)
├── tantivy_index/                # Tantivy index files (runtime)
├── data/                         # Vector storage and data files (runtime)
├── db/                           # SQLite databases (runtime)
├── cache/                        # Cache files (runtime)
├── logs/                         # Application logs (runtime)
├── Cargo.toml                    # Backend dependencies and features
├── build.rs                      # Build-time metadata (git SHA, build time)
└── .env.example                  # Environment variable examples
```

---

### Key Files to Know

| File | Purpose | When You'd Touch It |
|------|---------|---------------------|
| `src/main.rs` | Application entry point; initializes tracing, OTLP, DB, retriever, starts Actix server | Change startup behavior, add initialization steps |
| `src/api/mod.rs` | Main API server setup; defines all HTTP routes and middleware | Add new endpoints, modify CORS, adjust rate limits |
| `src/retriever.rs` | Core search engine; Tantivy index, vector storage, caching, metrics | Modify search logic, add vector operations, tune caching |
| `src/index.rs` | Document indexing pipeline; file scanning, text extraction, chunking | Add support for new file types, change chunking strategy |
| `src/config.rs` | Configuration from environment variables | Add new config options, change defaults |
| `src/monitoring/metrics.rs` | Prometheus metrics definitions and export | Add new metrics, modify histogram buckets |
| `src/monitoring/rate_limit_middleware.rs` | Rate limiting middleware with per-route rules | Adjust rate limits, add route-specific rules |
| `src/monitoring/otel_config.rs` | OpenTelemetry setup and OTLP exporter | Configure tracing backends, adjust sampling |
| `src/cache/redis_cache.rs` | Redis L3 cache implementation | Modify Redis connection, adjust TTL |
| `src/memory/agent.rs` | Agent with goals, tasks, episodes, reflections | Extend agent capabilities, add new memory types |
| `src/tools/mod.rs` | Tool registry and execution framework | Register new tools, modify tool interface |
| `frontend/fro/package.json` | npm scripts for Tailwind CSS | Update CSS build/watch commands |
| `frontend/fro/Dioxus.toml` | Dioxus build configuration | Adjust dev server settings, file watching |
| `Cargo.toml` | Backend dependencies and feature flags | Add dependencies, enable features (e.g., `installer`) |
| `.env.example` | Environment variable documentation | Document new env vars |

---

## 🔧 Technology Stack

### Core Technologies

- **Language**: Rust (edition 2021) - Chosen for performance, safety, and concurrency
- **Backend Framework**: Actix Web 4.x - High-performance async web framework
- **Search Engine**: Tantivy 0.24.x - Full-text search library (Lucene-like)
- **Database**: rusqlite 0.37 (SQLite) - Lightweight embedded database for metadata
- **Async Runtime**: Tokio 1.x - Async runtime for concurrent operations
- **Frontend**: Dioxus 0.6 - Rust-based reactive UI framework
- **Styling**: Tailwind CSS 4.x - Utility-first CSS framework

### Key Libraries

- **llm** (1.3.4) - LLM integration for embeddings and generation
- **prometheus** (0.13) - Metrics collection and export
- **opentelemetry** (0.21) + **opentelemetry-otlp** (0.14) - Distributed tracing
- **tracing** (0.1) + **tracing-subscriber** (0.3) - Structured logging and tracing
- **redis** (0.32) - Redis client for L3 caching
- **rayon** (1.10) - Data parallelism for batch operations
- **lru** (0.12) - LRU cache implementation for L2 caching
- **serde** (1.0) + **serde_json** (1.0) - Serialization/deserialization
- **actix-cors** (0.7) - CORS middleware for API
- **pdf-extract** (0.7) - PDF text extraction (placeholder)

### Development Tools

- **cargo** - Rust build system and package manager
- **rustfmt** - Code formatting
- **clippy** - Linting and code analysis
- **dx** (Dioxus CLI) - Frontend development server
- **npm** - Tailwind CSS build scripts

---

## 🌐 External Dependencies

### Required Services

- **Tantivy Index** (local filesystem) - Full-text search index stored in `./tantivy_index`
- **SQLite** (embedded) - Metadata storage for documents, agent memory, goals, tasks

### Optional Integrations

- **Redis** - L3 cache for search results (set `REDIS_ENABLED=true`)
- **Prometheus** - Metrics scraping from `/monitoring/metrics`
- **OpenTelemetry Collector** - Distributed tracing via OTLP (gRPC)
- **Webhook Endpoints** - Reindex completion alerts (set `REINDEX_WEBHOOK_URL`)

---

### Environment Variables

```bash
# Server Configuration
BACKEND_HOST=127.0.0.1          # Server bind address
BACKEND_PORT=3010               # Server port

# Indexing & Startup
SKIP_INITIAL_INDEXING=false     # Skip background indexing on startup
INDEX_IN_RAM=false              # Use in-memory index (high memory usage)

# Rate Limiting
RATE_LIMIT_ENABLED=false        # Enable rate limiting middleware
RATE_LIMIT_QPS=1.0              # Global queries per second limit
RATE_LIMIT_BURST=5              # Global burst capacity
RATE_LIMIT_SEARCH_QPS=          # Search-specific QPS (optional)
RATE_LIMIT_SEARCH_BURST=        # Search-specific burst (optional)
RATE_LIMIT_UPLOAD_QPS=          # Upload-specific QPS (optional)
RATE_LIMIT_UPLOAD_BURST=        # Upload-specific burst (optional)
RATE_LIMIT_LRU_CAPACITY=1024    # LRU cache size for rate limiter
TRUST_PROXY=false               # Trust X-Forwarded-For headers

# Redis L3 Cache
REDIS_ENABLED=false             # Enable Redis L3 cache
REDIS_URL=redis://127.0.0.1:6379/  # Redis connection URL
REDIS_TTL=3600                  # Cache TTL in seconds

# Monitoring & Tracing
RUST_LOG=info,tantivy=warn      # Log level configuration
SEARCH_HISTO_BUCKETS=1,2,5,10,20,50,100,250,500,1000  # Search latency histogram buckets (ms)
REINDEX_HISTO_BUCKETS=50,100,250,500,1000,2000,5000,10000  # Reindex latency buckets (ms)

# Alerting
REINDEX_WEBHOOK_URL=            # Webhook URL for reindex completion alerts

# OpenTelemetry (set via otel_config.rs)
OTEL_EXPORTER_OTLP_ENDPOINT=    # OTLP collector endpoint
OTEL_SERVICE_NAME=ag            # Service name for traces
```

---

## 🔄 Common Workflows

### Local Development Workflow

1. **Set up environment**:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

2. **Start backend**:
   ```bash
   cargo run
   # Server starts on http://127.0.0.1:3010
   ```

3. **Verify health**:
   ```bash
   curl http://127.0.0.1:3010/monitoring/health
   curl http://127.0.0.1:3010/monitoring/metrics
   ```

4. **Start frontend** (in separate terminal):
   ```bash
   cd frontend/fro
   npm run css:watch &  # Watch Tailwind CSS
   dx serve --platform web
   ```

**Code path**: `main.rs` → `start_api_server()` → Actix Web routes

---

### Document Indexing Workflow

1. **Upload documents**:
   ```bash
   curl -F "file=@document.txt" http://127.0.0.1:3010/upload
   ```

2. **Trigger reindex** (synchronous):
   ```bash
   curl -X POST http://127.0.0.1:3010/reindex
   ```

3. **Or trigger async reindex**:
   ```bash
   curl -X POST http://127.0.0.1:3010/reindex/async
   # Returns job_id
   curl http://127.0.0.1:3010/reindex/status/{job_id}
   ```

4. **Verify indexing**:
   ```bash
   curl http://127.0.0.1:3010/index/info
   curl http://127.0.0.1:3010/documents
   ```

**Code path**: `POST /upload` → `upload_document_inner()` → files saved to `./documents` → `POST /reindex` → `index_all_documents()` → `retriever.index_chunk()` → Tantivy + `vectors.json`

---

### Search Workflow

1. **Perform search**:
   ```bash
   curl "http://127.0.0.1:3010/search?q=rust+programming"
   ```

2. **Check cache metrics**:
   ```bash
   curl http://127.0.0.1:3010/monitoring/metrics | grep cache
   ```

**Code path**: `GET /search` → `search_documents_inner()` → `retriever.search()` → Check L2/L3 cache → Query Tantivy → Update metrics → Return results

---

### Agent Memory Workflow

1. **Store agent memory**:
   ```bash
   curl -X POST http://127.0.0.1:3010/memory/store_rag \
     -H "Content-Type: application/json" \
     -d '{"agent_id": "agent1", "memory_type": "observation", "content": "User prefers concise answers"}'
   ```

2. **Search agent memory**:
   ```bash
   curl -X POST http://127.0.0.1:3010/memory/search_rag \
     -H "Content-Type: application/json" \
     -d '{"agent_id": "agent1", "query": "user preferences", "top_k": 5}'
   ```

3. **Recall recent memories**:
   ```bash
   curl -X POST http://127.0.0.1:3010/memory/recall_rag \
     -H "Content-Type: application/json" \
     -d '{"agent_id": "agent1", "limit": 10}'
   ```

**Code path**: `POST /memory/store_rag` → `AgentMemory::store_rag()` → SQLite insert → `POST /memory/search_rag` → `AgentMemory::search_rag()` → Vector similarity search

---

## 📈 Performance & Scale

### Performance Considerations

- **Caching Strategy**:
  - L1: Query-level cache in `Retriever` (LRU)
  - L2: In-memory LRU cache for search results
  - L3: Optional Redis cache with configurable TTL
  - Cache invalidation on reindex to ensure freshness

- **Batch Indexing**: Uses `begin_batch()` and `end_batch()` to reduce Tantivy commit overhead during bulk indexing

- **Parallel Processing**: Uses `rayon` for parallel document processing and embedding generation

- **In-Memory Index**: Optional `INDEX_IN_RAM=true` for faster search (high memory usage; recommended for <100 documents)

- **Rate Limiting**: Token bucket algorithm with per-route rules to prevent abuse

### Monitoring

- **Metrics**: Prometheus-compatible metrics at `/monitoring/metrics`:
  - Search latency histograms (configurable buckets)
  - Cache hit/miss rates
  - Reindex duration
  - Document and vector counts
  - Rate limit rejections

- **Tracing**: OpenTelemetry distributed tracing with OTLP export:
  - Request tracing with W3C trace context propagation
  - Span instrumentation for key operations
  - Configurable sampling

- **Health Checks**:
  - `/monitoring/health` - Component health status
  - `/monitoring/ready` - Readiness check for load balancers

- **Alerts**: Webhook notifications for reindex completion (success/failure)

---

## 🚨 Things to Be Careful About

### 🔒 Security Considerations

- **Authentication**: No built-in authentication; deploy behind reverse proxy with auth if exposing publicly
- **File Upload**: Only `.txt` and `.pdf` files accepted; enforce size limits and validate content
- **Rate Limiting**: Enable `RATE_LIMIT_ENABLED=true` in production; configure per-route limits
- **Proxy Trust**: Only set `TRUST_PROXY=true` when behind a trusted reverse proxy (e.g., nginx); otherwise, IP-based rate limiting can be bypassed
- **Redis Security**: Use authenticated Redis connection in production; avoid exposing Redis port
- **Secrets Management**: Never commit `.env` file; use environment variables or secret management systems

### ⚠️ Operational Warnings

- **In-Memory Index**: `INDEX_IN_RAM=true` loads entire index into RAM; only use for small datasets (<100 docs)
- **Concurrent Reindex**: Only one reindex operation allowed at a time (enforced by `REINDEX_IN_PROGRESS` atomic flag)
- **Background Indexing**: Initial indexing runs in background by default; set `SKIP_INITIAL_INDEXING=true` to disable
- **Vector Storage**: `vectors.json` is loaded entirely into memory; large datasets may require alternative storage
- **Cache Invalidation**: All caches (L2/L3) are invalidated on reindex; expect cache misses after reindex
- **Database Locking**: SQLite uses file-based locking; avoid concurrent writes from multiple processes

### 🐛 Known Limitations

- **PDF Parsing**: PDF text extraction is placeholder ("PDF parsing not implemented"); integrate `pdf-extract` or similar
- **Embedding Model**: Uses simple embedding via `llm` crate; consider dedicated embedding models for production
- **Vector Search**: Basic cosine similarity; no HNSW or other approximate nearest neighbor algorithms
- **Scalability**: Single-node architecture; horizontal scaling requires external index (e.g., Elasticsearch) and distributed cache

---

*Last updated: 2025-01-16 (based on commit f3baed9)*
