use crate::config::ChunkerMode;
use crate::embedder;
use crate::memory::chunker::ChunkerConfig;
use crate::memory::chunker_factory::{create_chunker, Chunker};
use crate::retriever::Retriever;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

pub fn index_all_documents(
    retriever: &mut Retriever,
    folder: &str,
    chunker_mode: ChunkerMode,
    chunker: &dyn Chunker,
) -> Result<(), String> {
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
                match index_file(retriever, &path, chunker_mode, chunker) {
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

pub fn index_file(
    retriever: &mut Retriever,
    path: &Path,
    chunker_mode: ChunkerMode,
    chunker: &dyn Chunker,
) -> Result<usize, String> {
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

    let chunk_start = std::time::Instant::now();
    let chunks = chunker.chunk_text(&content);
    let chunk_duration = chunk_start.elapsed();
    let mut ok = 0usize;
    let mut total_tokens = 0usize;
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_id = format!("{}#{}", filename, i);
        let vector = embedder::embed(chunk);

        total_tokens += chunk.split_whitespace().count();

        // Handle the Result from index_chunk
        if let Err(e) = retriever.index_chunk(&chunk_id, chunk, &vector) {
            warn!("index_file: Failed to index chunk {}: {}", chunk_id, e);
        } else {
            ok += 1;
        }
    }

    if let Some(stats) = chunker.stats() {
        info!(
            "index_file: file='{}' mode={:?} chunks={} tokens={} duration_ms={} semantic_threshold={} semantic_flushes={} heading_flushes={} size_flushes={} total_segments={} avg_similarity={:?}",
            filename,
            chunker_mode,
            ok,
            total_tokens,
            chunk_duration.as_millis(),
            stats.semantic_similarity_threshold,
            stats.semantic_flushes,
            stats.heading_flushes,
            stats.size_flushes,
            stats.total_segments,
            stats.average_similarity(),
        );
        crate::monitoring::record_chunking_snapshot(crate::monitoring::ChunkingStatsSnapshot::new(
            filename,
            chunker_mode,
            ok,
            total_tokens,
            chunk_duration.as_millis() as u64,
            Some(stats),
        ));
    } else {
        info!(
            "index_file: file='{}' mode={:?} chunks={} tokens={} duration_ms={}",
            filename,
            chunker_mode,
            ok,
            total_tokens,
            chunk_duration.as_millis()
        );
        crate::monitoring::record_chunking_snapshot(crate::monitoring::ChunkingStatsSnapshot::new(
            filename,
            chunker_mode,
            ok,
            total_tokens,
            chunk_duration.as_millis() as u64,
            None,
        ));
    }
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

pub fn default_chunker(mode: ChunkerMode) -> Box<dyn Chunker> {
    let config = ChunkerConfig::from_env();
    create_chunker(mode.into(), &config)
}
