use actix_web::{web, App, HttpResponse, HttpServer, Error};
use actix_cors::Cors;
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use tracing::{error, info};
use serde_json::json;
use crate::retriever::Retriever;
use crate::index;
use crate::agent::{Agent, AgentResponse};
use crate::agent_memory::{AgentMemory, MemoryItem, MemorySearchResult};
use chrono::Utc;
use crate::config::ApiConfig;
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
    pub q: String 
}

#[derive(serde::Deserialize)]
pub struct RerankRequest { 
    pub query: String, 
    pub candidates: Vec<String> 
}

#[derive(serde::Deserialize)]
pub struct SummarizeRequest { 
    pub query: String, 
    pub candidates: Vec<String> 
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
            Ok(()) => {
                Ok(HttpResponse::Ok().json(json!({
                    "status": "healthy",
                    "documents": retriever.metrics.total_documents_indexed,
                    "vectors": retriever.metrics.total_vectors,
                    "index_path": retriever.metrics.index_path,
                    "request_id": request_id
                })))
            },
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
        error!("[{}] Health check failed: Retriever not initialized", request_id);
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
            Ok(retriever) => {
                match retriever.ready_check() {
                    Ok(_) => {
                        Ok(HttpResponse::Ok().json(json!({
                            "status": "ready",
                            "timestamp": Utc::now().to_rfc3339(),
                            "request_id": request_id
                        })))
                    }
                    Err(e) => {
                        Ok(HttpResponse::ServiceUnavailable().json(json!({
                            "status": "not ready",
                            "error": e.to_string(),
                            "timestamp": Utc::now().to_rfc3339(),
                            "request_id": request_id
                        })))
                    }
                }
            }
            Err(e) => {
                Ok(HttpResponse::ServiceUnavailable().json(json!({
                    "status": "not ready",
                    "error": format!("Failed to acquire lock: {}", e),
                    "timestamp": Utc::now().to_rfc3339(),
                    "request_id": request_id
                })))
            }
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

        let ext = Path::new(&filename).extension().and_then(|s| s.to_str()).unwrap_or("");
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
        })))
    }
}

pub async fn reindex_handler() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let start = std::time::Instant::now();

    // Phase 15: Check concurrency
    if REINDEX_IN_PROGRESS.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
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
        let res = index::index_all_documents(&mut *retriever, UPLOAD_DIR);
        let duration_ms = start.elapsed().as_millis() as u64;
        let vectors = retriever.metrics.total_vectors as u64;
        let mappings = retriever.metrics.total_documents_indexed as u64;
        REINDEX_IN_PROGRESS.store(false, Ordering::SeqCst);

        // Fire webhook (non-blocking)
        let event = match res {
            Ok(_) => crate::monitoring::alerting_hooks::ReindexCompletionEvent::success(duration_ms, vectors, mappings),
            Err(_) => crate::monitoring::alerting_hooks::ReindexCompletionEvent::error(duration_ms, vectors, mappings),
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
pub async fn reindex_async_handler() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let job_id = Uuid::new_v4().to_string();
    
    // Check if already reindexing
    if REINDEX_IN_PROGRESS.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
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

            let res = index::index_all_documents(&mut *retriever, UPLOAD_DIR);

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
                    let event = crate::monitoring::alerting_hooks::ReindexCompletionEvent::success(duration_ms, vectors, mappings);
                    crate::monitoring::alerting_hooks::send_alert(&hooks, event).await;
                }
                Err(e) => {
                    job.status = "failed".to_string();
                    job.completed_at = Some(Utc::now().to_rfc3339());
                    job.error = Some(e.to_string());
                    let event = crate::monitoring::alerting_hooks::ReindexCompletionEvent::error(duration_ms, vectors, mappings);
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
            })))
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
fn default_top_k() -> usize { 5 }
fn default_limit() -> usize { 20 }

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
    let mem = AgentMemory::new("agent.db").map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let ts = req.timestamp.clone().unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    mem.store_rag(&req.agent_id, &req.memory_type, &req.content, &ts)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({ 
        "status": "success",
        "request_id": request_id
    })))
}


async fn search_rag_memory(req: web::Json<SearchRagRequest>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let mem = AgentMemory::new("agent.db").map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let results: Vec<MemorySearchResult> = mem.search_rag(&req.agent_id, &req.query, req.top_k)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(json!({
        "results": results,
        "request_id": request_id
    })))
}


async fn recall_rag_memory(req: web::Json<RecallRagRequest>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let mem = AgentMemory::new("agent.db").map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let items: Vec<MemoryItem> = mem.recall_rag(&req.agent_id, req.limit)
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

pub fn start_api_server(config: &ApiConfig) -> impl std::future::Future<Output = std::io::Result<()>> {
    // Snapshot needed config values to satisfy 'static factory closure
    let bind_addr = config.bind_addr();
    let trust_proxy = config.trust_proxy;
    let rate_limit_enabled = config.rate_limit_enabled;
    let rate_limit_qps = config.rate_limit_qps;
    let rate_limit_burst = config.rate_limit_burst as f64;
    let rate_limit_lru_capacity = config.rate_limit_lru_capacity;
    let search_qps = config.rate_limit_search_qps.unwrap_or(rate_limit_qps);
    let search_burst = config.rate_limit_search_burst.unwrap_or(config.rate_limit_burst) as f64;
    let upload_qps = config.rate_limit_upload_qps.unwrap_or(rate_limit_qps);
    let upload_burst = config.rate_limit_upload_burst.unwrap_or(config.rate_limit_burst) as f64;

    HttpServer::new(move || {
        // Shared RateLimiter across workers (middleware-only enforcement)
        let rl_cfg = crate::security::rate_limiter::RateLimiterConfig {
            enabled: rate_limit_enabled,
            qps: rate_limit_qps.max(0.0),
            burst: rate_limit_burst,
            max_ips: rate_limit_lru_capacity,
        };
        let rl = std::sync::Arc::new(crate::security::rate_limiter::RateLimiter::new(rl_cfg));
        let opts = crate::monitoring::rate_limit_middleware::RateLimitOptions {
            trust_proxy,
            search_qps: search_qps.max(0.0),
            search_burst,
            upload_qps: upload_qps.max(0.0),
            upload_burst,
            rules: vec![
                // Example stricter admin rule
                crate::monitoring::rate_limit_middleware::RouteRule { pattern: "/reindex".into(), match_kind: crate::monitoring::rate_limit_middleware::MatchKind::Exact, qps: 0.5, burst: 2.0, label: Some("admin-reindex".into()) },
                // Ensure upload constrained by prefix
                crate::monitoring::rate_limit_middleware::RouteRule { pattern: "/upload".into(), match_kind: crate::monitoring::rate_limit_middleware::MatchKind::Prefix, qps: upload_qps.max(0.0), burst: upload_burst.max(0.0), label: Some("upload".into()) },
            ],
            exempt_prefixes: vec!["/".into(), "/health".into(), "/ready".into(), "/metrics".into(), "/monitoring".into()],
        }.with_env_overrides();

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
            .allowed_origin("http://127.0.0.1:3011")
            .allowed_origin("http://localhost:3011")
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
            ])
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .wrap(crate::trace_middleware::TraceMiddleware::new())
            .wrap(crate::monitoring::rate_limit_middleware::RateLimitMiddleware::new_with_options(rl.clone(), opts.clone()))
            
            // ============================================================================
            // MONITORING ROUTES (Phase 16 Step 3 - OTLP Exporting)
            // Exports metrics in Prometheus text format for Prometheus scraping
            // ============================================================================
            .service(
                web::scope("/monitoring")
                    .route("/health", web::get().to(health_check))
                    .route("/ready", web::get().to(ready_check))
                    .route("/metrics", web::get().to(get_metrics))  // ← Prometheus format
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
            .route("/reindex/status/{job_id}", web::get().to(reindex_status_handler))
            .route("/index/info", web::get().to(index_info_handler))
            .route("/search", web::get().to(search_documents_inner))
            .route("/rerank", web::post().to(rerank))
            .route("/summarize", web::post().to(summarize))
            .route("/save_vectors", web::post().to(save_vectors_handler))
            
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
    })
    .bind(bind_addr.clone())
    .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", bind_addr, e))
    .run()
}