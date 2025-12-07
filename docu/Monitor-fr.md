# Monitor Frontend Documentation

**Version:** 1.0.0  
**Date:** 2025-12-03  
**Status:** Design Complete - Ready for Implementation

---

## Table of Contents

1. [Overview](#overview)
2. [Page Structure](#page-structure)
3. [Design Principles](#design-principles)
4. [Page Layouts](#page-layouts)
5. [Components](#components)
6. [Data Sources](#data-sources)
7. [File Structure](#file-structure)
8. [Installer Impact](#installer-impact)
9. [Implementation Order](#implementation-order)

---

## Overview

The Monitor Frontend is a multi-page Grafana-style monitoring dashboard for the AG (Agentic RAG) system. Built with Dioxus + Tailwind CSS, it provides real-time visibility into system health, performance metrics, caching, indexing, and rate limiting.

### Goals

- Provide real-time system observability
- Match Grafana's dark theme aesthetic
- Lightweight and performant (resource-constrained hardware)
- Lazy-load only active page content
- Reusable component architecture

---

## Page Structure

| Route | Page Name | Purpose | Backend Endpoints |
|-------|-----------|---------|-------------------|
| `/monitor` | Overview | Health + key stats at a glance | `/health`, `/ready`, `/metrics` |
| `/monitor/requests` | Requests | Latency, throughput, errors | `/monitoring/metrics` |
| `/monitor/cache` | Cache | L1/L2/L3 hit rates, Redis status | `/monitoring/metrics` |
| `/monitor/index` | Index | Documents, vectors, segments, reindex | `/index-info`, `/reindex/*` |
| `/monitor/rate-limits` | Rate Limits | Drops, buckets, per-route limits | `/monitoring/metrics` |
| `/monitor/logs` | Logs | Recent log entries | Loki/Vector or `/logs` endpoint |

---

## Design Principles

### Grafana Dark Theme Colors

| Element | Tailwind Class | Hex |
|---------|---------------|-----|
| Background | `bg-gray-900` | `#111827` |
| Panel Background | `bg-gray-800` | `#1f2937` |
| Panel Border | `border-gray-700` | `#374151` |
| Text Primary | `text-gray-100` | `#f3f4f6` |
| Text Secondary | `text-gray-400` | `#9ca3af` |
| Accent Green | `text-green-400` | `#4ade80` |
| Accent Yellow | `text-yellow-400` | `#facc15` |
| Accent Red | `text-red-400` | `#f87171` |
| Accent Blue | `text-blue-400` | `#60a5fa` |

### Layout Principles

1. **Dark background** with panel cards
2. **Row-based layout** with collapsible sections
3. **Stat panels** (big numbers with sparklines)
4. **Time series panels** (charts with legends)
5. **Table panels** for detailed data
6. **Top toolbar** with time range picker and refresh controls

---

## Page Layouts

### 1. `/monitor` - Overview (Dashboard Home)

```
┌─────────────────────────────────────────────────────────────────┐
│ AG Monitor                                    [⟳ 5s] [Refresh] │
├─────────────────────────────────────────────────────────────────┤
│ [Overview] [Requests] [Cache] [Index] [Rate Limits] [Logs]     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ▼ System Health                                                │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐       │
│  │ API    │ │Tantivy │ │SQLite  │ │ Redis  │ │ Uptime │       │
│  │ ● OK   │ │ ● OK   │ │ ● OK   │ │ ● OK   │ │ 2d 4h  │       │
│  │ 12ms   │ │ Ready  │ │ Ready  │ │ Conn   │ │        │       │
│  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘       │
│                                                                 │
│  ▼ Key Metrics                                                  │
│  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐         │
│  │ Requests/sec  │ │ p95 Latency   │ │ Error Rate    │         │
│  │    42.3 ▲     │ │    45ms       │ │   0.12%       │         │
│  └───────────────┘ └───────────────┘ └───────────────┘         │
│                                                                 │
│  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐         │
│  │ Cache Hit %   │ │ Documents     │ │ Rate Drops    │         │
│  │   85.3%       │ │   1,234       │ │     23        │         │
│  └───────────────┘ └───────────────┘ └───────────────┘         │
│                                                                 │
│  ▼ Quick Actions                                                │
│  [Trigger Reindex] [Clear Cache] [View Grafana ↗]              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Data sources:**
- `GET /monitoring/health`
- `GET /monitoring/ready`
- `GET /monitoring/metrics` (parsed)

---

### 2. `/monitor/requests` - Request Metrics

```
┌─────────────────────────────────────────────────────────────────┐
│ Requests                                      [⟳ 5s] [Refresh] │
├─────────────────────────────────────────────────────────────────┤
│ [Overview] [Requests] [Cache] [Index] [Rate Limits] [Logs]     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ▼ Throughput                                                   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Requests per Second (5m rate)                           │   │
│  │                                                         │   │
│  │  50 ┤                    ╭─╮                            │   │
│  │  40 ┤              ╭────╯  ╰──╮                         │   │
│  │  30 ┤         ╭───╯          ╰───╮                      │   │
│  │  20 ┤    ╭───╯                   ╰───                   │   │
│  │  10 ┤───╯                                               │   │
│  │   0 └──────────────────────────────────────────────     │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ▼ Latency Percentiles                                          │
│  ┌──────────────────────────────┐ ┌────────────────────────┐   │
│  │ Response Time Distribution   │ │ By Route               │   │
│  │                              │ │                        │   │
│  │ p99  ████████████████ 120ms │ │ /search     35ms ████  │   │
│  │ p95  ████████████░░░  45ms  │ │ /upload     89ms █████ │   │
│  │ p50  ████████░░░░░░░  12ms  │ │ /agent      67ms ████  │   │
│  │                              │ │ /rerank     28ms ███   │   │
│  └──────────────────────────────┘ └────────────────────────┘   │
│                                                                 │
│  ▼ Status Breakdown                                             │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 2xx: 98.5% ██████████████████████████████████████████░░ │   │
│  │ 4xx:  1.2% █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │   │
│  │ 5xx:  0.3% ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Metrics used:**
- `request_latency_ms_bucket{method, route, status_class, le}`
- `request_latency_ms_count`
- `request_latency_ms_sum`

---

### 3. `/monitor/cache` - Cache Performance

```
┌─────────────────────────────────────────────────────────────────┐
│ Cache                                         [⟳ 10s] [Refresh]│
├─────────────────────────────────────────────────────────────────┤
│ [Overview] [Requests] [Cache] [Index] [Rate Limits] [Logs]     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ▼ Cache Layers                                                 │
│  ┌───────────────────┐ ┌───────────────────┐ ┌────────────────┐│
│  │ L1 Query Cache    │ │ L2 LRU Cache      │ │ L3 Redis       ││
│  │                   │ │                   │ │                ││
│  │     92.3%         │ │     78.5%         │ │    85.1%       ││
│  │ ████████████░░░░░ │ │ ██████████░░░░░░░ │ │ ███████████░░░ ││
│  │                   │ │                   │ │                ││
│  │ Hits:   12,345    │ │ Hits:    8,901    │ │ Hits:   5,678  ││
│  │ Misses:  1,023    │ │ Misses:  2,456    │ │ Misses:   987  ││
│  │ Size:   128/256   │ │ Size:   512/1024  │ │ TTL:     300s  ││
│  └───────────────────┘ └───────────────────┘ └────────────────┘│
│                                                                 │
│  ▼ Cache Flow                                                   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Request → [L1] → miss → [L2] → miss → [L3] → miss → DB │   │
│  │             ↓ hit         ↓ hit         ↓ hit           │   │
│  │           Return        Return        Return            │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ▼ Redis Connection                                             │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Status: ● Connected   Host: 127.0.0.1:6379              │   │
│  │ Memory: 24.5 MB       Keys: 1,234                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  [Clear L1] [Clear L2] [Clear All]                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Data sources:**
- Cache metrics from `/monitoring/metrics`
- Redis status from health check

---

### 4. `/monitor/index` - Index & Storage

```
┌─────────────────────────────────────────────────────────────────┐
│ Index & Storage                               [⟳ 30s] [Refresh]│
├─────────────────────────────────────────────────────────────────┤
│ [Overview] [Requests] [Cache] [Index] [Rate Limits] [Logs]     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ▼ Index Statistics                                             │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │
│  │ Documents  │ │ Vectors    │ │ Index Size │ │ Segments   │   │
│  │   1,234    │ │  45,678    │ │  256 MB    │ │    12      │   │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘   │
│                                                                 │
│  ▼ Reindex Status                                               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Current Status: ● Idle                                   │   │
│  │ Last Reindex:   2025-12-03 10:30:15 (2 hours ago)       │   │
│  │ Duration:       45.2s                                    │   │
│  │ Documents:      1,234 processed                          │   │
│  │ Vectors:        45,678 generated                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ▼ Async Jobs                                                   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Job ID          │ Status    │ Started     │ Progress    │   │
│  │ abc-123-def     │ ● Running │ 10:45:00    │ 45%         │   │
│  │ xyz-789-ghi     │ ✓ Done    │ 09:30:00    │ 100%        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  [Trigger Sync Reindex] [Trigger Async Reindex]                │
│                                                                 │
│  ▼ Storage Paths                                                │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Tantivy Index: ~/.local/share/ag/index/tantivy/         │   │
│  │ Vectors:       ~/.local/share/ag/vectors.json           │   │
│  │ SQLite:        ~/.local/share/ag/metadata.db            │   │
│  │ Documents:     ~/ag/documents/                          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Endpoints:**
- `GET /index-info`
- `POST /reindex`
- `POST /reindex/async`
- `GET /reindex/status/:job_id`

---

### 5. `/monitor/rate-limits` - Rate Limiting

```
┌─────────────────────────────────────────────────────────────────┐
│ Rate Limits                                   [⟳ 5s] [Refresh] │
├─────────────────────────────────────────────────────────────────┤
│ [Overview] [Requests] [Cache] [Index] [Rate Limits] [Logs]     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ▼ Summary                                                      │
│  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐         │
│  │ Total Drops   │ │ Active IPs    │ │ Bucket Cap    │         │
│  │    23 (5m)    │ │    47/1024    │ │    1024       │         │
│  └───────────────┘ └───────────────┘ └───────────────┘         │
│                                                                 │
│  ▼ Drops by Route (5m)                                          │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Route              │ Drops │ QPS Limit │ Burst │ Status │   │
│  ├────────────────────┼───────┼───────────┼───────┼────────┤   │
│  │ /search            │  12   │    10     │  40   │ ● OK   │   │
│  │ /upload            │   8   │     2     │   5   │ ● OK   │   │
│  │ /agent             │   3   │    10     │  40   │ ● OK   │   │
│  │ /reindex           │   0   │   0.5     │   2   │ ● OK   │   │
│  │ /memory/store_rag  │   0   │     1     │   5   │ ● OK   │   │
│  │ /memory/search_rag │   0   │     5     │  20   │ ● OK   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ▼ Configuration                                                │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ RATE_LIMIT_ENABLED:    true                             │   │
│  │ TRUST_PROXY:           false                            │   │
│  │ LRU_CAPACITY:          1024                             │   │
│  │ Default Search QPS:    10                               │   │
│  │ Default Upload QPS:    2                                │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ▼ Exempt Prefixes                                              │
│  │ /, /health, /ready, /metrics                            │   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Metrics:**
- `rate_limit_drops_total`
- `rate_limit_drops_by_route_total{route}`

---

### 6. `/monitor/logs` - Log Viewer

```
┌─────────────────────────────────────────────────────────────────┐
│ Logs                                          [⟳ 2s] [Refresh] │
├─────────────────────────────────────────────────────────────────┤
│ [Overview] [Requests] [Cache] [Index] [Rate Limits] [Logs]     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Filter: [All ▼] [INFO ▼]  Search: [________________] [🔍]    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 12:45:23.456 INFO  ag::api     GET /search q="rust"     │   │
│  │ 12:45:23.467 INFO  ag::api     Response 200 in 11ms     │   │
│  │ 12:45:24.123 WARN  ag::rate    Rate limit drop /upload  │   │
│  │ 12:45:25.789 INFO  ag::cache   L1 cache hit for "rust"  │   │
│  │ 12:45:26.012 DEBUG ag::retriever Hybrid search started  │   │
│  │ 12:45:26.045 DEBUG ag::retriever Found 12 results       │   │
│  │ 12:45:27.234 INFO  ag::api     POST /reindex started    │   │
│  │ 12:45:27.890 INFO  ag::index   Processing 1234 docs     │   │
│  │ ...                                                      │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  [Open in Grafana ↗]  [Download Logs]                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Source:** Loki via Vector, or polling a `/logs/recent` endpoint.

---

## Components

### Reusable Components

| Component | File | Purpose |
|-----------|------|---------|
| `StatCard` | `components/monitor/stat_card.rs` | Big number with optional sparkline |
| `HealthCard` | `components/monitor/health_card.rs` | Health status indicator |
| `NavTabs` | `components/monitor/nav_tabs.rs` | Tab navigation bar |
| `ProgressBar` | `components/monitor/progress_bar.rs` | Horizontal progress indicator |
| `DataTable` | `components/monitor/data_table.rs` | Styled data table |
| `Panel` | `components/monitor/panel.rs` | Grafana-style panel container |
| `RowHeader` | `components/monitor/row_header.rs` | Collapsible section header |
| `RefreshControl` | `components/monitor/refresh_control.rs` | Auto-refresh dropdown |

### Component Props

#### StatCard
```rust
#[derive(Props, Clone, PartialEq)]
pub struct StatCardProps {
    pub title: String,
    pub value: String,
    pub unit: Option<String>,
    pub trend: Option<Trend>,      // Up, Down, Neutral
    pub sparkline: Option<Vec<f64>>,
}
```

#### HealthCard
```rust
#[derive(Props, Clone, PartialEq)]
pub struct HealthCardProps {
    pub name: String,
    pub status: HealthStatus,      // Healthy, Degraded, Unhealthy
    pub detail: Option<String>,
}
```

---

## Data Sources

### Backend Endpoints Used

| Endpoint | Method | Purpose | Refresh |
|----------|--------|---------|---------|
| `/monitoring/health` | GET | Component health status | 5s |
| `/monitoring/ready` | GET | Readiness probe | 5s |
| `/monitoring/metrics` | GET | Prometheus metrics (text) | 5s |
| `/index-info` | GET | Index statistics | 30s |
| `/reindex` | POST | Trigger sync reindex | Manual |
| `/reindex/async` | POST | Trigger async reindex | Manual |
| `/reindex/status/:id` | GET | Async job status | 5s |

### Prometheus Metrics Parsed

| Metric | Type | Labels |
|--------|------|--------|
| `request_latency_ms_bucket` | Histogram | method, route, status_class, le |
| `request_latency_ms_count` | Counter | method, route, status_class |
| `request_latency_ms_sum` | Counter | method, route, status_class |
| `rate_limit_drops_total` | Counter | - |
| `rate_limit_drops_by_route_total` | Counter | route |

### Optional New Endpoints

| Endpoint | Method | Purpose | Priority |
|----------|--------|---------|----------|
| `/monitoring/summary` | GET | Pre-aggregated JSON metrics | Medium |
| `/logs/recent` | GET | Recent log lines as JSON | Low |
| `/cache/stats` | GET | Detailed cache statistics | Low |

---

## File Structure

```
frontend/fro/src/
├── pages/
│   ├── mod.rs                      # Add monitor module
│   └── monitor/
│       ├── mod.rs                  # Monitor page exports
│       ├── overview.rs             # /monitor - Dashboard home
│       ├── requests.rs             # /monitor/requests
│       ├── cache.rs                # /monitor/cache
│       ├── index_page.rs           # /monitor/index
│       ├── rate_limits.rs          # /monitor/rate-limits
│       └── logs.rs                 # /monitor/logs
├── components/
│   ├── mod.rs                      # Add monitor module
│   └── monitor/
│       ├── mod.rs                  # Component exports
│       ├── stat_card.rs            # Metric display card
│       ├── health_card.rs          # Health status card
│       ├── nav_tabs.rs             # Tab navigation
│       ├── progress_bar.rs         # Progress indicator
│       ├── data_table.rs           # Data table
│       ├── panel.rs                # Panel container
│       ├── row_header.rs           # Section header
│       └── refresh_control.rs      # Refresh dropdown
└── app.rs                          # Add routes to Route enum
```

### Route Enum Updates (app.rs)

```rust
#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/about")]
    About {},
    #[route("/monitor")]
    MonitorOverview {},
    #[route("/monitor/requests")]
    MonitorRequests {},
    #[route("/monitor/cache")]
    MonitorCache {},
    #[route("/monitor/index")]
    MonitorIndex {},
    #[route("/monitor/rate-limits")]
    MonitorRateLimits {},
    #[route("/monitor/logs")]
    MonitorLogs {},
    #[route("/:..segments")]
    PageNotFound { segments: Vec<String> },
}
```

---

## Installer Impact

| Item | Impact | Notes |
|------|--------|-------|
| Frontend routes | Add to `Route` enum in `app.rs` | `/monitor`, `/monitor/*` |
| New pages | 6 new page files | ~200 lines each |
| New components | 8 new component files | ~50-100 lines each |
| API endpoints | Already exist | No backend changes required |
| CSS | Use existing Tailwind | Dark mode classes |
| Dependencies | None | Use existing gloo-net |

### Optional Backend Additions

| Endpoint | Impact | Priority |
|----------|--------|----------|
| `GET /monitoring/summary` | New endpoint - JSON aggregated metrics | Medium |
| `GET /logs/recent` | New endpoint - Recent logs as JSON | Low |
| `GET /cache/stats` | New endpoint - Cache statistics | Low |

---

## Implementation Order

### Phase 1: Foundation (Day 1)
1. Create `pages/monitor/mod.rs`
2. Create `components/monitor/mod.rs`
3. Add routes to `app.rs`
4. Implement `NavTabs` component
5. Implement `Panel` component

### Phase 2: Components (Day 1-2)
1. Implement `StatCard` component
2. Implement `HealthCard` component
3. Implement `ProgressBar` component
4. Implement `DataTable` component
5. Implement `RefreshControl` component

### Phase 3: Overview Page (Day 2)
1. Implement `/monitor` overview page
2. Add health status cards
3. Add key metrics cards
4. Add quick action buttons
5. Test with live backend

### Phase 4: Detail Pages (Day 3-4)
1. Implement `/monitor/requests`
2. Implement `/monitor/cache`
3. Implement `/monitor/index`
4. Implement `/monitor/rate-limits`
5. Implement `/monitor/logs`

### Phase 5: Polish (Day 5)
1. Add auto-refresh functionality
2. Add loading states
3. Add error handling
4. Test all pages
5. Performance optimization

---

## API Response Examples

### GET /monitoring/health
```json
{
  "status": "healthy",
  "timestamp": "2025-12-03T12:30:45Z",
  "uptime_seconds": 1830,
  "components": {
    "api": { "status": "healthy", "latency_ms": 12 },
    "tantivy": { "status": "healthy" },
    "sqlite": { "status": "healthy" },
    "redis": { "status": "healthy", "connected": true }
  }
}
```

### GET /index-info
```json
{
  "documents": 1234,
  "vectors": 45678,
  "index_size_bytes": 268435456,
  "segments": 12,
  "index_in_ram": false,
  "last_reindex": "2025-12-03T10:30:15Z"
}
```

### GET /reindex/status/:job_id
```json
{
  "job_id": "abc-123-def",
  "status": "running",
  "progress": 45,
  "started_at": "2025-12-03T10:45:00Z",
  "documents_processed": 556,
  "vectors_generated": 20555
}
```

---

## Notes

- All pages use polling (not WebSocket) for simplicity
- Prometheus metrics are parsed in frontend (text format → JSON)
- Consider adding `/monitoring/summary` endpoint for efficiency
- Dark mode is default, matches Grafana aesthetic
- Components are designed for reuse across pages

---

**Last Updated:** 2025-12-03  
**Author:** AG Project Team

2. Implementation steps (short)

    Decide data source path:
        Either:
            Frontend → backend → Prometheus, or
            Frontend → Prometheus HTTP API directly (if allowed).
    For each Grafana panel you want:
        Copy the PromQL expression from the JSON.
        Implement a matching backend endpoint or direct Prometheus query.
    Replace the hardcoded numbers in:
        monitor_requests*.rs, monitor_index*.rs, monitor_rate_limits*.rs, monitor_logs*.rs, monitor_health*.rs
        with real values from those queries.
    Optionally add “View in Grafana” links under each section to jump to the full dashboard.
