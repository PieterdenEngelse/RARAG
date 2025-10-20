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
