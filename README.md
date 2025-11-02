# ag (agentic-rag)

## Logging Presets

- Production:
  - `RUST_LOG=info,tantivy=warn`
- Development:
  - `RUST_LOG=debug,tantivy=info`

Set via environment variable before running the backend:

```bash
RUST_LOG=info,tantivy=warn cargo run
```

## Metrics Histogram Buckets (Optional)

Override default Prometheus histogram buckets using environment variables:

- Search latency (ms):
  - `SEARCH_HISTO_BUCKETS=1,2,5,10,20,50,100,250,500,1000`
- Reindex duration (ms):
  - `REINDEX_HISTO_BUCKETS=50,100,250,500,1000,2000,5000,10000`

Notes:
- Values must be positive numbers; list will be sorted ascending.
- If an invalid token is detected, a warning is logged and defaults are used.
- Check current buckets in `/monitoring/metrics` output (look for `*_bucket` lines).

## Profiling (Dev-only, Optional)

Feature-gated pprof stubs:

- Endpoints:
  - `GET /monitoring/pprof/cpu`
  - `GET /monitoring/pprof/heap`
- Default: return 501 Not Implemented
- Enable feature (stubs only for now):

```bash
cargo run --features profiling
```

