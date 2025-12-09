use serde::{Deserialize, Serialize};

pub const API_BASE_URL: &str = "http://127.0.0.1:3010";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthResponse {
    pub status: String,
    pub documents: Option<usize>,
    pub vectors: Option<usize>,
    pub index_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub content: String,
    pub score: f32,
    pub document: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResponse {
    pub status: String,
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentsResponse {
    pub status: String,
    pub documents: Vec<String>,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestChartPoint {
    pub ts: i64,
    pub latency_ms: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct LatencyBreakdown {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct StatusBreakdown {
    pub success_rate: f64,
    pub client_error_rate: f64,
    pub server_error_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestsSnapshot {
    pub request_rate_rps: f64,
    pub latency_p95_ms: f64,
    pub error_rate_percent: f64,
    #[serde(default)]
    pub latency_breakdown: LatencyBreakdown,
    #[serde(default)]
    pub status_breakdown: StatusBreakdown,
    pub points: Vec<RequestChartPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RerankRequest {
    pub query: String,
    pub candidates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SummarizeRequest {
    pub query: String,
    pub candidates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexInfoResponse {
    pub index_in_ram: bool,
    pub mode: String,
    pub warning: Option<String>,
    pub total_documents: usize,
    pub total_vectors: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkingLoggingResponse {
    pub status: String,
    pub request_id: String,
    pub logging_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReindexAsyncResponse {
    pub status: String,
    pub job_id: String,
    pub request_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReindexStatusResponse {
    pub status: String,
    pub job_id: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub vectors_indexed: Option<usize>,
    pub mappings_indexed: Option<usize>,
    pub error: Option<String>,
    pub request_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheLayerStats {
    pub enabled: bool,
    pub total_searches: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheL2Stats {
    pub enabled: bool,
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub total_items: u64,
    pub hit_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheCountersSnapshot {
    pub hits_total: i64,
    pub misses_total: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisSummary {
    pub enabled: bool,
    pub connected: bool,
    pub ttl_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheInfoResponse {
    pub request_id: String,
    pub l1: CacheLayerStats,
    pub l2: CacheL2Stats,
    pub redis: RedisSummary,
    pub counters: CacheCountersSnapshot,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouteDropStat {
    pub route: String,
    pub drops: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimitConfigSnapshot {
    pub enabled: bool,
    pub trust_proxy: bool,
    pub search_qps: f64,
    pub search_burst: f64,
    pub upload_qps: f64,
    pub upload_burst: f64,
    pub exempt_prefixes: Vec<String>,
    pub rules: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimiterState {
    pub enabled: bool,
    pub active_keys: usize,
    pub capacity: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimitInfoResponse {
    pub request_id: String,
    pub total_drops: i64,
    pub drops_by_route: Vec<RouteDropStat>,
    pub config: RateLimitConfigSnapshot,
    pub limiter_state: RateLimiterState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub target: Option<String>,
    pub message: Option<String>,
    pub raw: String,
    pub fields: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogsResponse {
    pub request_id: String,
    pub file: Option<String>,
    pub entries: Vec<LogEntry>,
    pub note: Option<String>,
}

/// Check backend health
pub async fn health_check() -> Result<HealthResponse, String> {
    let url = format!("{}/health", API_BASE_URL);

    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Search documents
pub async fn search(query: &str) -> Result<SearchResponse, String> {
    let url = format!("{}/search?q={}", API_BASE_URL, urlencoding::encode(query));

    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// List all documents
pub async fn list_documents() -> Result<DocumentsResponse, String> {
    let url = format!("{}/documents", API_BASE_URL);

    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Delete a document
pub async fn delete_document(filename: &str) -> Result<serde_json::Value, String> {
    let url = format!("{}/documents/{}", API_BASE_URL, filename);

    gloo_net::http::Request::delete(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Trigger reindexing
pub async fn reindex() -> Result<serde_json::Value, String> {
    let url = format!("{}/reindex", API_BASE_URL);

    gloo_net::http::Request::post(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

pub async fn reindex_async() -> Result<ReindexAsyncResponse, String> {
    let url = format!("{}/reindex/async", API_BASE_URL);

    gloo_net::http::Request::post(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

pub async fn fetch_reindex_status(job_id: &str) -> Result<ReindexStatusResponse, String> {
    let url = format!("{}/reindex/status/{}", API_BASE_URL, job_id);

    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Fetch request metrics snapshot for the Monitor UI
pub async fn fetch_requests_snapshot() -> Result<RequestsSnapshot, String> {
    let url = format!("{}/monitoring/ui/requests", API_BASE_URL);

    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Fetch index info for the monitor page
pub async fn fetch_index_info() -> Result<IndexInfoResponse, String> {
    fetch_json::<IndexInfoResponse>("/index/info").await
}

pub async fn get_chunking_logging() -> Result<ChunkingLoggingResponse, String> {
    fetch_json::<ChunkingLoggingResponse>("/monitoring/chunking/logging").await
}

pub async fn set_chunking_logging(enabled: bool) -> Result<ChunkingLoggingResponse, String> {
    let url = format!("/monitoring/chunking/logging?enabled={}", enabled);
    fetch_json::<ChunkingLoggingResponse>(&url).await
}

pub async fn fetch_cache_info() -> Result<CacheInfoResponse, String> {
    let url = format!("{}/monitor/cache/info", API_BASE_URL);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.status() == 204 {
        return Err("Backend returned 204 No Content for cache info".into());
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

pub async fn fetch_rate_limit_info() -> Result<RateLimitInfoResponse, String> {
    let url = format!("{}/monitor/rate_limits/info", API_BASE_URL);
    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

pub async fn fetch_recent_logs(limit: usize) -> Result<LogsResponse, String> {
    let url = format!("{}/monitor/logs/recent?limit={}", API_BASE_URL, limit);
    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

async fn fetch_json<T>(path: &str) -> Result<T, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let url = format!("{}{}", API_BASE_URL, path);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if !(200..=299).contains(&status) {
        let body = match response.text().await {
            Ok(body) => body.trim().to_string(),
            Err(_) => String::new(),
        };
        let detail = if body.is_empty() {
            "(empty response)".to_string()
        } else {
            body
        };
        return Err(format!("HTTP {} {}", status, detail));
    }

    response
        .json::<T>()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}
