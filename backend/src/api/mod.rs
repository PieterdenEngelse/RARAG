use crate::agent::{Agent, AgentResponse};
use crate::agent_memory::{AgentMemory, MemoryItem, MemorySearchResult};
use crate::config::ApiConfig;
use crate::db::chunk_settings;
use crate::db::llm_settings::{self, LlmConfig};
use crate::index;
use crate::memory::chunker::ChunkerConfig;
use crate::monitoring::config::MonitoringConfig;
use crate::monitoring::metrics;
use crate::monitoring::rate_limit_middleware::{MatchKind, RateLimitOptions, RouteRule};
use crate::retriever::Retriever;
use crate::security::rate_limiter::{RateLimiter, RateLimiterState};
use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{http::StatusCode, web, App, Error, HttpResponse, HttpServer};
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

#[derive(Debug, serde::Deserialize)]
struct ChunkConfigCommitRequest {
    target_size: usize,
    min_size: usize,
    max_size: usize,
    overlap: usize,
    #[serde(default)]
    semantic_similarity_threshold: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
struct ChunkerConfigSnapshot {
    target_size: usize,
    min_size: usize,
    max_size: usize,
    overlap: usize,
    semantic_similarity_threshold: f32,
}

impl From<&ChunkerConfig> for ChunkerConfigSnapshot {
    fn from(cfg: &ChunkerConfig) -> Self {
        Self {
            target_size: cfg.target_size,
            min_size: cfg.min_size,
            max_size: cfg.max_size,
            overlap: cfg.overlap,
            semantic_similarity_threshold: cfg.semantic_similarity_threshold,
        }
    }
}

#[derive(Debug, Serialize)]
struct ChunkCommitResponse {
    status: String,
    message: String,
    request_id: String,
    chunker_config: ChunkerConfigSnapshot,
    reindex_status: String,
    reindex_job_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct LlmConfigRequest {
    temperature: f32,
    top_p: f32,
    top_k: usize,
    max_tokens: usize,
    repeat_penalty: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
    stop_sequences: Vec<String>,
    seed: Option<i64>,
    #[serde(default = "default_min_p")]
    min_p: f32,
    #[serde(default = "default_typical_p")]
    typical_p: f32,
    #[serde(default = "default_tfs_z")]
    tfs_z: f32,
    #[serde(default = "default_mirostat")]
    mirostat: i32,
    #[serde(default = "default_mirostat_eta")]
    mirostat_eta: f32,
    #[serde(default = "default_mirostat_tau")]
    mirostat_tau: f32,
    #[serde(default = "default_repeat_last_n")]
    repeat_last_n: usize,
}

fn default_min_p() -> f32 {
    llm_settings::DEFAULT_MIN_P
}
fn default_typical_p() -> f32 {
    llm_settings::DEFAULT_TYPICAL_P
}
fn default_tfs_z() -> f32 {
    llm_settings::DEFAULT_TFS_Z
}
fn default_mirostat() -> i32 {
    llm_settings::DEFAULT_MIROSTAT
}
fn default_mirostat_eta() -> f32 {
    llm_settings::DEFAULT_MIROSTAT_ETA
}
fn default_mirostat_tau() -> f32 {
    llm_settings::DEFAULT_MIROSTAT_TAU
}
fn default_repeat_last_n() -> usize {
    llm_settings::DEFAULT_REPEAT_LAST_N
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct HardwareConfigRequest {
    backend_type: String,
    model: String,
    num_thread: usize,
    num_gpu: usize,
    gpu_layers: usize,
    main_gpu: usize,
    low_vram: bool,
    f16_kv: bool,
    rope_frequency_base: f32,
    rope_frequency_scale: f32,
    numa: bool,
    num_ctx: usize,
    num_batch: usize,
    logits_all: bool,
    vocab_only: bool,
    use_mmap: bool,
    use_mlock: bool,
}

impl Default for HardwareConfigRequest {
    fn default() -> Self {
        crate::db::param_hardware::HardwareParams::default().into()
    }
}

impl From<crate::db::param_hardware::HardwareParams> for HardwareConfigRequest {
    fn from(params: crate::db::param_hardware::HardwareParams) -> Self {
        Self {
            backend_type: backend_type_to_string(&params.backend_type),
            model: params.model,
            num_thread: params.num_thread,
            num_gpu: params.num_gpu,
            gpu_layers: params.gpu_layers,
            main_gpu: params.main_gpu,
            low_vram: params.low_vram,
            f16_kv: params.f16_kv,
            rope_frequency_base: params.rope_frequency_base,
            rope_frequency_scale: params.rope_frequency_scale,
            numa: params.numa,
            num_ctx: params.num_ctx,
            num_batch: params.num_batch,
            logits_all: params.logits_all,
            vocab_only: params.vocab_only,
            use_mmap: params.use_mmap,
            use_mlock: params.use_mlock,
        }
    }
}

impl From<HardwareConfigRequest> for crate::db::param_hardware::HardwareParams {
    fn from(req: HardwareConfigRequest) -> Self {
        Self {
            backend_type: string_to_backend_type(&req.backend_type),
            model: req.model,
            num_thread: req.num_thread,
            num_gpu: req.num_gpu,
            gpu_layers: req.gpu_layers,
            main_gpu: req.main_gpu,
            low_vram: req.low_vram,
            f16_kv: req.f16_kv,
            rope_frequency_base: req.rope_frequency_base,
            rope_frequency_scale: req.rope_frequency_scale,
            numa: req.numa,
            num_ctx: req.num_ctx,
            num_batch: req.num_batch,
            logits_all: req.logits_all,
            vocab_only: req.vocab_only,
            use_mmap: req.use_mmap,
            use_mlock: req.use_mlock,
        }
    }
}

fn backend_type_to_string(bt: &crate::db::param_hardware::BackendType) -> String {
    use crate::db::param_hardware::BackendType;
    match bt {
        BackendType::Ollama => "ollama".to_string(),
        BackendType::LlamaCpp => "llama_cpp".to_string(),
        BackendType::OpenAi => "openai".to_string(),
        BackendType::Anthropic => "anthropic".to_string(),
        BackendType::Vllm => "vllm".to_string(),
        BackendType::Custom => "custom".to_string(),
    }
}

fn string_to_backend_type(s: &str) -> crate::db::param_hardware::BackendType {
    use crate::db::param_hardware::BackendType;
    match s {
        "ollama" => BackendType::Ollama,
        "llama_cpp" => BackendType::LlamaCpp,
        "openai" => BackendType::OpenAi,
        "anthropic" => BackendType::Anthropic,
        "vllm" => BackendType::Vllm,
        "custom" => BackendType::Custom,
        _ => BackendType::Ollama, // default fallback
    }
}

#[derive(Debug, Serialize)]
struct LlmConfigResponse {
    status: String,
    message: String,
    request_id: String,
    config: LlmConfig,
}

#[derive(Debug, Serialize)]
struct HardwareConfigResponse {
    status: String,
    message: String,
    request_id: String,
    config: HardwareConfigRequest,
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

fn validate_chunk_request(req: &ChunkConfigCommitRequest) -> Result<(), String> {
    if req.min_size == 0 {
        return Err("min_size must be greater than 0".into());
    }
    if req.min_size > req.target_size {
        return Err("min_size cannot exceed target_size".into());
    }
    if req.target_size > req.max_size {
        return Err("target_size cannot exceed max_size".into());
    }
    if req.overlap >= req.target_size {
        return Err("overlap must be smaller than target_size".into());
    }
    if req.max_size == 0 {
        return Err("max_size must be greater than 0".into());
    }
    if req
        .semantic_similarity_threshold
        .map_or(false, |v| !(0.0..=1.0).contains(&v))
    {
        return Err("semantic_similarity_threshold must be between 0 and 1".into());
    }
    Ok(())
}

fn validate_llm_request(req: &LlmConfigRequest) -> Result<(), String> {
    if !(0.0..=2.0).contains(&req.temperature) {
        return Err("temperature must be between 0 and 2".into());
    }
    if !(0.0..=1.0).contains(&req.top_p) {
        return Err("top_p must be between 0 and 1".into());
    }
    if req.top_k == 0 {
        return Err("top_k must be greater than 0".into());
    }
    if req.max_tokens == 0 {
        return Err("max_tokens must be greater than 0".into());
    }
    if req.repeat_penalty < 1.0 {
        return Err("repeat_penalty must be at least 1.0".into());
    }
    if !(0.0..=2.0).contains(&req.frequency_penalty) {
        return Err("frequency_penalty must be between 0 and 2".into());
    }
    if !(0.0..=2.0).contains(&req.presence_penalty) {
        return Err("presence_penalty must be between 0 and 2".into());
    }
    if !(0.0..=1.0).contains(&req.min_p) {
        return Err("min_p must be between 0 and 1".into());
    }
    if !(0.0..=1.0).contains(&req.typical_p) {
        return Err("typical_p must be between 0 and 1".into());
    }
    if !(0.0..=1.0).contains(&req.tfs_z) {
        return Err("tfs_z must be between 0 and 1".into());
    }
    if !(0..=2).contains(&req.mirostat) {
        return Err("mirostat must be 0, 1, or 2".into());
    }
    if !(0.0..=1.0).contains(&req.mirostat_eta) {
        return Err("mirostat_eta must be between 0 and 1".into());
    }
    if !(0.0..=10.0).contains(&req.mirostat_tau) {
        return Err("mirostat_tau must be between 0 and 10".into());
    }
    if req.repeat_last_n == 0 {
        return Err("repeat_last_n must be greater than 0".into());
    }
    Ok(())
}

fn validate_hardware_request(req: &HardwareConfigRequest) -> Result<(), String> {
    if req.num_thread == 0 {
        return Err("num_thread must be greater than 0".into());
    }
    if req.num_gpu > 64 {
        return Err("num_gpu must be 64 or less".into());
    }
    if req.main_gpu > 64 {
        return Err("main_gpu index must be 64 or less".into());
    }
    if req.rope_frequency_base <= 0.0 {
        return Err("rope_frequency_base must be positive".into());
    }
    if req.rope_frequency_scale <= 0.0 {
        return Err("rope_frequency_scale must be positive".into());
    }
    if req.num_ctx == 0 {
        return Err("num_ctx must be greater than 0".into());
    }
    if req.num_batch == 0 {
        return Err("num_batch must be greater than 0".into());
    }
    Ok(())
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

async fn commit_chunk_config(
    config: web::Data<ApiConfig>,
    payload: web::Json<ChunkConfigCommitRequest>,
) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let body = payload.into_inner();
    if let Err(msg) = validate_chunk_request(&body) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "invalid",
            "message": msg,
            "request_id": request_id
        })));
    }

    let new_cfg = ChunkerConfig {
        target_size: body.target_size,
        min_size: body.min_size,
        max_size: body.max_size,
        overlap: body.overlap,
        semantic_similarity_threshold: body
            .semantic_similarity_threshold
            .unwrap_or_else(|| chunk_settings::global_config().semantic_similarity_threshold),
    };

    match chunk_settings::save_chunker_config_default_db(&new_cfg) {
        Ok(_) => {
            tracing::info!(
                request_id = %request_id,
                target = new_cfg.target_size,
                min = new_cfg.min_size,
                max = new_cfg.max_size,
                overlap = new_cfg.overlap,
                "Chunk config committed"
            );
        }
        Err(err) => {
            tracing::error!(
                request_id = %request_id,
                error = %err,
                "Failed to save chunk config"
            );
            return Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Failed to save chunk config: {}", err),
                "request_id": request_id
            })));
        }
    }

    let chunk_snapshot = ChunkerConfigSnapshot::from(&new_cfg);

    match launch_async_reindex_job(config) {
        Ok(job_id) => Ok(HttpResponse::Accepted().json(ChunkCommitResponse {
            status: "accepted".into(),
            message: "Chunk settings saved; reindex started".into(),
            request_id,
            chunker_config: chunk_snapshot,
            reindex_status: "accepted".into(),
            reindex_job_id: Some(job_id),
        })),
        Err((status, message)) => {
            let http_status = if status == StatusCode::TOO_MANY_REQUESTS {
                StatusCode::TOO_MANY_REQUESTS
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            tracing::warn!(
                request_id = %request_id,
                status = %http_status.as_u16(),
                message = %message,
                "Chunk commit applied but reindex not started"
            );
            Ok(HttpResponse::build(http_status).json(ChunkCommitResponse {
                status: "saved_pending_reindex".into(),
                message: format!("Settings saved, but reindex not started: {}", message),
                request_id,
                chunker_config: chunk_snapshot,
                reindex_status: "skipped".into(),
                reindex_job_id: None,
            }))
        }
    }
}

async fn get_llm_config() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let config = llm_settings::global_config();
    Ok(HttpResponse::Ok().json(LlmConfigResponse {
        status: "ok".into(),
        message: "Current LLM configuration".into(),
        request_id,
        config,
    }))
}

async fn commit_llm_config(payload: web::Json<LlmConfigRequest>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let body = payload.into_inner();

    if let Err(msg) = validate_llm_request(&body) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "invalid",
            "message": msg,
            "request_id": request_id
        })));
    }

    let new_cfg = LlmConfig {
        temperature: body.temperature,
        top_p: body.top_p,
        top_k: body.top_k,
        max_tokens: body.max_tokens,
        repeat_penalty: body.repeat_penalty,
        frequency_penalty: body.frequency_penalty,
        presence_penalty: body.presence_penalty,
        stop_sequences: body.stop_sequences,
        seed: body.seed,
        min_p: body.min_p,
        typical_p: body.typical_p,
        tfs_z: body.tfs_z,
        mirostat: body.mirostat,
        mirostat_eta: body.mirostat_eta,
        mirostat_tau: body.mirostat_tau,
        repeat_last_n: body.repeat_last_n,
    };

    match llm_settings::save_llm_config_default_db(&new_cfg) {
        Ok(_) => {
            tracing::info!(
                request_id = %request_id,
                temperature = new_cfg.temperature,
                top_p = new_cfg.top_p,
                top_k = new_cfg.top_k,
                max_tokens = new_cfg.max_tokens,
                repeat_penalty = new_cfg.repeat_penalty,
                frequency_penalty = new_cfg.frequency_penalty,
                presence_penalty = new_cfg.presence_penalty,
                stop_sequences = ?new_cfg.stop_sequences,
                seed = ?new_cfg.seed,
                "LLM config committed"
            );
            Ok(HttpResponse::Ok().json(LlmConfigResponse {
                status: "ok".into(),
                message: "LLM settings saved".into(),
                request_id,
                config: new_cfg,
            }))
        }
        Err(err) => {
            tracing::error!(
                request_id = %request_id,
                error = %err,
                "Failed to save LLM config"
            );
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Failed to save LLM config: {}", err),
                "request_id": request_id
            })))
        }
    }
}

async fn get_hardware_config() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let config = crate::db::param_hardware::global_config().into();
    Ok(HttpResponse::Ok().json(HardwareConfigResponse {
        status: "ok".into(),
        message: "".into(),
        request_id,
        config,
    }))
}

async fn commit_hardware_config(
    payload: web::Json<HardwareConfigRequest>,
) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let body = payload.into_inner();

    if let Err(msg) = validate_hardware_request(&body) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "invalid",
            "message": msg,
            "request_id": request_id
        })));
    }

    let params = crate::db::param_hardware::HardwareParams::from(body.clone());
    match crate::db::param_hardware::save_default_db(&params) {
        Ok(_) => {
            tracing::info!(
                request_id = %request_id,
                num_thread = params.num_thread,
                num_gpu = params.num_gpu,
                gpu_layers = params.gpu_layers,
                main_gpu = params.main_gpu,
                low_vram = params.low_vram,
                f16_kv = params.f16_kv,
                rope_frequency_base = params.rope_frequency_base,
                rope_frequency_scale = params.rope_frequency_scale,
                "Hardware config committed"
            );
            Ok(HttpResponse::Ok().json(HardwareConfigResponse {
                status: "ok".into(),
                message: "Hardware settings saved".into(),
                request_id,
                config: body,
            }))
        }
        Err(err) => {
            tracing::error!(
                request_id = %request_id,
                error = %err,
                "Failed to save hardware config"
            );
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Failed to save hardware config: {}", err),
                "request_id": request_id
            })))
        }
    }
}

// ============================================================================
// API KEYS CONFIG
// ============================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ApiKeysRequest {
    #[serde(default)]
    openai_api_key: String,
    #[serde(default)]
    anthropic_api_key: String,
}

#[derive(Debug, Serialize)]
struct ApiKeysResponse {
    status: String,
    message: String,
    request_id: String,
    has_openai_key: bool,
    has_anthropic_key: bool,
    openai_key_masked: String,
    anthropic_key_masked: String,
    openai_from_env: bool,
    anthropic_from_env: bool,
}

async fn get_api_keys() -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let keys = crate::db::api_keys::global_config();

    let openai_from_env = std::env::var("OPENAI_API_KEY").is_ok();
    let anthropic_from_env = std::env::var("ANTHROPIC_API_KEY").is_ok();

    let openai_key_masked = if openai_from_env {
        "[from environment]".to_string()
    } else if !keys.openai_api_key.is_empty() {
        crate::db::api_keys::ApiKeys::mask_key(&keys.openai_api_key)
    } else {
        String::new()
    };

    let anthropic_key_masked = if anthropic_from_env {
        "[from environment]".to_string()
    } else if !keys.anthropic_api_key.is_empty() {
        crate::db::api_keys::ApiKeys::mask_key(&keys.anthropic_api_key)
    } else {
        String::new()
    };

    Ok(HttpResponse::Ok().json(ApiKeysResponse {
        status: "ok".into(),
        message: "API keys status".into(),
        request_id,
        has_openai_key: keys.has_openai_key(),
        has_anthropic_key: keys.has_anthropic_key(),
        openai_key_masked,
        anthropic_key_masked,
        openai_from_env,
        anthropic_from_env,
    }))
}

async fn save_api_keys(payload: web::Json<ApiKeysRequest>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let body = payload.into_inner();

    // Get current keys and update only non-empty values
    let mut keys = crate::db::api_keys::global_config();

    if !body.openai_api_key.is_empty() {
        keys.openai_api_key = body.openai_api_key;
    }
    if !body.anthropic_api_key.is_empty() {
        keys.anthropic_api_key = body.anthropic_api_key;
    }

    match crate::db::api_keys::update_config(keys.clone()) {
        Ok(_) => {
            tracing::info!(
                request_id = %request_id,
                has_openai = keys.has_openai_key(),
                has_anthropic = keys.has_anthropic_key(),
                "API keys saved"
            );

            let openai_from_env = std::env::var("OPENAI_API_KEY").is_ok();
            let anthropic_from_env = std::env::var("ANTHROPIC_API_KEY").is_ok();

            Ok(HttpResponse::Ok().json(ApiKeysResponse {
                status: "ok".into(),
                message: "API keys saved".into(),
                request_id,
                has_openai_key: keys.has_openai_key(),
                has_anthropic_key: keys.has_anthropic_key(),
                openai_key_masked: if openai_from_env {
                    "[from environment]".to_string()
                } else if !keys.openai_api_key.is_empty() {
                    crate::db::api_keys::ApiKeys::mask_key(&keys.openai_api_key)
                } else {
                    String::new()
                },
                anthropic_key_masked: if anthropic_from_env {
                    "[from environment]".to_string()
                } else if !keys.anthropic_api_key.is_empty() {
                    crate::db::api_keys::ApiKeys::mask_key(&keys.anthropic_api_key)
                } else {
                    String::new()
                },
                openai_from_env,
                anthropic_from_env,
            }))
        }
        Err(err) => {
            tracing::error!(
                request_id = %request_id,
                error = %err,
                "Failed to save API keys"
            );
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Failed to save API keys: {}", err),
                "request_id": request_id
            })))
        }
    }
}

async fn delete_api_key(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let provider = path.into_inner();

    let mut keys = crate::db::api_keys::global_config();

    match provider.as_str() {
        "openai" => {
            keys.openai_api_key = String::new();
        }
        "anthropic" => {
            keys.anthropic_api_key = String::new();
        }
        _ => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": format!("Unknown provider: {}. Use 'openai' or 'anthropic'", provider),
                "request_id": request_id
            })));
        }
    }

    match crate::db::api_keys::update_config(keys) {
        Ok(_) => {
            tracing::info!(
                request_id = %request_id,
                provider = %provider,
                "API key deleted"
            );
            Ok(HttpResponse::Ok().json(json!({
                "status": "ok",
                "message": format!("{} API key deleted", provider),
                "request_id": request_id
            })))
        }
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to delete API key: {}", err),
            "request_id": request_id
        }))),
    }
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

#[derive(Debug, serde::Deserialize)]
struct SetRateLimitEnabledRequest {
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct SetRateLimitEnabledResponse {
    request_id: String,
    enabled: bool,
    message: String,
}

async fn set_rate_limit_enabled(
    state: web::Data<RateLimitSharedState>,
    body: web::Json<SetRateLimitEnabledRequest>,
) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();
    let new_state = state.limiter.set_enabled(body.enabled);

    let message = if new_state {
        "Rate limiter enabled".to_string()
    } else {
        "Rate limiter disabled".to_string()
    };

    tracing::info!("[{}] Rate limiter set to: {}", request_id, new_state);

    Ok(HttpResponse::Ok().json(SetRateLimitEnabledResponse {
        request_id,
        enabled: new_state,
        message,
    }))
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

async fn upload_document_inner(
    mut payload: Multipart,
    config: web::Data<ApiConfig>,
) -> Result<HttpResponse, Error> {
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
        if ext != "txt" && ext != "pdf" && ext != "md" {
            return Ok(HttpResponse::BadRequest().body("Only .txt/.pdf/.md allowed"));
        }

        let filepath = format!("{}/{}", UPLOAD_DIR, filename);
        let mut f = web::block(move || std::fs::File::create(&filepath)).await??;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            f = web::block(move || f.write_all(&data).map(|_| f)).await??;
        }

        uploaded_files.push(filename);
    }

    let mut indexed_files = Vec::new();
    let mut index_errors = Vec::new();
    if !uploaded_files.is_empty() {
        if is_reindex_in_progress() {
            index_errors.push(json!({
                "file": null,
                "error": "Reindex already in progress; automatic indexing skipped",
            }));
        } else if let Some(handle) = RETRIEVER.get() {
            match handle.lock() {
                Ok(mut retriever) => {
                    let chunker = crate::index::default_chunker(config.chunker_mode);
                    let chunker_ref = chunker.as_ref();
                    for filename in &uploaded_files {
                        let path = Path::new(UPLOAD_DIR).join(filename);
                        match index::index_file(
                            &mut *retriever,
                            &path,
                            config.chunker_mode,
                            chunker_ref,
                        ) {
                            Ok(chunks) => indexed_files.push(json!({
                                "file": filename,
                                "chunks_indexed": chunks,
                            })),
                            Err(err) => index_errors.push(json!({
                                "file": filename,
                                "error": err,
                            })),
                        }
                    }
                    if let Err(err) = retriever.commit() {
                        index_errors.push(json!({
                            "file": null,
                            "error": format!("commit failed: {}", err),
                        }));
                    }
                }
                Err(_) => {
                    index_errors.push(json!({
                        "file": null,
                        "error": "Failed to lock retriever for indexing",
                    }));
                }
            }
        } else {
            index_errors.push(json!({
                "file": null,
                "error": "Retriever not initialized; run /reindex manually",
            }));
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "uploaded_files": uploaded_files,
        "indexed_files": indexed_files,
        "index_errors": index_errors,
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

fn launch_async_reindex_job(config: web::Data<ApiConfig>) -> Result<String, (StatusCode, String)> {
    if REINDEX_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Reindex already in progress".to_string(),
        ));
    }

    let job_id = Uuid::new_v4().to_string();
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
    jobs.lock().unwrap().insert(job_id.clone(), job);

    let job_id_clone = job_id.clone();
    let jobs_map = jobs.clone();
    let retriever_handle = RETRIEVER.get().map(|h| Arc::clone(h));
    let config_clone = config.clone();

    actix_web::rt::spawn(async move {
        let start = std::time::Instant::now();
        let hooks = crate::monitoring::alerting_hooks::AlertingHooksConfig::from_env();
        if let Some(retriever) = retriever_handle {
            let mut retriever = retriever.lock().unwrap();
            {
                let mut job = jobs_map
                    .lock()
                    .unwrap()
                    .get(&job_id_clone)
                    .cloned()
                    .unwrap();
                job.status = "running".to_string();
                jobs_map.lock().unwrap().insert(job_id_clone.clone(), job);
            }

            let chunker = crate::index::default_chunker(config_clone.chunker_mode);
            let res = index::index_all_documents(
                &mut *retriever,
                UPLOAD_DIR,
                config_clone.chunker_mode,
                chunker.as_ref(),
            );

            let mut job = jobs_map
                .lock()
                .unwrap()
                .get(&job_id_clone)
                .cloned()
                .unwrap();
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
            jobs_map.lock().unwrap().insert(job_id_clone.clone(), job);
        } else {
            let mut job = jobs_map
                .lock()
                .unwrap()
                .get(&job_id_clone)
                .cloned()
                .unwrap();
            job.status = "failed".to_string();
            job.completed_at = Some(Utc::now().to_rfc3339());
            job.error = Some("Retriever not initialized".to_string());
            jobs_map
                .lock()
                .unwrap()
                .insert(job_id_clone.clone(), job.clone());
            let event = crate::monitoring::alerting_hooks::ReindexCompletionEvent::error(0, 0, 0);
            crate::monitoring::alerting_hooks::send_alert(&hooks, event).await;
        }
        REINDEX_IN_PROGRESS.store(false, Ordering::SeqCst);
    });

    Ok(job_id)
}

/// Phase 15: Async reindex endpoint
pub async fn reindex_async_handler(config: web::Data<ApiConfig>) -> Result<HttpResponse, Error> {
    let request_id = generate_request_id();

    match launch_async_reindex_job(config) {
        Ok(job_id) => Ok(HttpResponse::Accepted().json(json!({
            "status": "accepted",
            "job_id": job_id,
            "request_id": request_id
        }))),
        Err((status, message)) => Ok(HttpResponse::build(status).json(json!({
            "status": "busy",
            "message": message,
            "request_id": request_id
        }))),
    }
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

pub mod sys_routes;

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
    let api_config = config.clone();
    let mut http_server = HttpServer::new(move || {
        let api_config = api_config.clone();
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
            .app_data(web::Data::new(api_config.clone()))
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
            .route("/config/chunk_size", web::post().to(commit_chunk_config))
            .route("/config/llm", web::get().to(get_llm_config))
            .route("/config/llm", web::post().to(commit_llm_config))
            .route("/config/hardware", web::get().to(get_hardware_config))
            .route("/config/hardware", web::post().to(commit_hardware_config))
            .route("/config/api_keys", web::get().to(get_api_keys))
            .route("/config/api_keys", web::post().to(save_api_keys))
            .route(
                "/config/api_keys/{provider}",
                web::delete().to(delete_api_key),
            )
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
            .route(
                "/monitor/rate_limits/enabled",
                web::post().to(set_rate_limit_enabled),
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
            .service(web::scope("/sys").configure(sys_routes::sys_routes))
    });
    if force_single_worker {
        http_server = http_server.workers(1);
    }
    http_server
        .bind(bind_addr.clone())
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", bind_addr, e))
        .run()
}
