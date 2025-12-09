use crate::agent::{Agent, AgentResponse};
use crate::agent_memory::{AgentMemory, MemoryItem, MemorySearchResult};
use crate::config::ApiConfig;
use crate::index;
use crate::monitoring::config::MonitoringConfig;
use crate::monitoring::metrics;
use crate::monitoring::rate_limit_middleware::{MatchKind, RateLimitOptions, RouteRule};
use crate::retriever::Retriever;
use crate::security::rate_limiter::{RateLimiter, RateLimiterState};
use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use chrono::Utc;
use futures_util::stream::StreamExt;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;
use tracing::{error, info, warn};
use uuid::Uuid;

pub const UPLOAD_DIR: &str = "documents";

// Phase 15: Global reindex concurrency guard
static REINDEX_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Check if reindex is currently in progress
pub fn is_reindex_in_progress() -> bool {
    REINDEX_IN_PROGRESS.load(Ordering::SeqCst)
}

// Phase 15: Async job tracking
#[derive(Clone, Debug, serde::Serialize)]
struct AsyncJob {
    job_id: String,
    status: String, // "pending", "running", "completed", "failed"
    started_at: String,
    completed_at: Option<String>,
    vectors_indexed: Option<usize>,
    mappings_indexed: Option<usize>,
    error: Option<String>,
}

static ASYNC_JOBS: OnceLock<Arc<Mutex<HashMap<String, AsyncJob>>>> = OnceLock::new();

fn get_jobs_map() -> Arc<Mutex<HashMap<String, AsyncJob>>> {
    ASYNC_JOBS
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

// Global retriever handle
static RETRIEVER: OnceLock<Arc<Mutex<Retriever>>> = OnceLock::new();

pub fn set_retriever_handle(handle: Arc<Mutex<Retriever>>) {
    let _ = RETRIEVER.set(handle);
}

// Rate limiting is enforced by middleware (see monitoring/rate_limit_middleware.rs).
// The per-handler token-bucket implementation was removed to avoid double-limiting.

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(serde::Deserialize)]
pub struct RerankRequest {
    pub query: String,
    pub candidates: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct SummarizeRequest {
    pub query: String,
    pub candidates: Vec<String>,
}

const DEFAULT_LOG_LIMIT: usize = 200;
const MAX_LOG_LIMIT: usize = 500;
const LOG_FILE_PREFIX: &str = "backend.log";

#[derive(Clone)]
struct RateLimitSharedState {
    limiter: Arc<RateLimiter>,
    opts: RateLimitOptions,
}

impl RateLimitSharedState {
    fn config_snapshot(&self, enabled: bool) -> RateLimitConfigSnapshot {
        RateLimitConfigSnapshot {
            enabled,
            trust_proxy: self.opts.trust_proxy,
            search_qps: self.opts.search_qps,
            search_burst: self.opts.search_burst,
            upload_qps: self.opts.upload_qps,
            upload_burst: self.opts.upload_burst,
            exempt_prefixes: self.opts.exempt_prefixes.clone(),
            rules: self.opts.rules.clone(),
        }
    }
}

#[derive(Serialize)]
struct L1CacheSnapshot {
    enabled: bool,
    total_searches: u64,
    hits: u64,
    misses: u64,
    hit_rate: f64,
}

#[derive(Serialize)]
struct L2CacheSnapshot {
    enabled: bool,
    l1_hits: u64,
    l1_misses: u64,
    l2_hits: u64,
    l2_misses: u64,
    total_items: u64,
    hit_rate: f64,
}

#[derive(Serialize)]
struct CacheCountersSnapshot {
    hits_total: i64,
    misses_total: i64,
}

#[derive(Serialize)]
struct CacheMonitorResponse {
    request_id: String,
    l1: L1CacheSnapshot,
    l2: L2CacheSnapshot,
    redis: crate::cache::redis_cache::RedisCacheSummary,
    counters: CacheCountersSnapshot,
}

#[derive(Serialize)]
struct RouteDropStat {
    route: String,
    drops: i64,
}

#[derive(Serialize)]
struct RateLimitConfigSnapshot {
    enabled: bool,
    trust_proxy: bool,
    search_qps: f64,
    search_burst: f64,
    upload_qps: f64,
    upload_burst: f64,
    exempt_prefixes: Vec<String>,
    rules: Vec<RouteRule>,
}

#[derive(Serialize)]
struct RateLimitMonitorResponse {
    request_id: String,
    total_drops: i64,
    drops_by_route: Vec<RouteDropStat>,
    config: RateLimitConfigSnapshot,
    limiter_state: RateLimiterState,
}

#[derive(serde::Deserialize)]
struct LogsQuery {
    limit: Option<usize>,
}

#[derive(serde::Deserialize)]
struct ChunkingQuery {
    limit: Option<usize>,
    capacity: Option<usize>,
}

#[derive(serde::Deserialize)]
struct LoggingQuery {
    enabled: Option<bool>,
}

#[derive(Serialize)]
struct LogEntry {
    timestamp: Option<String>,
    level: Option<String>,
    target: Option<String>,
    message: Option<String>,
    raw: String,
    fields: Option<Value>,
}

#[derive(Serialize)]
struct LogsResponse {
    request_id: String,
    file: Option<String>,
    entries: Vec<LogEntry>,
    note: Option<String>,
}

/// Generate a short request ID for correlation
fn generate_request_id() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}

pub async fn health_check() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    if let Some(retriever) = RETRIEVER.get() {
        let retriever = retriever.lock().unwrap();
        match retriever.health_check() {
            Ok(()) => Ok(HttpResponse::Ok().json(json!({
                "status": "healthy",
                "documents": retriever.metrics.total_documents_indexed,
                "vectors": retriever.metrics.total_vectors,
                "index_path": retriever.metrics.index_path,
                "request_id": request_id
            }))),
            Err(e) => {
                error!("[{}] Health check failed: {}", request_id, e);
                Ok(HttpResponse::ServiceUnavailable().json(json!({
                    "status": "unhealthy",
                    "error": e.to_string(),
                    "request_id": request_id
                })))
            }
        }
    } else {
        error!(
            "[{}] Health check failed: Retriever not initialized",
            request_id
        );
        Ok(HttpResponse::ServiceUnavailable().json(json!({
            "status": "unhealthy",
            "error": "Retriever not initialized",
            "request_id": request_id
        })))
    }
}

async fn root_handler() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("✅ Backend is running (Actix Web)\n\nTry /health or /ready\n"))
}

async fn ready_check() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    if let Some(retriever) = RETRIEVER.get() {
        match retriever.lock() {
            Ok(retriever) => match retriever.ready_check() {
                Ok(_) => Ok(HttpResponse::Ok().json(json!({
                    "status": "ready",
                    "timestamp": Utc::now().to_rfc3339(),
                    "request_id": request_id
                }))),
                Err(e) => Ok(HttpResponse::ServiceUnavailable().json(json!({
                    "status": "not ready",
                    "error": e.to_string(),
                    "timestamp": Utc::now().to_rfc3339(),
                    "request_id": request_id
                }))),
            },
            Err(e) => Ok(HttpResponse::ServiceUnavailable().json(json!({
                "status": "not ready",
                "error": format!("Failed to acquire lock: {}", e),
                "timestamp": Utc::now().to_rfc3339(),
                "request_id": request_id
            }))),
        }
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(json!({
            "status": "not ready",
            "message": "Retriever not initialized",
            "timestamp": Utc::now().to_rfc3339(),
            "request_id": request_id
        })))
    }
}

/// Phase 16: Export metrics in Prometheus text format
/// GET /monitoring/metrics
/// Returns: Prometheus-compliant text format metrics
async fn get_metrics() -> Result<HttpResponse, Error> {
    // Export metrics in Prometheus text format (not JSON)
    // Phase 16 Step 3: OTLP Exporting - Prometheus format compliance
    let prometheus_text = crate::monitoring::metrics::export_prometheus();

    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(prometheus_text))
}

/// Self-contained UI metrics: HTTP Requests summary + chart
/// GET /monitoring/ui/requests
/// Returns: JSON with rate, p95 latency, error%, and recent points
async fn get_ui_requests() -> Result<HttpResponse, Error> {
    let snapshot = crate::monitoring::get_requests_snapshot();
    Ok(HttpResponse::Ok().json(snapshot))
}

async fn get_chunking_stats(query: web::Query<ChunkingQuery>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();

    if let Some(new_cap) = query.capacity {
        let applied = crate::monitoring::set_chunking_history_capacity(new_cap);
        return Ok(HttpResponse::Ok().json(json!({
            "status": "ok",
            "request_id": request_id,
            "capacity_applied": applied,
            "message": "History capacity updated",
        })));
    }

    let limit = query.limit.unwrap_or(10);
    let history = crate::monitoring::chunking_snapshot_history(limit);

    if history.is_empty() {
        Ok(HttpResponse::Ok().json(json!({
            "status": "empty",
            "message": "No chunking stats recorded yet",
            "request_id": request_id
        })))
    } else {
        Ok(HttpResponse::Ok().json(json!({
            "status": "ok",
            "request_id": request_id,
            "count": history.len(),
            "snapshots": history,
        })))
    }
}

async fn toggle_chunking_logging(query: web::Query<LoggingQuery>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();

    if let Some(enabled) = query.enabled {
        crate::monitoring::set_chunking_logging_enabled(enabled);
        return Ok(HttpResponse::Ok().json(json!({
            "status": "ok",
            "request_id": request_id,
            "logging_enabled": enabled,
            "message": "Chunking snapshot logging updated",
        })));
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "request_id": request_id,
        "logging_enabled": crate::monitoring::chunking_logging_enabled(),
    })))
}

async fn get_cache_monitor_info() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let retriever = match RETRIEVER.get() {
        Some(handle) => handle,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "status": "unavailable",
                "error": "Retriever not initialized",
                "request_id": request_id,
            })));
        }
    };

    let (metrics_snapshot, l2_stats, redis_summary, l1_enabled, l2_enabled) = {
        let guard = retriever.lock().unwrap();
        (
            guard.metrics.clone(),
            guard.get_l2_cache_stats(),
            guard.get_l3_cache_summary(),
            guard.l1_cache_enabled(),
            guard.l2_cache_enabled(),
        )
    };

    let l1_snapshot = L1CacheSnapshot {
        enabled: l1_enabled,
        total_searches: metrics_snapshot.total_searches as u64,
        hits: metrics_snapshot.cache_hits as u64,
        misses: metrics_snapshot.cache_misses as u64,
        hit_rate: metrics_snapshot.cache_hit_rate(),
    };
    let l2_snapshot = L2CacheSnapshot {
        enabled: l2_enabled,
        l1_hits: l2_stats.l1_hits,
        l1_misses: l2_stats.l1_misses,
        l2_hits: l2_stats.l2_hits,
        l2_misses: l2_stats.l2_misses,
        total_items: l2_stats.total_items as u64,
        hit_rate: l2_stats.hit_rate(),
    };
    let counters = metrics::cache_hit_miss_counts();
    let counters_snapshot = CacheCountersSnapshot {
        hits_total: counters.0,
        misses_total: counters.1,
    };

    let response = CacheMonitorResponse {
        request_id,
        l1: l1_snapshot,
        l2: l2_snapshot,
        redis: redis_summary,
        counters: counters_snapshot,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn get_rate_limit_monitor_info(
    state: web::Data<RateLimitSharedState>,
) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let limiter_state = state.limiter.snapshot();
    let total_drops = metrics::rate_limit_drop_total();
    let drops_by_route = metrics::rate_limit_drops_by_route_snapshot()
        .into_iter()
        .map(|(route, drops)| RouteDropStat { route, drops })
        .collect();
    let config = state.config_snapshot(limiter_state.enabled);

    let response = RateLimitMonitorResponse {
        request_id,
        total_drops,
        drops_by_route,
        config,
        limiter_state,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn get_recent_logs(query: web::Query<LogsQuery>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let limit = query
        .limit
        .unwrap_or(DEFAULT_LOG_LIMIT)
        .clamp(1, MAX_LOG_LIMIT);
    let config = MonitoringConfig::from_env();
    let log_dir = config.log_dir;

    let file = latest_log_file(&log_dir);
    let (entries, note) = if let Some(path) = file.clone() {
        match read_recent_lines(&path, limit) {
            Ok(lines) => {
                let entries = lines
                    .into_iter()
                    .map(|line| parse_log_line(&line))
                    .collect();
                (entries, None)
            }
            Err(err) => {
                warn!(error = %err, path = %path.display(), "Failed to read logs");
                (Vec::new(), Some(format!("Failed to read logs: {}", err)))
            }
        }
    } else {
        (Vec::new(), Some("No backend log files found".to_string()))
    };

    let response = LogsResponse {
        request_id,
        file: file.and_then(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        }),
        entries,
        note,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn upload_document_inner(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    fs::create_dir_all(UPLOAD_DIR).ok();
    let mut uploaded_files = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let filename = field
            .content_disposition()
            .as_ref()
            .and_then(|cd| cd.get_filename())
            .ok_or_else(|| actix_web::error::ErrorBadRequest("No filename"))?
            .to_string();

        let ext = Path::new(&filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if ext != "txt" && ext != "pdf" {
            return Ok(HttpResponse::BadRequest().body("Only .txt/.pdf allowed"));
        }

        let filepath = format!("{}/{}", UPLOAD_DIR, filename);
        let mut f = web::block(move || std::fs::File::create(&filepath)).await??;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            f = web::block(move || f.write_all(&data).map(|_| f)).await??;
        }

        uploaded_files.push(filename);
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "uploaded_files": uploaded_files,
        "message": "Use /reindex to refresh index",
        "request_id": request_id
    })))
}

pub async fn list_documents() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(UPLOAD_DIR) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                if let Some(filename) = entry.file_name().to_str() {
                    files.push(filename.to_string());
                }
            }
        }
    }
    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "documents": files,
        "count": files.len(),
        "request_id": request_id
    })))
}

pub async fn delete_document(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let filename = path.into_inner();
    let filepath = format!("{}/{}", UPLOAD_DIR, filename);
    match fs::remove_file(&filepath) {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": format!("Deleted {}", filename),
            "request_id": request_id
        }))),
        Err(_) => Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "File not found",
            "request_id": request_id
        }))),
    }
}

pub async fn reindex_handler(config: web::Data<ApiConfig>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let start = std::time::Instant::now();

    // Phase 15: Check concurrency
    if REINDEX_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(HttpResponse::TooManyRequests().json(json!({
            "status": "busy",
            "message": "Reindex already in progress",
            "request_id": request_id
        })));
    }

    // Alerting config
    let hooks = crate::monitoring::alerting_hooks::AlertingHooksConfig::from_env();

    if let Some(retriever) = RETRIEVER.get() {
        let mut retriever = retriever.lock().unwrap();
        let chunker = crate::index::default_chunker(config.chunker_mode);
        let res = index::index_all_documents(
            &mut *retriever,
            UPLOAD_DIR,
            config.chunker_mode,
            chunker.as_ref(),
        );
        let duration_ms = start.elapsed().as_millis() as u64;
        let vectors = retriever.metrics.total_vectors as u64;
        let mappings = retriever.metrics.total_documents_indexed as u64;
        REINDEX_IN_PROGRESS.store(false, Ordering::SeqCst);

        // Fire webhook (non-blocking)
        let event = match res {
            Ok(_) => crate::monitoring::alerting_hooks::ReindexCompletionEvent::success(
                duration_ms,
                vectors,
                mappings,
            ),
            Err(_) => crate::monitoring::alerting_hooks::ReindexCompletionEvent::error(
                duration_ms,
                vectors,
                mappings,
            ),
        };
        actix_web::rt::spawn(async move {
            crate::monitoring::alerting_hooks::send_alert(&hooks, event).await;
        });

        match res {
            Ok(_) => Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Reindexing complete",
                "request_id": request_id
            }))),
            Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Reindex failed: {}", e),
                "request_id": request_id
            }))),
        }
    } else {
        REINDEX_IN_PROGRESS.store(false, Ordering::SeqCst);
        // Fire error webhook for missing retriever
        let hooks2 = crate::monitoring::alerting_hooks::AlertingHooksConfig::from_env();
        let event = crate::monitoring::alerting_hooks::ReindexCompletionEvent::error(0, 0, 0);
        actix_web::rt::spawn(async move {
            crate::monitoring::alerting_hooks::send_alert(&hooks2, event).await;
        });
        Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Retriever not initialized",
            "request_id": request_id
        })))
    }
}

/// Phase 15: Async reindex endpoint
pub async fn reindex_async_handler(config: web::Data<ApiConfig>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let job_id = Uuid::new_v4().to_string();

    // Check if already reindexing
    if REINDEX_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(HttpResponse::TooManyRequests().json(json!({
            "status": "busy",
            "message": "Reindex already in progress",
            "request_id": request_id
        })));
    }

    let job = AsyncJob {
        job_id: job_id.clone(),
        status: "pending".to_string(),
        started_at: Utc::now().to_rfc3339(),
        completed_at: None,
        vectors_indexed: None,
        mappings_indexed: None,
        error: None,
    };

    let jobs = get_jobs_map();
    jobs.lock().unwrap().insert(job_id.clone(), job.clone());

    // Spawn async task (non-blocking)
    let job_id_clone = job_id.clone();
    let retriever_handle = RETRIEVER.get().map(|h| Arc::clone(h));

    actix_web::rt::spawn(async move {
        let start = std::time::Instant::now();
        let hooks = crate::monitoring::alerting_hooks::AlertingHooksConfig::from_env();
        if let Some(retriever) = retriever_handle {
            let mut retriever = retriever.lock().unwrap();
            let mut job = jobs.lock().unwrap().get(&job_id_clone).cloned().unwrap();
            job.status = "running".to_string();
            jobs.lock().unwrap().insert(job_id_clone.clone(), job);

            let chunker = crate::index::default_chunker(config.chunker_mode);
            let res = index::index_all_documents(
                &mut *retriever,
                UPLOAD_DIR,
                config.chunker_mode,
                chunker.as_ref(),
            );

            let mut job = jobs.lock().unwrap().get(&job_id_clone).cloned().unwrap();
            let duration_ms = start.elapsed().as_millis() as u64;
            let vectors = retriever.metrics.total_vectors as u64;
            let mappings = retriever.metrics.total_documents_indexed as u64;

            match res {
                Ok(_) => {
                    job.status = "completed".to_string();
                    job.completed_at = Some(Utc::now().to_rfc3339());
                    job.vectors_indexed = Some(vectors as usize);
                    job.mappings_indexed = Some(mappings as usize);
                    let event = crate::monitoring::alerting_hooks::ReindexCompletionEvent::success(
                        duration_ms,
                        vectors,
                        mappings,
                    );
                    crate::monitoring::alerting_hooks::send_alert(&hooks, event).await;
                }
                Err(e) => {
                    job.status = "failed".to_string();
                    job.completed_at = Some(Utc::now().to_rfc3339());
                    job.error = Some(e.to_string());
                    let event = crate::monitoring::alerting_hooks::ReindexCompletionEvent::error(
                        duration_ms,
                        vectors,
                        mappings,
                    );
                    crate::monitoring::alerting_hooks::send_alert(&hooks, event).await;
                }
            }
            jobs.lock().unwrap().insert(job_id_clone.clone(), job);
        }
        REINDEX_IN_PROGRESS.store(false, Ordering::SeqCst);
    });

    Ok(HttpResponse::Accepted().json(json!({
        "status": "accepted",
        "job_id": job_id,
        "request_id": request_id
    })))
}

/// Phase 15: Check async job status
pub async fn reindex_status_handler(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let job_id = path.into_inner();

    let jobs = get_jobs_map();
    let jobs_lock = jobs.lock().unwrap();

    if let Some(job) = jobs_lock.get(&job_id) {
        Ok(HttpResponse::Ok().json(json!({
            "status": job.status,
            "job_id": job.job_id,
            "started_at": job.started_at,
            "completed_at": job.completed_at,
            "vectors_indexed": job.vectors_indexed,
            "mappings_indexed": job.mappings_indexed,
            "error": job.error,
            "request_id": request_id
        })))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "status": "not_found",
            "message": format!("Job {} not found", job_id),
            "request_id": request_id
        })))
    }
}

/// Phase 15: Index info endpoint
pub async fn index_info_handler() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let in_ram = std::env::var("INDEX_IN_RAM")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    if let Some(retriever) = RETRIEVER.get() {
        let retriever = retriever.lock().unwrap();
        Ok(HttpResponse::Ok().json(json!({
            "index_in_ram": in_ram,
            "mode": if in_ram { "RAM (fast)" } else { "Disk (standard)" },
            "warning": if in_ram {
                json!("INDEX_IN_RAM enabled: High memory usage for large datasets. Recommended for <100 docs only.")
            } else {
                json!(null)
            },
            "total_documents": retriever.metrics.total_documents_indexed,
            "total_vectors": retriever.metrics.total_vectors,
            "request_id": request_id
        })))
    } else {
        Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Retriever not initialized",
            "request_id": request_id
        })))
    }
}

async fn search_documents_inner(query: web::Query<SearchQuery>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    if let Some(retriever) = RETRIEVER.get() {
        let mut retriever = retriever.lock().unwrap();
        let results = retriever.search(&query.q).unwrap_or_default();
        Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "results": results,
            "request_id": request_id
        })))
    } else {
        Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Retriever not initialized",
            "request_id": request_id
        })))
    }
}

pub async fn rerank(request: web::Json<RerankRequest>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    if let Some(retriever) = RETRIEVER.get() {
        let retriever = retriever.lock().unwrap();
        let reranked = retriever.rerank_by_similarity(&request.query, &request.candidates);
        Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "results": reranked,
            "request_id": request_id
        })))
    } else {
        Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Retriever not initialized",
            "request_id": request_id
        })))
    }
}

pub async fn summarize(request: web::Json<SummarizeRequest>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    if let Some(retriever) = RETRIEVER.get() {
        let retriever = retriever.lock().unwrap();
        let summary = retriever.summarize_chunks(&request.query, &request.candidates);
        Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "summary": summary,
            "request_id": request_id
        })))
    } else {
        Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Retriever not initialized",
            "request_id": request_id
        })))
    }
}

pub async fn save_vectors_handler() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    if let Some(retriever) = RETRIEVER.get() {
        let mut retriever = retriever.lock().unwrap();
        match retriever.force_save() {
            Ok(_) => Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Vectors saved successfully",
                "vector_count": retriever.vectors.len(),
                "request_id": request_id
            }))),
            Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Failed to save vectors: {}", e),
                "request_id": request_id
            }))),
        }
    } else {
        Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Retriever not initialized",
            "request_id": request_id
        })))
    }
}

#[derive(serde::Deserialize)]
pub struct AgentRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

// Simple query variant for GET /agent/chat
#[derive(serde::Deserialize)]
pub struct AgentQueryParams {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}
fn default_limit() -> usize {
    20
}

#[derive(serde::Deserialize)]
pub struct StoreRagRequest {
    pub agent_id: String,
    pub memory_type: String,
    pub content: String,
    pub timestamp: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SearchRagRequest {
    pub agent_id: String,
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

#[derive(serde::Deserialize)]
pub struct RecallRagRequest {
    pub agent_id: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

async fn store_rag_memory(req: web::Json<StoreRagRequest>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let mem = AgentMemory::new("agent.db")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let ts = req
        .timestamp
        .clone()
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    mem.store_rag(&req.agent_id, &req.memory_type, &req.content, &ts)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "request_id": request_id
    })))
}

async fn search_rag_memory(req: web::Json<SearchRagRequest>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let mem = AgentMemory::new("agent.db")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let results: Vec<MemorySearchResult> = mem
        .search_rag(&req.agent_id, &req.query, req.top_k)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({
        "results": results,
        "request_id": request_id
    })))
}

async fn recall_rag_memory(req: web::Json<RecallRagRequest>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let mem = AgentMemory::new("agent.db")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let items: Vec<MemoryItem> = mem
        .recall_rag(&req.agent_id, req.limit)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({
        "items": items,
        "request_id": request_id
    })))
}

async fn run_agent(req: web::Json<AgentRequest>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    if let Some(retriever) = RETRIEVER.get() {
        let agent = Agent::new("default", "agent.db", Arc::clone(retriever));
        let resp: AgentResponse = agent.run(&req.query, req.top_k);
        Ok(HttpResponse::Ok().json(json!({
            "response": resp,
            "request_id": request_id
        })))
    } else {
        Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Retriever not initialized",
            "request_id": request_id
        })))
    }
}

// GET-based chat endpoint to avoid CORS preflight
async fn run_agent_get(query: web::Query<AgentQueryParams>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    if let Some(retriever) = RETRIEVER.get() {
        let agent = Agent::new("default", "agent.db", Arc::clone(retriever));
        let resp: AgentResponse = agent.run(&query.query, query.top_k);
        Ok(HttpResponse::Ok().json(json!({
            "response": resp,
            "request_id": request_id
        })))
    } else {
        Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Retriever not initialized",
            "request_id": request_id
        })))
    }
}

fn latest_log_file(log_dir: &Path) -> Option<PathBuf> {
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !file_name.starts_with(LOG_FILE_PREFIX) {
                continue;
            }
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let replace = newest
                .as_ref()
                .map(|(ts, _)| modified > *ts)
                .unwrap_or(true);
            if replace {
                newest = Some((modified, path));
            }
        }
    }
    newest.map(|(_, path)| path)
}

fn read_recent_lines(path: &Path, limit: usize) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut buffer = VecDeque::with_capacity(limit);
    for line in reader.lines() {
        let line = line?;
        if buffer.len() == limit {
            buffer.pop_front();
        }
        buffer.push_back(line);
    }
    Ok(buffer.into_iter().collect())
}

fn parse_log_line(line: &str) -> LogEntry {
    let parsed = serde_json::from_str::<Value>(line)
        .ok()
        .and_then(|value| match value {
            Value::Object(_) => Some(value),
            _ => None,
        });
    if let Some(value) = parsed {
        let timestamp = value
            .get("timestamp")
            .or_else(|| value.get("ts"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let level = value
            .get("level")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let target = value
            .get("target")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let fields = value.get("fields").cloned();
        let message = fields
            .as_ref()
            .and_then(|f| f.get("message"))
            .or_else(|| value.get("message"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        LogEntry {
            timestamp,
            level,
            target,
            message,
            raw: line.to_string(),
            fields,
        }
    } else {
        LogEntry {
            timestamp: None,
            level: None,
            target: None,
            message: None,
            raw: line.to_string(),
            fields: None,
        }
    }
}

pub fn start_api_server(
    config: &ApiConfig,
) -> impl std::future::Future<Output = std::io::Result<()>> {
    // Snapshot needed config values to satisfy 'static factory closure
    let bind_addr = config.bind_addr();
    let trust_proxy = config.trust_proxy;
    let rate_limit_enabled = config.rate_limit_enabled;
    let rate_limit_qps = config.rate_limit_qps;
    let rate_limit_burst = config.rate_limit_burst as f64;
    let rate_limit_lru_capacity = config.rate_limit_lru_capacity;
    let search_qps = config.rate_limit_search_qps.unwrap_or(rate_limit_qps);
    let search_burst = config
        .rate_limit_search_burst
        .unwrap_or(config.rate_limit_burst) as f64;
    let upload_qps = config.rate_limit_upload_qps.unwrap_or(rate_limit_qps);
    let upload_burst = config
        .rate_limit_upload_burst
        .unwrap_or(config.rate_limit_burst) as f64;

    let force_single_worker = std::env::var("NO_DOTENV")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);
    let mut http_server = HttpServer::new(move || {
        // Shared RateLimiter across workers (middleware-only enforcement)
        let rl_cfg = crate::security::rate_limiter::RateLimiterConfig {
            enabled: rate_limit_enabled,
            qps: rate_limit_qps.max(0.0),
            burst: rate_limit_burst,
            max_ips: rate_limit_lru_capacity,
        };
        let rl = std::sync::Arc::new(crate::security::rate_limiter::RateLimiter::new(rl_cfg));
        let opts = RateLimitOptions {
            trust_proxy,
            search_qps: search_qps.max(0.0),
            search_burst,
            upload_qps: upload_qps.max(0.0),
            upload_burst,
            rules: vec![
                RouteRule {
                    pattern: "/reindex".into(),
                    match_kind: MatchKind::Exact,
                    qps: 0.5,
                    burst: 2.0,
                    label: Some("admin-reindex".into()),
                },
                RouteRule {
                    pattern: "/upload".into(),
                    match_kind: MatchKind::Prefix,
                    qps: upload_qps.max(0.0),
                    burst: upload_burst.max(0.0),
                    label: Some("upload".into()),
                },
            ],
            exempt_prefixes: vec![
                "/".into(),
                "/health".into(),
                "/ready".into(),
                "/metrics".into(),
                "/monitoring".into(),
            ],
        }
        .with_env_overrides();
        let rate_limit_state_data = web::Data::new(RateLimitSharedState {
            limiter: rl.clone(),
            opts: opts.clone(),
        });

        // Log effective rate limit options for visibility
        info!(
            trust_proxy = opts.trust_proxy,
            search_qps = opts.search_qps,
            search_burst = opts.search_burst,
            upload_qps = opts.upload_qps,
            upload_burst = opts.upload_burst,
            rules = %serde_json::to_string(&opts.rules).unwrap_or_default(),
            exempt_prefixes = %serde_json::to_string(&opts.exempt_prefixes).unwrap_or_default(),
            "Rate limit options initialized"
        );
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
            ])
            .max_age(3600);

        App::new()
            .app_data(rate_limit_state_data.clone())
            .wrap(cors)
            .wrap(crate::trace_middleware::TraceMiddleware::new())
            .wrap(
                crate::monitoring::rate_limit_middleware::RateLimitMiddleware::new_with_options(
                    rl.clone(),
                    opts.clone(),
                ),
            )
            // ============================================================================
            // MONITORING ROUTES (Phase 16 Step 3 - OTLP Exporting)
            // Exports metrics in Prometheus text format for Prometheus scraping
            // ============================================================================
            .service(
                web::scope("/monitoring")
                    .route("/health", web::get().to(health_check))
                    .route("/ready", web::get().to(ready_check))
                    .route("/metrics", web::get().to(get_metrics)) // ← Prometheus format
                    .route("/ui/requests", web::get().to(get_ui_requests)) // ← Self-contained UI metrics for Requests
                    .route("/chunking/latest", web::get().to(get_chunking_stats))
                    .route("/chunking/logging", web::get().to(toggle_chunking_logging)),
            )
            // ============================================================================
            // ROOT & CORE ROUTES
            // ============================================================================
            .route("/", web::get().to(root_handler))
            .route("/upload", web::post().to(upload_document_inner))
            .route("/documents", web::get().to(list_documents))
            .route("/documents/{filename}", web::delete().to(delete_document))
            .route("/reindex", web::post().to(reindex_handler))
            .route("/reindex/async", web::post().to(reindex_async_handler))
            .route(
                "/reindex/status/{job_id}",
                web::get().to(reindex_status_handler),
            )
            .route("/index/info", web::get().to(index_info_handler))
            .route("/search", web::get().to(search_documents_inner))
            .route("/rerank", web::post().to(rerank))
            .route("/summarize", web::post().to(summarize))
            .route("/save_vectors", web::post().to(save_vectors_handler))
            .route("/monitor/cache/info", web::get().to(get_cache_monitor_info))
            .route(
                "/monitor/rate_limits/info",
                web::get().to(get_rate_limit_monitor_info),
            )
            .route("/monitor/logs/recent", web::get().to(get_recent_logs))
            // ============================================================================
            // RAG MEMORY ROUTES
            // ============================================================================
            .route("/memory/store_rag", web::post().to(store_rag_memory))
            .route("/memory/search_rag", web::post().to(search_rag_memory))
            .route("/memory/recall_rag", web::post().to(recall_rag_memory))
            // ============================================================================
            // AGENT ROUTES
            // ============================================================================
            .route("/agent", web::post().to(run_agent))
            .route("/agent/chat", web::get().to(run_agent_get))
    });
    if force_single_worker {
        http_server = http_server.workers(1);
    }
    http_server
        .bind(bind_addr.clone())
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", bind_addr, e))
        .run()
}
