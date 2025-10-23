// src/api/upload_routes.rs or similar

use actix_web::{post, web, HttpResponse};
use crate::memory::chunker::{SemanticChunker, SourceType};

#[post("/api/documents/upload")]
async fn upload_document(
    payload: web::Bytes,
    filename: web::Query<String>,
) -> HttpResponse {
    let content = String::from_utf8(payload.to_vec()).unwrap();
    
    // Determine source type from filename
    let source_type = match filename.extension() {
        Some("pdf") => SourceType::Pdf,
        Some("md") => SourceType::Markdown,
        Some("html") => SourceType::Html,
        _ => SourceType::Text,
    };
    
    let chunker = SemanticChunker::with_default();
    let chunks = chunker.chunk_document(
        &content,
        Uuid::new_v4().to_string(),
        filename.to_string(),
        source_type,
    );
    
    // Store chunks...
    
    HttpResponse::Ok().json(json!({ "chunks_created": chunks.len() }))
}