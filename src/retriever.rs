use tantivy::{
    Index, 
    collector::TopDocs, 
    query::QueryParser,
    schema::{Schema, TEXT, STORED, Field, Value},
    directory::MmapDirectory,
    TantivyError,
    query::QueryParserError,
    directory::error::OpenDirectoryError,
    IndexWriter,
};
use serde::{Serialize, Deserialize};
use std::fmt;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Write, Read};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::prelude::*;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::time::{Instant, Duration};
use std::path::Path;
use tracing::{info, error};
use std::fs; 
use fs2;

/// Custom error type for Retriever operations
#[derive(Debug, Serialize, Deserialize)]
pub enum RetrieverError {
    TantivyError(String),
    IoError(String),
    IndexError(String),
    VectorError(String),
    QueryParserError(String),
    DirectoryError(String),
    SerializationError(String),
}

impl fmt::Display for RetrieverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RetrieverError::TantivyError(e) => write!(f, "Tantivy error: {}", e),
            RetrieverError::IoError(e) => write!(f, "IO error: {}", e),
            RetrieverError::IndexError(msg) => write!(f, "Index error: {}", msg),
            RetrieverError::VectorError(msg) => write!(f, "Vector error: {}", msg),
            RetrieverError::QueryParserError(msg) => write!(f, "Query parser error: {}", msg),
            RetrieverError::DirectoryError(msg) => write!(f, "Directory error: {}", msg),
            RetrieverError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for RetrieverError {}

impl From<TantivyError> for RetrieverError {
    fn from(err: TantivyError) -> Self {
        RetrieverError::TantivyError(err.to_string())
    }
}

impl From<std::io::Error> for RetrieverError {
    fn from(err: std::io::Error) -> Self {
        RetrieverError::IoError(err.to_string())
    }
}

impl From<QueryParserError> for RetrieverError {
    fn from(err: QueryParserError) -> Self {
        RetrieverError::QueryParserError(err.to_string())
    }
}

impl From<OpenDirectoryError> for RetrieverError {
    fn from(err: OpenDirectoryError) -> Self {
        RetrieverError::DirectoryError(err.to_string())
    }
}

#[derive(Serialize, Deserialize)]
struct VectorStorage {
    vectors: Vec<Vec<f32>>,
    doc_id_to_vector_idx: HashMap<String, usize>,
}

/// Metrics for monitoring Retriever performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieverMetrics {
    pub total_searches: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub avg_search_latency_us: f64,
    pub total_search_latency_us: u128,
    pub max_search_latency_us: u128,
    pub total_documents_indexed: usize,
    pub total_vectors: usize,
    pub index_path: String,
    pub last_updated: u64,
}

impl Default for RetrieverMetrics {
    fn default() -> Self {
        Self {
            total_searches: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_search_latency_us: 0.0,
            total_search_latency_us: 0,
            max_search_latency_us: 0,
            total_documents_indexed: 0,
            total_vectors: 0,
            index_path: String::new(),
            last_updated: 0,
        }
    }
}

impl RetrieverMetrics {
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_searches == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_searches as f64
        }
    }

    pub fn get_index_size_bytes(&self) -> Result<u64, std::io::Error> {
        let path = Path::new(&self.index_path);
        if !path.exists() {
            return Ok(0);
        }
        let mut total_size = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total_size += metadata.len();
            }
        }
        Ok(total_size)
    }

    pub fn get_index_size_human(&self) -> Result<String, std::io::Error> {
        let size = self.get_index_size_bytes()?;
        let sizes = ["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut i = 0;
        while size >= 1024.0 && i < sizes.len() - 1 {
            size /= 1024.0;
            i += 1;
        }
        Ok(format!("{:.2} {}", size, sizes[i]))
    }
}

pub struct Retriever {
    pub vectors: Vec<Vec<f32>>,
    pub index: Index,
    pub title_field: Field,
    pub content_field: Field,
    pub doc_id_field: Field,
    pub doc_id_to_vector_idx: HashMap<String, usize>,
    pub vector_file_path: String,
    pub auto_save_threshold: usize,
    documents_since_save: Arc<AtomicUsize>,
    index_writer: Option<IndexWriter>,
    batch_mode: bool,
    search_cache: LruCache<String, Vec<String>>,
    cache_enabled: bool,
    pub metrics: RetrieverMetrics,
    index_dir_path: String,
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        0.0
    } else {
        dot_product / (magnitude_a * magnitude_b)
    }
}

impl Retriever {
    /// Create a new Retriever with custom vector storage path
    pub fn new_with_vector_file(index_dir: &str, vector_file_path: &str) -> Result<Self, RetrieverError> {
        let mut schema_builder = Schema::builder();
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let doc_id_field = schema_builder.add_text_field("doc_id", TEXT | STORED); 
        let schema = schema_builder.build();
        fs::create_dir_all(index_dir)?;
        let dir = MmapDirectory::open(index_dir)?;
        let index = Index::open_or_create(dir, schema)?;
        
        let vector_file_path_owned = vector_file_path.to_string();
        
        let mut retriever = Retriever {
            vectors: Vec::new(),
            index,
            title_field,
            content_field,
            doc_id_field,
            doc_id_to_vector_idx: HashMap::new(),
            vector_file_path: vector_file_path_owned.clone(),
            auto_save_threshold: 100,
            documents_since_save: Arc::new(AtomicUsize::new(0)),
            index_writer: None,
            batch_mode: false,
            search_cache: LruCache::new(NonZeroUsize::new(100).unwrap()),
            cache_enabled: true,
            metrics: RetrieverMetrics {
                index_path: index_dir.to_string(),
                ..Default::default()
            },
            index_dir_path: index_dir.to_string(),
        };
        
        // Now load from the CORRECT path - clone the path to avoid borrow issues
        if let Err(e) = retriever.load_vectors(&vector_file_path_owned) {
            info!("No existing vectors found at '{}', starting fresh: {}", vector_file_path_owned, e);
        } else {
            info!("Loaded existing vectors from {}", vector_file_path_owned);
            retriever.metrics.total_vectors = retriever.vectors.len();
        }
        
        if let Ok(reader) = retriever.index.reader() {
            retriever.metrics.total_documents_indexed = reader.searcher().num_docs() as usize;
        }
        
        Ok(retriever)
    }

    /// Create a new Retriever with default vector storage path ("./vectors.json")
    pub fn new(index_dir: &str) -> Result<Self, RetrieverError> {
        Self::new_with_vector_file(index_dir, "./vectors.json")
    }

    pub fn new_dummy() -> Result<Self, RetrieverError> {
        // Create a unique dummy path to avoid conflicts
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dummy_dir = format!("./dummy_tantivy_index_{}", timestamp);
        let dummy_vector_file = format!("./dummy_vectors_{}.json", timestamp);
        let result = Self::new_with_vector_file(&dummy_dir, &dummy_vector_file);
        
        // Clean up the dummy files immediately if creation failed
        if result.is_err() {
            let _ = fs::remove_dir_all(&dummy_dir);
            let _ = fs::remove_file(&dummy_vector_file);
        }
        
        result
    }

    /// Repair vector mappings by adding default IDs for unmapped vectors
    pub fn repair_vector_mappings(&mut self) -> usize {
        let mapped_indices: HashSet<usize> = 
            self.doc_id_to_vector_idx.values().cloned().collect();
        
        let mut repaired = 0;
        for idx in 0..self.vectors.len() {
            if !mapped_indices.contains(&idx) {
                let default_id = format!("unmapped_vector_{}", idx);
                self.doc_id_to_vector_idx.insert(default_id, idx);
                repaired += 1;
            }
        }
        
        if repaired > 0 {
            info!("Repaired {} unmapped vectors", repaired);
            if let Err(e) = self.save_vectors(&self.vector_file_path.clone()) {
                error!("Failed to save repaired mappings: {}", e);
            }
        }
        
        repaired
    }

    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
        if !enabled {
            self.search_cache.clear();
        }
        info!("Search cache {}", if enabled { "enabled" } else { "disabled" });
    }

    pub fn clear_cache(&mut self) {
        self.search_cache.clear();
        info!("Search cache cleared");
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        (self.search_cache.len(), self.search_cache.cap().get())
    }

    pub fn get_metrics(&self) -> RetrieverMetrics {
        self.metrics.clone()
    }

    pub fn reset_metrics(&mut self) {
        self.metrics = RetrieverMetrics {
            index_path: self.index_dir_path.clone(),
            ..Default::default()
        };
    }

    pub fn begin_batch(&mut self) -> Result<(), RetrieverError> {
        if self.index_writer.is_some() {
            return Err(RetrieverError::IndexError("Batch already in progress".to_string()));
        }
        self.index_writer = Some(self.index.writer(50_000_000)?);
        self.batch_mode = true;
        info!("Batch indexing mode started");
        Ok(())
    }

    pub fn end_batch(&mut self) -> Result<(), RetrieverError> {
        if let Some(mut writer) = self.index_writer.take() {
            writer.commit()?;
            self.batch_mode = false;
            self.clear_cache();
            if let Ok(reader) = self.index.reader() {
                self.metrics.total_documents_indexed = reader.searcher().num_docs() as usize;
            }
            info!("Batch indexing mode ended, changes committed");
            Ok(())
        } else {
            Err(RetrieverError::IndexError("No batch in progress".to_string()))
        }
    }

    pub fn add_documents_batch(&mut self, documents: Vec<(String, String, String)>) -> Result<usize, RetrieverError> {
    let was_batch = self.batch_mode;
    if !was_batch {
        self.begin_batch()?;
    }
    let mut count = 0;
    for (doc_id, title, content) in documents {  // ← unpack all 3 values
        if let Err(e) = self.add_document_to_batch(&doc_id, &title, &content) {  // ← pass all 3 to add_document_to_batch
            error!("Failed to add document '{}': {}", doc_id, e);
        } else {
            count += 1;
        }
    }
    if !was_batch {
        self.end_batch()?;
    }
    Ok(count)
}

    fn add_document_to_batch(&mut self, doc_id: &str, title: &str, content: &str) -> Result<(), RetrieverError> {
        if !self.batch_mode {
            return Err(RetrieverError::IndexError("Not in batch mode".to_string()));
        }
        let mut doc = tantivy::TantivyDocument::default();
        doc.add_text(self.doc_id_field, doc_id); 
        doc.add_text(self.title_field, title);
        doc.add_text(self.content_field, content);
        if let Some(writer) = &mut self.index_writer {
            writer.add_document(doc)?;
            Ok(())
        } else {
            Err(RetrieverError::IndexError("No writer available".to_string()))
        }
    }

    pub fn search(&mut self, query_str: &str) -> Result<Vec<String>, RetrieverError> {
        let start_time = Instant::now();
        if self.cache_enabled {
            if let Some(cached) = self.search_cache.get(query_str) {
                self.metrics.cache_hits += 1;
                self.metrics.total_searches += 1;
                let latency_us = start_time.elapsed().as_micros();
                self.metrics.total_search_latency_us += latency_us;
                self.metrics.avg_search_latency_us = 
                    self.metrics.total_search_latency_us as f64 / self.metrics.total_searches as f64;
                if latency_us > self.metrics.max_search_latency_us {
                    self.metrics.max_search_latency_us = latency_us;
                }
                self.metrics.last_updated = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs();
                return Ok(cached.clone());
            }
        }
        self.metrics.cache_misses += 1;
        self.metrics.total_searches += 1;
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let parser = QueryParser::for_index(&self.index, vec![self.title_field, self.content_field]);
        let query = parser.parse_query(query_str)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
            let content = doc
                .get_first(self.content_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            results.push(content);
        }
        if self.cache_enabled {
            self.search_cache.put(query_str.to_string(), results.clone());
        }
        let latency_us = start_time.elapsed().as_micros();
        self.metrics.total_search_latency_us += latency_us;
        self.metrics.avg_search_latency_us = 
            self.metrics.total_search_latency_us as f64 / self.metrics.total_searches as f64;
        if latency_us > self.metrics.max_search_latency_us {
            self.metrics.max_search_latency_us = latency_us;
        }
        self.metrics.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        Ok(results)
    }

    pub fn add_vector(&mut self, vector: Vec<f32>) {
        self.vectors.push(vector);
        self.metrics.total_vectors += 1;
        self.check_auto_save();
    }

    pub fn add_vector_with_id(&mut self, doc_id: String, vector: Vec<f32>) {
        let idx = self.vectors.len();
        self.vectors.push(vector);
        self.doc_id_to_vector_idx.insert(doc_id, idx);
        self.metrics.total_vectors += 1;
        self.check_auto_save();
    }

    fn check_auto_save(&mut self) {
        let count = self.documents_since_save.fetch_add(1, Ordering::SeqCst) + 1;
        if count >= self.auto_save_threshold {
            if let Err(e) = self.save_vectors(&self.vector_file_path.clone()) {
                error!("Auto-save failed: {}", e);
            } else {
                info!("Auto-saved vectors after {} documents", count);
                self.documents_since_save.store(0, Ordering::SeqCst);
            }
        }
    }

    pub fn vector_search(&self, query_vector: &[f32], top_k: usize) -> Vec<(usize, f32)> {
        let use_parallel = self.vectors.len() > 1000;
        let mut similarities: Vec<(usize, f32)> = if use_parallel {
            self.vectors
                .par_iter()
                .enumerate()
                .map(|(idx, vec)| (idx, cosine_similarity(query_vector, vec)))
                .collect()
        } else {
            self.vectors
                .iter()
                .enumerate()
                .map(|(idx, vec)| (idx, cosine_similarity(query_vector, vec)))
                .collect()
        };
        similarities.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        similarities.into_iter().take(top_k).collect()
    }

    pub fn hybrid_search(&mut self, query: &str, query_vector: Option<&[f32]>) -> Result<Vec<String>, RetrieverError> {
        let keyword_results = self.search(query)?;
        let query_vec = match query_vector {
            Some(v) => v,
            None => return Ok(keyword_results),
        };
        let vector_results = self.vector_search(query_vec, 10);
        let k = 60.0;
        let mut score_map: HashMap<String, f32> = HashMap::new();
        for (rank, content) in keyword_results.iter().enumerate() {
            let score = 1.0 / (k + (rank as f32) + 1.0);
            *score_map.entry(content.clone()).or_insert(0.0) += score;
        }
        for (rank, (idx, _similarity)) in vector_results.iter().enumerate() {
            if let Some(content) = self.get_content_by_vector_idx(*idx) { // ← Fixed: dereference idx
                let score = 1.0 / (k + (rank as f32) + 1.0);
                *score_map.entry(content).or_insert(0.0) += score;
            }
        }
        let mut merged_results: Vec<(String, f32)> = score_map.into_iter().collect();
        merged_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(merged_results.into_iter().take(10).map(|(content, _)| content).collect())
    }

    pub fn get_content_by_vector_idx(&self, idx: usize) -> Option<String> {
        for (doc_id, &vec_idx) in &self.doc_id_to_vector_idx {
            if vec_idx == idx {
                return Some(doc_id.clone());
            }
        }
        None
    }

    pub fn rerank_by_similarity(&self, _query: &str, candidates: &Vec<String>) -> Vec<String> {
        let mut results = candidates.clone();
        results.reverse();
        results
    }

    pub fn rerank_by_vector_similarity(&self, query_vector: &[f32], candidate_indices: &[usize]) -> Result<Vec<(usize, f32)>, RetrieverError> {
        let mut scored_candidates: Vec<(usize, f32)> = candidate_indices
            .iter()
            .filter_map(|&idx| {
                if idx < self.vectors.len() {
                    Some((idx, cosine_similarity(query_vector, &self.vectors[idx])))
                } else {
                    None
                }
            })
            .collect();
        if scored_candidates.is_empty() && !candidate_indices.is_empty() {
            return Err(RetrieverError::VectorError("No valid candidate indices found".to_string()));
        }
        scored_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(scored_candidates)
    }

    pub fn summarize_chunks(&self, _query: &str, candidates: &Vec<String>) -> String {
        format!("Summary for {} chunks", candidates.len())
    }

    pub fn index_document(&mut self, doc: impl tantivy::Document) -> Result<(), RetrieverError> {
        let mut index_writer = self.index.writer(50_000_000)?;
        index_writer.add_document(doc)?;
        index_writer.commit()?;
        self.clear_cache();
        self.metrics.total_documents_indexed += 1;
        Ok(())
    }

    pub fn add_document(&mut self, doc_id: &str, title: &str, content: &str) -> Result<(), RetrieverError> {
        if self.batch_mode {
            return self.add_document_to_batch(doc_id, title, content);
        }
        let mut doc = tantivy::TantivyDocument::default();
        doc.add_text(self.doc_id_field, doc_id);
        doc.add_text(self.title_field, title);
        doc.add_text(self.content_field, content);
        let mut index_writer = self.index.writer(50_000_000)?;
        index_writer.add_document(doc)?;
        index_writer.commit()?;
        self.clear_cache();
        self.metrics.total_documents_indexed += 1;
        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), RetrieverError> {
        if self.batch_mode {
            self.end_batch()?;
        }
        self.save_vectors(&self.vector_file_path.clone())?;
        Ok(())
    }

    pub fn save_vectors(&self, filename: &str) -> Result<(), RetrieverError> {
        let storage = VectorStorage {
            vectors: self.vectors.clone(),
            doc_id_to_vector_idx: self.doc_id_to_vector_idx.clone(),
        };
        let json = serde_json::to_string(&storage)
            .map_err(|e| RetrieverError::SerializationError(e.to_string()))?;
        let mut file = File::create(filename)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load_vectors(&mut self, filename: &str) -> Result<(), RetrieverError> {
        let mut file = File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let storage: VectorStorage = serde_json::from_str(&contents)
            .map_err(|e| RetrieverError::SerializationError(e.to_string()))?;
        self.vectors = storage.vectors;
        self.doc_id_to_vector_idx = storage.doc_id_to_vector_idx;
        self.metrics.total_vectors = self.vectors.len();
        Ok(())
    }

    pub fn force_save(&mut self) -> Result<(), RetrieverError> {
        info!("Manual save triggered");
        self.save_vectors(&self.vector_file_path)?;
        self.documents_since_save.store(0, Ordering::SeqCst);
        Ok(())
    }

    pub fn set_auto_save_threshold(&mut self, threshold: usize) {
        self.auto_save_threshold = threshold;
        info!("Auto-save threshold set to {} documents", threshold);
    }

    pub fn index_chunk(&mut self, chunk_id: &str, chunk_text: &str, vector: &Vec<f32>) -> Result<(), RetrieverError> {
        self.add_document(chunk_id, chunk_id, chunk_text)?;
        self.add_vector_with_id(chunk_id.to_string(), vector.clone());
        Ok(())
    }

    fn check_disk_space(&self, min_free_bytes: u64) -> Result<(), RetrieverError> {
        let path = Path::new(&self.index_dir_path);
        let available_space = fs2::available_space(path)
            .map_err(|e| RetrieverError::IoError(format!("Failed to get disk space: {}", e)))?;
        if available_space < min_free_bytes {
            return Err(RetrieverError::IoError(
                format!("Insufficient disk space: {} bytes available, {} bytes required", 
                       available_space, min_free_bytes)
            ));
        }
        Ok(())
    }

    fn validate_vector_dimensions(&self) -> Result<(), RetrieverError> {
        if self.vectors.is_empty() {
            return Ok(());
        }
        let expected_dim = self.vectors[0].len();
        for (idx, vector) in self.vectors.iter().enumerate() {
            if vector.len() != expected_dim {
                return Err(RetrieverError::VectorError(
                    format!("Vector dimension mismatch at index {}: expected {}, found {}", 
                           idx, expected_dim, vector.len())
                ));
            }
        }
        info!("Vector dimension validation passed: {} vectors with dimension {}", 
              self.vectors.len(), expected_dim);
        Ok(())
    }

    pub fn health_check(&self) -> Result<(), RetrieverError> {
        let index_path = Path::new(&self.index_dir_path);
        if !index_path.exists() {
            return Err(RetrieverError::DirectoryError(
                format!("Index directory does not exist: {}", self.index_dir_path)
            ));
        }
        if !index_path.is_dir() {
            return Err(RetrieverError::DirectoryError(
                format!("Index path is not a directory: {}", self.index_dir_path)
            ));
        }

        let reader = self.index.reader()
            .map_err(|e| RetrieverError::IndexError(format!("Failed to create index reader: {}", e)))?;
        let searcher = reader.searcher();
        let doc_count = searcher.num_docs();

        if self.vectors.len() != self.doc_id_to_vector_idx.len() {
            return Err(RetrieverError::VectorError(
                format!("Vector storage inconsistency: {} vectors but {} document mappings", 
                       self.vectors.len(), self.doc_id_to_vector_idx.len())
            ));
        }

        for (doc_id, &vec_idx) in &self.doc_id_to_vector_idx {
            if vec_idx >= self.vectors.len() {
                return Err(RetrieverError::VectorError(
                    format!("Invalid vector index {} for document '{}' (vectors length: {})", 
                           vec_idx, doc_id, self.vectors.len())
                ));
            }
        }

        self.validate_vector_dimensions()?;

        if doc_count > 0 {
            let parser = QueryParser::for_index(&self.index, vec![self.title_field, self.content_field]);
            match parser.parse_query("*") {
                Ok(query) => {
                    if let Err(e) = searcher.search(&query, &TopDocs::with_limit(1)) {
                        return Err(RetrieverError::IndexError(
                            format!("Basic search test failed: {}", e)
                        ));
                    }
                }
                Err(_e) => {
                    let fallback_query = parser.parse_query("a").unwrap_or_else(|_| parser.parse_query("*").unwrap());
                    let first_doc_addr = searcher
                        .search(&fallback_query, &TopDocs::with_limit(1))
                        .map(|top_docs| top_docs.first().map(|(_, addr)| *addr))
                        .unwrap_or(None);
                    if let Some(addr) = first_doc_addr {
                        if let Err(e) = searcher.doc::<tantivy::TantivyDocument>(addr) {
                            return Err(RetrieverError::IndexError(
                                format!("Failed to retrieve document: {}", e)
                            ));
                        }
                    }
                }
            }
        }

        if self.cache_enabled && !self.search_cache.is_empty() {
            let _ = self.search_cache.len();
            let _ = self.search_cache.cap();
        }

        if Path::new(&self.vector_file_path).exists() {
            if let Err(e) = std::fs::OpenOptions::new().write(true).open(&self.vector_file_path) {
                return Err(RetrieverError::IoError(
                    format!("Vector file exists but is not writable: {}", e)
                ));
            }
        } else {
            if let Some(parent) = Path::new(&self.vector_file_path).parent() {
                if !parent.exists() {
                    return Err(RetrieverError::IoError(
                        format!("Vector file parent directory does not exist: {:?}", parent)
                    ));
                }
                let temp_file = parent.join(".health_check_test");
                if let Err(e) = std::fs::File::create(&temp_file) {
                    let _ = std::fs::remove_file(&temp_file);
                    return Err(RetrieverError::IoError(
                        format!("Cannot write to vector file directory: {}", e)
                    ));
                }
                let _ = std::fs::remove_file(&temp_file);
            }
        }

        self.check_disk_space(100 * 1024 * 1024)?;

        info!("Health check passed - {} documents, {} vectors", doc_count, self.vectors.len());
        Ok(())
    }

    pub fn ready_check(&self) -> Result<(), RetrieverError> {
        let reader = self.index.reader()
            .map_err(|e| RetrieverError::IndexError(format!("Failed to create index reader: {}", e)))?;
        let _searcher = reader.searcher();
        let _ = self.vectors.len();
        let _ = self.doc_id_to_vector_idx.len();
        Ok(())
    }
}

impl Drop for Retriever {
    fn drop(&mut self) {
        if self.batch_mode {
            if let Err(e) = self.end_batch() {
                error!("Failed to end batch on shutdown: {}", e);
            }
        }
        info!("Retriever shutting down, saving vectors...");
        if let Err(e) = self.save_vectors(&self.vector_file_path.clone()) {
            error!("Failed to save vectors on shutdown: {}", e);
        } else {
            info!("Vectors saved successfully on shutdown");
        }
    }
}

