// src/api/mod.rs - Complete with Phase 4 Memory API

use actix_web::{web, App, HttpResponse, HttpServer, Error};
use actix_cors::Cors;
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use log::error;
use chrono::Utc;

use crate::retriever::Retriever;
use crate::index;
use crate::agent::{Agent, AgentResponse};
use crate::agent_memory::{AgentMemory, MemoryItem, MemorySearchResult};
use crate::config::ApiConfig;
use crate::embedder::EmbeddingService;
use crate::memory::VectorStore;

pub mod memory_routes;

pub const UPLOAD_DIR: &str = "documents";

// Global retriever handle
static RETRIEVER: OnceLock<Arc<Mutex<Retriever>>> = OnceLock::new();

pub fn set_retriever_handle(handle: Arc<Mutex<Retriever>>) {
    let _ = RETRIEVER.set(handle);
}

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

#[derive(serde::Deserialize)]
pub struct StoreRagRequest {
    pub agent_id: String,
    pub memory_type: String,
    pub content: String,
}

#[derive(serde::Deserialize)]
pub struct SearchRagRequest {
    pub agent_id: String,
    pub query: String,
    pub top_k: usize,
}

#[derive(serde::Deserialize)]
pub struct RecallRagRequest {
    pub agent_id: String,
    pub limit: usize,
}

#[derive(serde::Deserialize)]
pub struct AgentRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    10
}

// ============ Health & Status Handlers ============

pub async fn health_check() -> Result<HttpResponse, Error> {
    if let Some(retriever) = RETRIEVER.get() {
        let retriever = retriever.lock().unwrap();
        match retriever.health_check() {
            Ok(()) => {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "status": "healthy",
                    "documents": retriever.metrics.total_documents_indexed,
                    "vectors": retriever.metrics.total_vectors,
                    "index_path": retriever.metrics.index_path
                })))
            }
            Err(e) => {
                error!("Health check failed: {}", e);
                Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "status": "unhealthy",
                    "error": e.to_string()
                })))
            }
        }
    } else {
        error!("Health check failed: Retriever not initialized");
        Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "error": "Retriever not initialized"
        })))
    }
}

async fn root_handler() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("✅ Backend is running (Actix Web)\n\nTry /health or /ready\n"))
}

async fn ready_check() -> Result<HttpResponse, Error> {
    if let Some(retriever) = RETRIEVER.get() {
        match retriever.lock() {
            Ok(retriever) => {
                match retriever.ready_check() {
                    Ok(_) => {
                        Ok(HttpResponse::Ok().json(serde_json::json!({
                            "status": "ready",
                            "timestamp": Utc::now().to_rfc3339()
                        })))
                    }
                    Err(e) => {
                        Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                            "status": "not ready",
                            "error": e.to_string(),
                            "timestamp": Utc::now().to_rfc3339()
                        })))
                    }
                }
            }
            Err(e) => {
                Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "status": "not ready",
                    "error": format!("Failed to acquire lock: {}", e),
                    "timestamp": Utc::now().to_rfc3339()
                })))
            }
        }
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not ready",
            "message": "Retriever not initialized",
            "timestamp": Utc::now().to_rfc3339()
        })))
    }
}

async fn get_metrics() -> Result<HttpResponse, Error> {
    if let Some(retriever) = RETRIEVER.get() {
        match retriever.lock() {
            Ok(retriever) => {
                let metrics = retriever.get_metrics();
                Ok(HttpResponse::Ok().json(metrics))
            }
            Err(e) => {
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "error",
                    "error": format!("Failed to acquire lock: {}", e)
                })))
            }
        }
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "error",
            "message": "Retriever not initialized"
        })))
    }
}

// ============ Document Upload & Management ============

pub async fn upload_document(mut payload: Multipart) -> Result<HttpResponse, Error> {
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

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "uploaded_files": uploaded_files,
        "message": "Use /reindex to refresh index"
    })))
}

pub async fn list_documents() -> Result<HttpResponse, Error> {
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
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "documents": files,
        "count": files.len()
    })))
}

pub async fn delete_document(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let filename = path.into_inner();
    let filepath = format!("{}/{}", UPLOAD_DIR, filename);
    match fs::remove_file(&filepath) {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": format!("Deleted {}", filename)
        }))),
        Err(_) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "status": "error",
            "message": "File not found"
        })))
    }
}

pub async fn reindex_handler() -> Result<HttpResponse, Error> {
    if let Some(retriever) = RETRIEVER.get() {
        let mut retriever = retriever.lock().unwrap();
        index::index_all_documents(&mut *retriever, UPLOAD_DIR);
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Reindexing complete"
        })))
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Retriever not initialized"
        })))
    }
}

// ============ Search & Retrieval ============

pub async fn search_documents(query: web::Query<SearchQuery>) -> Result<HttpResponse, Error> {
    if let Some(retriever) = RETRIEVER.get() {
        let mut retriever = retriever.lock().unwrap();
        let results = retriever.search(&query.q).unwrap_or_default();
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "results": results
        })))
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Retriever not initialized"
        })))
    }
}

pub async fn rerank(request: web::Json<RerankRequest>) -> Result<HttpResponse, Error> {
    if let Some(retriever) = RETRIEVER.get() {
        let retriever = retriever.lock().unwrap();
        let reranked = retriever.rerank_by_similarity(&request.query, &request.candidates);
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "results": reranked
        })))
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Retriever not initialized"
        })))
    }
}

pub async fn summarize(request: web::Json<SummarizeRequest>) -> Result<HttpResponse, Error> {
    if let Some(retriever) = RETRIEVER.get() {
        let retriever = retriever.lock().unwrap();
        let summary = retriever.summarize_chunks(&request.query, &request.candidates);
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "summary": summary
        })))
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Retriever not initialized"
        })))
    }
}

pub async fn save_vectors_handler() -> Result<HttpResponse, Error> {
    if let Some(retriever) = RETRIEVER.get() {
        let mut retriever = retriever.lock().unwrap();
        match retriever.force_save() {
            Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "Vectors saved successfully",
                "vector_count": retriever.vectors.len()
            }))),
            Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to save vectors: {}", e)
            })))
        }
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Retriever not initialized"
        })))
    }
}

// ============ Legacy RAG Memory (SQLite) ============

pub async fn store_rag_memory(req: web::Json<StoreRagRequest>) -> Result<HttpResponse, Error> {
    let mem = AgentMemory::new("agent.db")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let ts = Utc::now().to_rfc3339();
    mem.store_rag(&req.agent_id, &req.memory_type, &req.content, &ts)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Memory stored"
    })))
}

pub async fn search_rag_memory(req: web::Json<SearchRagRequest>) -> Result<HttpResponse, Error> {
    let mem = AgentMemory::new("agent.db")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let results: Vec<MemorySearchResult> = mem.search_rag(&req.agent_id, &req.query, req.top_k)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(results))
}

pub async fn recall_rag_memory(req: web::Json<RecallRagRequest>) -> Result<HttpResponse, Error> {
    let mem = AgentMemory::new("agent.db")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    let items: Vec<MemoryItem> = mem.recall_rag(&req.agent_id, req.limit)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(items))
}

// ============ Agent ============

pub async fn run_agent(req: web::Json<AgentRequest>) -> Result<HttpResponse, Error> {
    if let Some(retriever) = RETRIEVER.get() {
        let agent = Agent::new("default", "agent.db", Arc::clone(retriever));
        let resp: AgentResponse = agent.run(&req.query, req.top_k);
        Ok(HttpResponse::Ok().json(resp))
    } else {
        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": "Retriever not initialized"
        })))
    }
}

// ============ API Server ============

pub fn start_api_server(config: &ApiConfig) -> impl std::future::Future<Output = std::io::Result<()>> {
    // Initialize Phase 4 services
    let embedding_service = EmbeddingService::new(crate::embedder::EmbeddingConfig::default());
    let embedding_service = Arc::new(embedding_service);

    let vector_store = VectorStore::with_defaults()
        .expect("Failed to initialize vector store");
    let vector_store = Arc::new(tokio::sync::RwLock::new(vector_store));

    let config_bind = config.bind_addr().to_string();

    HttpServer::new(move || {
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
            .app_data(web::Data::new(embedding_service.clone()))
            .app_data(web::Data::new(vector_store.clone()))
            // Core endpoints
            .route("/", web::get().to(root_handler))
            .route("/health", web::get().to(health_check))
            .route("/ready", web::get().to(ready_check))
            .route("/metrics", web::get().to(get_metrics))
            // Document management
            .route("/upload", web::post().to(upload_document))
            .route("/documents", web::get().to(list_documents))
            .route("/documents/{filename}", web::delete().to(delete_document))
            .route("/reindex", web::post().to(reindex_handler))
            // Search & retrieval
            .route("/search", web::get().to(search_documents))
            .route("/rerank", web::post().to(rerank))
            .route("/summarize", web::post().to(summarize))
            .route("/save_vectors", web::post().to(save_vectors_handler))
            // Legacy RAG memory (SQLite)
            .route("/memory/store_rag", web::post().to(store_rag_memory))
            .route("/memory/search_rag", web::post().to(search_rag_memory))
            .route("/memory/recall_rag", web::post().to(recall_rag_memory))
            // Phase 4: New Memory API (Vector Store)
            .route("/api/memory/health", web::get().to(memory_routes::memory_health))
            .route("/api/memory/add", web::post().to(memory_routes::add_chunk))
            .route("/api/memory/batch", web::post().to(memory_routes::add_chunks_batch))
            .route("/api/memory/search", web::post().to(memory_routes::search_chunks))
            .route("/api/memory/document/{document_id}", web::get().to(memory_routes::get_document_chunks))
            .route("/api/memory/delete", web::delete().to(memory_routes::delete_chunk))
            .route("/api/memory/stats", web::get().to(memory_routes::get_stats))
            .route("/api/memory/clear", web::post().to(memory_routes::clear_store))
            // Agent
            .route("/agent", web::post().to(run_agent))
    })
    .bind(&config_bind)
    .unwrap_or_else(|_| panic!("Failed to bind to {}", config_bind))
    .run()
}