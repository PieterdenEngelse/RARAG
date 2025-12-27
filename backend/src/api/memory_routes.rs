// src/api/memory_routes.rs
// Phase 4: Memory API Layer - RAG Vector Store Endpoints

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::embedder::EmbeddingService;
use crate::memory::{VectorStore, VectorRecord};

// Shared state for vector store and embedding service
pub type SharedVectorStore = Arc<RwLock<VectorStore>>;
pub type SharedEmbeddingService = Arc<EmbeddingService>;

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct AddChunkRequest {
    pub chunk_id: String,
    pub document_id: String,
    pub content: String,
    pub chunk_index: usize,
    pub token_count: usize,
    pub source: String,
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
pub struct BatchAddRequest {
    pub chunks: Vec<AddChunkRequest>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub chunk_id: Option<String>,
    pub document_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
    pub total_found: usize,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub chunk_id: String,
    pub document_id: String,
    pub content: String,
    pub similarity_score: f32,
    pub chunk_index: usize,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_records: usize,
    pub total_documents: usize,
    pub db_path: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error: String,
}

fn default_top_k() -> usize {
    10
}

// ============ Handlers ============

/// Add a single chunk with embedding
pub async fn add_chunk(
    vector_store: web::Data<SharedVectorStore>,
    embedding_service: web::Data<SharedEmbeddingService>,
    req: web::Json<AddChunkRequest>,
) -> ActixResult<HttpResponse> {
    info!(chunk_id = %req.chunk_id, "Adding chunk to vector store");

    // Generate embedding if not provided
    let embedding = if let Some(emb) = &req.embedding {
        emb.clone()
    } else {
        embedding_service.embed_text(&req.content).await
    };

    let record = VectorRecord::new(
        req.chunk_id.clone(),
        req.document_id.clone(),
        req.content.clone(),
        embedding,
        req.chunk_index,
        req.token_count,
        req.source.clone(),
        chrono::Utc::now().timestamp(),
    );

    let mut store = vector_store.write().await;
    match store.add_record(record).await {
        Ok(()) => Ok(HttpResponse::Ok().json(MessageResponse {
            status: "success".to_string(),
            message: format!("Chunk {} added", req.chunk_id),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

/// Add multiple chunks in batch
pub async fn add_chunks_batch(
    vector_store: web::Data<SharedVectorStore>,
    embedding_service: web::Data<SharedEmbeddingService>,
    req: web::Json<BatchAddRequest>,
) -> ActixResult<HttpResponse> {
    info!(count = req.chunks.len(), "Adding batch of chunks");

    let mut records = Vec::new();

    for chunk_req in &req.chunks {
        // Generate embedding if not provided
        let embedding = if let Some(emb) = &chunk_req.embedding {
            emb.clone()
        } else {
            embedding_service.embed_text(&chunk_req.content).await
        };

        let record = VectorRecord::new(
            chunk_req.chunk_id.clone(),
            chunk_req.document_id.clone(),
            chunk_req.content.clone(),
            embedding,
            chunk_req.chunk_index,
            chunk_req.token_count,
            chunk_req.source.clone(),
            chrono::Utc::now().timestamp(),
        );

        records.push(record);
    }

    let mut store = vector_store.write().await;
    match store.add_records(records).await {
        Ok(()) => Ok(HttpResponse::Ok().json(MessageResponse {
            status: "success".to_string(),
            message: format!("Added {} chunks", req.chunks.len()),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

/// Search for similar chunks
pub async fn search_chunks(
    vector_store: web::Data<SharedVectorStore>,
    embedding_service: web::Data<SharedEmbeddingService>,
    req: web::Json<SearchRequest>,
) -> ActixResult<HttpResponse> {
    info!(query = %req.query, top_k = req.top_k, "Searching chunks");

    // Embed the query
    let query_embedding = embedding_service.embed_query(&req.query).await;

    // Search vector store - FIXED: use write() for mutable search
    let mut store = vector_store.write().await;
    match store.search(&query_embedding, req.top_k).await {
        Ok(results) => {
            let items: Vec<SearchResultItem> = results
                .into_iter()
                .map(|r| SearchResultItem {
                    chunk_id: r.chunk_id,
                    document_id: r.document_id,
                    content: r.content,
                    similarity_score: r.similarity_score,
                    chunk_index: r.chunk_index,
                })
                .collect();

            let total_found = items.len();

            Ok(HttpResponse::Ok().json(SearchResponse {
                results: items,
                total_found,
            }))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

/// Get chunks by document ID
pub async fn get_document_chunks(
    vector_store: web::Data<SharedVectorStore>,
    document_id: web::Path<String>,
) -> ActixResult<HttpResponse> {
    info!(document_id = %document_id, "Retrieving document chunks");

    // FIXED: use write() for mutable search_by_document
    let mut store = vector_store.write().await;
    match store.search_by_document(&document_id, 1000).await {
        Ok(records) => {
            let items: Vec<SearchResultItem> = records
                .into_iter()
                .map(|r| SearchResultItem {
                    chunk_id: r.chunk_id,
                    document_id: r.document_id,
                    content: r.content,
                    similarity_score: 1.0,
                    chunk_index: r.chunk_index,
                })
                .collect();

            let total_found = items.len();

            Ok(HttpResponse::Ok().json(SearchResponse {
                results: items,
                total_found,
            }))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

/// Delete a chunk or document
pub async fn delete_chunk(
    vector_store: web::Data<SharedVectorStore>,
    req: web::Json<DeleteRequest>,
) -> ActixResult<HttpResponse> {
    let mut store = vector_store.write().await;

    if let Some(chunk_id) = &req.chunk_id {
        info!(chunk_id = %chunk_id, "Deleting chunk");
        match store.delete_record(chunk_id).await {
            Ok(()) => Ok(HttpResponse::Ok().json(MessageResponse {
                status: "success".to_string(),
                message: format!("Deleted chunk {}", chunk_id),
            })),
            Err(e) => Ok(HttpResponse::NotFound().json(ErrorResponse {
                status: "error".to_string(),
                error: e.to_string(),
            })),
        }
    } else if let Some(document_id) = &req.document_id {
        info!(document_id = %document_id, "Deleting document");
        match store.delete_document(document_id).await {
            Ok(count) => Ok(HttpResponse::Ok().json(MessageResponse {
                status: "success".to_string(),
                message: format!("Deleted {} chunks from document {}", count, document_id),
            })),
            Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                status: "error".to_string(),
                error: e.to_string(),
            })),
        }
    } else {
        Ok(HttpResponse::BadRequest().json(ErrorResponse {
            status: "error".to_string(),
            error: "Must provide chunk_id or document_id".to_string(),
        }))
    }
}

/// Get vector store statistics
pub async fn get_stats(
    vector_store: web::Data<SharedVectorStore>,
) -> ActixResult<HttpResponse> {
    info!("Retrieving vector store stats");

    let store = vector_store.read().await;
    let stats = store.stats().await;

    Ok(HttpResponse::Ok().json(StatsResponse {
        total_records: stats.total_records,
        total_documents: stats.total_documents,
        db_path: stats.db_path.to_string_lossy().to_string(),
    }))
}

/// Clear all records (use with caution!)
pub async fn clear_store(
    vector_store: web::Data<SharedVectorStore>,
) -> ActixResult<HttpResponse> {
    info!("⚠️  Clearing vector store");

    let mut store = vector_store.write().await;
    match store.clear().await {
        Ok(()) => Ok(HttpResponse::Ok().json(MessageResponse {
            status: "success".to_string(),
            message: "Vector store cleared".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

/// Health check for memory service
pub async fn memory_health(
    vector_store: web::Data<SharedVectorStore>,
) -> ActixResult<HttpResponse> {
    let store = vector_store.read().await;
    let stats = store.stats().await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "total_records": stats.total_records,
        "total_documents": stats.total_documents,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}