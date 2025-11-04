use once_cell::sync::Lazy;
use prometheus::{Encoder, TextEncoder, Registry, IntCounter, IntCounterVec, IntGauge, Histogram, HistogramOpts, Opts};

// Global Prometheus registry
pub static REGISTRY: Lazy<Registry> = Lazy::new(|| Registry::new());

// App info gauge (const)
pub static APP_INFO: Lazy<IntGauge> = Lazy::new(|| {
    // Labels: app and version
    let g = IntGauge::with_opts(
        Opts::new("app_info", "Application info gauge")
            .const_label("app", "ag")
            .const_label("version", env!("CARGO_PKG_VERSION")),
    )
    .unwrap();
    REGISTRY.register(Box::new(g.clone())).ok();
    g
});

// Startup duration
pub static STARTUP_DURATION_MS: Lazy<IntGauge> = Lazy::new(|| {
    let g = IntGauge::new("startup_duration_ms", "Application startup duration in milliseconds").unwrap();
    REGISTRY.register(Box::new(g.clone())).ok();
    g
});

// Reindex metrics
pub static REINDEX_SUCCESS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let c = IntCounter::new("reindex_success_total", "Total successful reindex operations").unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

pub static REINDEX_FAILURE_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let c = IntCounter::new("reindex_failure_total", "Total failed reindex operations").unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

#[doc(hidden)]
pub fn __test_parse_buckets_env(var: &str) -> Option<Vec<f64>> { parse_buckets_env(var) }

fn parse_buckets_env(var: &str) -> Option<Vec<f64>> {
    match std::env::var(var) {
        Ok(val) if !val.trim().is_empty() => {
            let mut parsed: Vec<f64> = Vec::new();
            for tok in val.split(',') {
                let t = tok.trim();
                if t.is_empty() { continue; }
                match t.parse::<f64>() {
                    Ok(v) if v > 0.0 => parsed.push(v),
                    _ => {
                        tracing::warn!(env_var = %var, token = %t, "Invalid histogram bucket value; ignoring");
                        return None;
                    }
                }
            }
            if parsed.is_empty() {
                None
            } else {
                parsed.sort_by(|a, b| a.partial_cmp(b).unwrap());
                Some(parsed)
            }
        }
        _ => None,
    }
}

pub static REINDEX_DURATION_MS: Lazy<Histogram> = Lazy::new(|| {
    let default = vec![50.0, 100.0, 250.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0];
    let buckets = parse_buckets_env("REINDEX_HISTO_BUCKETS").unwrap_or(default);
    let h = Histogram::with_opts(
        HistogramOpts::new("reindex_duration_ms", "Reindex duration in milliseconds")
            .buckets(buckets),
    )
    .unwrap();
    REGISTRY.register(Box::new(h.clone())).ok();
    h
});

// Search metrics
pub static SEARCH_LATENCY_MS: Lazy<Histogram> = Lazy::new(|| {
    let default = vec![1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 250.0, 500.0, 1000.0];
    let buckets = parse_buckets_env("SEARCH_HISTO_BUCKETS").unwrap_or(default);
    let h = Histogram::with_opts(
        HistogramOpts::new("search_latency_ms", "Search latency in milliseconds")
            .buckets(buckets),
    )
    .unwrap();
    REGISTRY.register(Box::new(h.clone())).ok();
    h
});

pub static CACHE_HITS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let c = IntCounter::new("cache_hits_total", "Total cache hits").unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

pub static CACHE_MISSES_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let c = IntCounter::new("cache_misses_total", "Total cache misses").unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

pub static RATE_LIMIT_DROPS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let c = IntCounter::new("rate_limit_drops_total", "Total requests dropped due to rate limit").unwrap();
    REGISTRY.register(Box::new(c.clone())).ok();
    c
});

pub static RATE_LIMIT_DROPS_BY_ROUTE: Lazy<IntCounterVec> = Lazy::new(|| {
    let cv = IntCounterVec::new(
        Opts::new("rate_limit_drops_by_route_total", "Rate limit drops partitioned by route"),
        &["route"],
    ).unwrap();
    REGISTRY.register(Box::new(cv.clone())).ok();
    cv
});

// State gauges
pub static DOCUMENTS_TOTAL: Lazy<IntGauge> = Lazy::new(|| {
    let g = IntGauge::new("documents_total", "Total number of indexed documents").unwrap();
    REGISTRY.register(Box::new(g.clone())).ok();
    g
});

pub static VECTORS_TOTAL: Lazy<IntGauge> = Lazy::new(|| {
    let g = IntGauge::new("vectors_total", "Total number of vectors").unwrap();
    REGISTRY.register(Box::new(g.clone())).ok();
    g
});

pub static INDEX_SIZE_BYTES: Lazy<IntGauge> = Lazy::new(|| {
    let g = IntGauge::new("index_size_bytes", "Index size in bytes (approximate)").unwrap();
    REGISTRY.register(Box::new(g.clone())).ok();
    g
});

pub static REQUEST_LATENCY_MS: Lazy<prometheus::HistogramVec> = Lazy::new(|| {
    use prometheus::{HistogramVec, histogram_opts};
    let opts = histogram_opts!("request_latency_ms", "HTTP request latency in milliseconds");
    let hv = HistogramVec::new(opts, &["method", "route", "status_class"]).unwrap();
    REGISTRY.register(Box::new(hv.clone())).ok();
    hv
});

// Helper to update gauges from retriever
pub fn refresh_retriever_gauges(retriever: &crate::retriever::Retriever) {
    DOCUMENTS_TOTAL.set(retriever.metrics.total_documents_indexed as i64);
    VECTORS_TOTAL.set(retriever.metrics.total_vectors as i64);
    if let Ok(size) = retriever.metrics.get_index_size_bytes() {
        INDEX_SIZE_BYTES.set(size as i64);
    }
}

// Observe search latency in ms
pub fn observe_search_latency_ms(duration_ms: f64) {
    SEARCH_LATENCY_MS.observe(duration_ms);
}

// Record reindex duration in ms
pub fn observe_reindex_duration_ms(duration_ms: f64) {
    REINDEX_DURATION_MS.observe(duration_ms);
}

// Exporter for Prometheus text format
pub fn export_prometheus() -> String {
    let metric_families = REGISTRY.gather();
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    if encoder.encode(&metric_families, &mut buffer).is_ok() {
        String::from_utf8(buffer).unwrap_or_default()
    } else {
        "".to_string()
    }
}
