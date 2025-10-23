use std::fs;
use std::path::Path;
use crate::retriever::Retriever;
use crate::embedder;

pub fn index_all_documents(retriever: &mut Retriever, folder: &str) {
    let entries = match fs::read_dir(folder) {
        Ok(e) => e,
        Err(_) => return,
    };
    
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str());
            if ext == Some("txt") || ext == Some("pdf") {
                index_file(retriever, &path);
            }
        }
    }
    
    // Handle the Result from commit
    if let Err(e) = retriever.commit() {
        eprintln!("Warning: Failed to commit index: {}", e);
    }
}

pub fn index_file(retriever: &mut Retriever, path: &Path) {
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
    let content = match extract_text(path) {
        Some(text) => text,
        None => return,
    };
    
    let chunks = chunk_text(&content);
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_id = format!("{}#{}", filename, i);
        let vector = embedder::embed(chunk);
        
        // Handle the Result from index_chunk
        if let Err(e) = retriever.index_chunk(&chunk_id, chunk, &vector) {
            eprintln!("Warning: Failed to index chunk {}: {}", chunk_id, e);
        }
    }
}

fn extract_text(path: &Path) -> Option<String> {
    let ext = path.extension().and_then(|s| s.to_str())?;
    match ext {
        "txt" => fs::read_to_string(path).ok(),
        "pdf" => Some("PDF parsing not implemented.".to_string()),
        _ => None,
    }
}

fn chunk_text(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}