use crate::embedder;
use crate::retriever::Retriever;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

pub fn index_all_documents(retriever: &mut Retriever, folder: &str) -> Result<(), String> {
    debug!("index_all_documents: scanning folder='{}'", folder);
    let entries =
        fs::read_dir(folder).map_err(|e| format!("read_dir('{}') failed: {}", folder, e))?;

    for entry_res in entries {
        let entry = match entry_res {
            Ok(e) => e,
            Err(e) => {
                warn!("index_all_documents: failed to read directory entry: {}", e);
                continue;
            }
        };
        let path = entry.path();
        let path_str = path.to_string_lossy();
        if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str());
            debug!(
                "index_all_documents: considering file='{}' ext={:?}",
                path_str, ext
            );
            if matches!(ext, Some("txt") | Some("pdf")) {
                match index_file(retriever, &path) {
                    Ok(chunks) => debug!("indexed file='{}' chunks={}", path_str, chunks),
                    Err(e) => warn!("index_file failed for '{}': {}", path_str, e),
                }
            } else {
                debug!(
                    "index_all_documents: skipping unsupported file='{}'",
                    path_str
                );
            }
        } else {
            debug!("index_all_documents: skipping non-file path='{}'", path_str);
        }
    }

    // Commit retriever state (vectors live write suppressed during reindex)
    retriever
        .commit()
        .map_err(|e| format!("commit failed: {}", e))
}

pub fn index_file(retriever: &mut Retriever, path: &Path) -> Result<usize, String> {
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    debug!("index_file: start file='{}'", path.to_string_lossy());
    let content = match extract_text(path) {
        Some(text) => text,
        None => {
            warn!("index_file: extract_text returned None for '{}'", filename);
            return Err("extract_text failed".into());
        }
    };

    let chunks = chunk_text(&content);
    let mut ok = 0usize;
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_id = format!("{}#{}", filename, i);
        let vector = embedder::embed(chunk);

        // Handle the Result from index_chunk
        if let Err(e) = retriever.index_chunk(&chunk_id, chunk, &vector) {
            warn!("index_file: Failed to index chunk {}: {}", chunk_id, e);
        } else {
            ok += 1;
        }
    }
    debug!("index_file: done file='{}' chunks_indexed={}", filename, ok);
    Ok(ok)
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
