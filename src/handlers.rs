// src/handlers.rs
use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use crate::retriever::{Retriever, RetrieverError, RetrieverMetrics};
use crate::cache::cache_layer::CacheStats;

type SharedRetriever = Arc<RwLock<Retriever>>;

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
}

#[derive(Deserialize)]
pub struct VectorSearchRequest {
    pub vector: Vec<f32>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

#[derive(Deserialize)]
pub struct DocumentRequest {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChunkRequest {
    pub chunk_id: String,
    pub text: String,
    pub vector: Vec<f32>,
}

#[derive(Deserialize)]
pub struct BatchDocumentsRequest {
    pub documents: Vec<DocumentRequest>,
}

fn default_top_k() -> usize {
    10
}

// === Handlers ===

pub async fn get_metrics(data: web::Data<SharedRetriever>) -> ActixResult<HttpResponse> {
    match data.read() {
        Ok(retriever) => Ok(HttpResponse::Ok().json(retriever.get_metrics())),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

pub async fn health_check(data: web::Data<SharedRetriever>) -> ActixResult<HttpResponse> {
    match data.read() {
        Ok(retriever) => match retriever.health_check() {
            Ok(()) => Ok(HttpResponse::Ok().json("OK")),
            Err(e) => Ok(HttpResponse::ServiceUnavailable().json(format!("{}", e))),
        },
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

pub async fn ready_check(data: web::Data<SharedRetriever>) -> ActixResult<HttpResponse> {
    match data.read() {
        Ok(retriever) => match retriever.ready_check() {
            Ok(()) => Ok(HttpResponse::Ok().json("Ready")),
            Err(e) => Ok(HttpResponse::ServiceUnavailable().json(format!("{}", e))),
        },
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

pub async fn search(
    data: web::Data<SharedRetriever>,
    req: web::Json<SearchRequest>,
) -> ActixResult<HttpResponse> {
    match data.write() {
        Ok(mut retriever) => match retriever.search(&req.query) {
            Ok(results) => Ok(HttpResponse::Ok().json(results)),
            Err(e) => Ok(HttpResponse::BadRequest().json(format!("{}", e))),
        },
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

pub async fn vector_search(
    data: web::Data<SharedRetriever>,
    req: web::Json<VectorSearchRequest>,
) -> ActixResult<HttpResponse> {
    if req.vector.is_empty() {
        return Ok(HttpResponse::BadRequest().json("Vector must not be empty"));
    }
    match data.read() {
        Ok(retriever) => {
            let results = retriever.vector_search(&req.vector, req.top_k);
            // Map indices to document content if possible
            let contents: Vec<String> = results
                .into_iter()
                .filter_map(|(idx, _score)| retriever.get_content_by_vector_idx(idx))
                .collect();
            Ok(HttpResponse::Ok().json(contents))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

pub async fn hybrid_search(
    data: web::Data<SharedRetriever>,
    req: web::Json<SearchRequest>,
) -> ActixResult<HttpResponse> {
    // For simplicity, this version uses keyword-only.
    // To support vector, extend the request struct.
    match data.write() {
        Ok(mut retriever) => match retriever.hybrid_search(&req.query, None) {
            Ok(results) => Ok(HttpResponse::Ok().json(results)),
            Err(e) => Ok(HttpResponse::BadRequest().json(format!("{}", e))),
        },
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

pub async fn add_document(
    data: web::Data<SharedRetriever>,
    req: web::Json<DocumentRequest>,
) -> ActixResult<HttpResponse> {
    match data.write() {
        Ok(mut retriever) => match retriever.add_document(&req.title, &req.content) {
            Ok(()) => Ok(HttpResponse::Ok().json("Document added")),
            Err(e) => Ok(HttpResponse::InternalServerError().json(format!("{}", e))),
        },
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

pub async fn add_chunk(
    data: web::Data<SharedRetriever>,
    req: web::Json<ChunkRequest>,
) -> ActixResult<HttpResponse> {
    if req.vector.is_empty() {
        return Ok(HttpResponse::BadRequest().json("Vector must not be empty"));
    }
    match data.write() {
        Ok(mut retriever) => match retriever.index_chunk(&req.chunk_id, &req.text, &req.vector) {
            Ok(()) => Ok(HttpResponse::Ok().json("Chunk indexed")),
            Err(e) => Ok(HttpResponse::InternalServerError().json(format!("{}", e))),
        },
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

pub async fn add_batch(
    data: web::Data<SharedRetriever>,
    req: web::Json<BatchDocumentsRequest>,
) -> ActixResult<HttpResponse> {
    let docs: Vec<(String, String)> = req
        .documents
        .iter()
        .map(|d| (d.title.clone(), d.content.clone()))
        .collect();

    match data.write() {
        Ok(mut retriever) => match retriever.add_documents_batch(docs) {
            Ok(count) => Ok(HttpResponse::Ok().json(format!("Added {} documents", count))),
            Err(e) => Ok(HttpResponse::InternalServerError().json(format!("{}", e))),
        },
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

pub async fn clear_cache(data: web::Data<SharedRetriever>) -> ActixResult<HttpResponse> {
    match data.write() {
        Ok(mut retriever) => {
            retriever.clear_cache();
            Ok(HttpResponse::Ok().json("Cache cleared"))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

pub async fn force_save(data: web::Data<SharedRetriever>) -> ActixResult<HttpResponse> {
    match data.read() {
        Ok(retriever) => match retriever.force_save() {
            Ok(()) => Ok(HttpResponse::Ok().json("Vectors saved")),
            Err(e) => Ok(HttpResponse::InternalServerError().json(format!("{}", e))),
        },
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

// === Phase 11 Step 3: L2 Cache Handlers - Version 1.0.0 ===

/// Get L2 cache statistics
/// GET /cache/stats
pub async fn get_l2_cache_stats(data: web::Data<SharedRetriever>) -> ActixResult<HttpResponse> {
    match data.read() {
        Ok(retriever) => {
            let stats = retriever.get_l2_cache_stats();
            Ok(HttpResponse::Ok().json(stats))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

/// Clear L2 cache
/// POST /cache/clear-l2
pub async fn clear_l2_cache(data: web::Data<SharedRetriever>) -> ActixResult<HttpResponse> {
    match data.write() {
        Ok(mut retriever) => {
            retriever.clear_l2_cache();
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "L2 cache cleared"
            })))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}

/// Log cache statistics to console
/// POST /cache/log
pub async fn log_l2_cache_stats(data: web::Data<SharedRetriever>) -> ActixResult<HttpResponse> {
    match data.read() {
        Ok(retriever) => {
            retriever.log_cache_stats();
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "Cache stats logged to console"
            })))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Lock poisoned")),
    }
}