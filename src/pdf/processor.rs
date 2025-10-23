// src/pdf/processor.rs or similar

use crate::memory::chunker::{SemanticChunker, SourceType};

pub async fn process_pdf(pdf_path: &Path) -> Result<(), Error> {
    // Your existing PDF text extraction
    let content = extract_text_from_pdf(pdf_path)?;
    
    // NEW: Chunk the extracted text
    let chunker = SemanticChunker::with_default();
    let chunks = chunker.chunk_document(
        &content,
        generate_document_id(), // or use existing doc ID
        pdf_path.to_string_lossy().to_string(),
        SourceType::Pdf,
    );
    
    // Store chunks (we'll implement this in Phase 3)
    for chunk in chunks {
        store_chunk_to_db(&chunk).await?;
        // Phase 2: generate_and_store_embedding(&chunk).await?;
    }
    
    Ok(())
}