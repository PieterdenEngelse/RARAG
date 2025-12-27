// src/embedder.rs - UPDATED for Phase 2
// Extends your existing embedder with async/batching/caching

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Embedding vector (384-dimensional for all-MiniLM-L6-v2)
pub type EmbeddingVector = Vec<f32>;

/// Basic embedding function (your existing hash-based approach)
pub fn embed(text: &str) -> Vec<f32> {
    let hash = seahash::hash(text.as_bytes());
    let mut vec = vec![0.0; 384];
    vec[0] = (hash & 0xFFFF) as f32;
    vec
}

/// Embedding cache using LRU strategy
type EmbeddingCache = LruCache<String, EmbeddingVector>;

/// Configuration for the embedding service
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub batch_size: usize,
    pub cache_size: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            batch_size: 32,
            cache_size: 10_000,
        }
    }
}

/// Thread-safe async embedding service with caching and batching
pub struct EmbeddingService {
    config: EmbeddingConfig,
    cache: Arc<RwLock<EmbeddingCache>>,
}

impl EmbeddingService {
    /// Create a new embedding service
    pub fn new(config: EmbeddingConfig) -> Self {
        let cache_size = NonZeroUsize::new(config.cache_size).expect("cache_size must be > 0");

        let cache = LruCache::new(cache_size);

        info!(
            batch_size = config.batch_size,
            cache_size = config.cache_size,
            "Initializing EmbeddingService"
        );

        Self {
            config,
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    /// Embed a single text, with cache lookup
    pub async fn embed_text(&self, text: &str) -> EmbeddingVector {
        let key = format!("{:x}", seahash::hash(text.as_bytes()));

        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(embedding) = cache.get(&key) {
                debug!(cache_key = %key, text_len = text.len(), "Cache hit for embedding");
                return embedding.clone();
            }
        }

        debug!(text_len = text.len(), "Generating embedding");

        // Generate embedding
        let embedding = embed(text);

        // Store in cache
        {
            let mut cache = self.cache.write().await;
            cache.put(key.clone(), embedding.clone());
        }

        embedding
    }

    /// Embed multiple texts in batches (efficient for bulk operations)
    pub async fn embed_batch(&self, texts: &[&str]) -> Vec<EmbeddingVector> {
        info!(
            total_texts = texts.len(),
            batch_size = self.config.batch_size,
            "Starting batch embedding"
        );

        let mut results = Vec::new();

        for batch in texts.chunks(self.config.batch_size) {
            for text in batch {
                results.push(self.embed_text(text).await);
            }
            // Yield to tokio runtime to avoid blocking
            tokio::task::yield_now().await;
        }

        info!(
            total_embeddings = results.len(),
            "Batch embedding completed"
        );
        results
    }

    /// Embed multiple texts with indices (preserves order)
    pub async fn embed_indexed_batch(
        &self,
        texts: &[(usize, &str)],
    ) -> Vec<(usize, EmbeddingVector)> {
        info!(
            total_texts = texts.len(),
            batch_size = self.config.batch_size,
            "Starting indexed batch embedding"
        );

        let mut results = Vec::new();

        for batch in texts.chunks(self.config.batch_size) {
            for (idx, text) in batch {
                let embedding = self.embed_text(text).await;
                results.push((*idx, embedding));
            }
            tokio::task::yield_now().await;
        }

        info!(
            total_embeddings = results.len(),
            "Indexed batch embedding completed"
        );
        results
    }

    /// Clear the embedding cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Embedding cache cleared");
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            len: cache.len(),
            cap: cache.cap().get(),
        }
    }

    /// Embed a query string (for semantic search)
    pub async fn embed_query(&self, query: &str) -> EmbeddingVector {
        debug!(query = %query, "Generating query embedding");
        self.embed_text(query).await
    }
}

/// Statistics for the embedding cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub len: usize,
    pub cap: usize,
}

/// Similarity search helper functions
pub mod similarity {
    use super::EmbeddingVector;

    /// Cosine similarity between two vectors
    pub fn cosine_similarity(a: &EmbeddingVector, b: &EmbeddingVector) -> f32 {
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            0.0
        } else {
            dot / (mag_a * mag_b)
        }
    }

    /// Euclidean distance between two vectors
    pub fn euclidean_distance(a: &EmbeddingVector, b: &EmbeddingVector) -> f32 {
        if a.is_empty() || b.is_empty() {
            return f32::INFINITY;
        }

        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Find top-k most similar embeddings
    pub fn top_k_similar(
        query_embedding: &EmbeddingVector,
        candidates: &[(usize, &EmbeddingVector)],
        k: usize,
    ) -> Vec<(usize, f32)> {
        let mut scores: Vec<_> = candidates
            .iter()
            .map(|(idx, emb)| (*idx, cosine_similarity(query_embedding, emb)))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);
        scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_basic() {
        let vec = embed("hello world");
        assert_eq!(vec.len(), 384);
        assert!(vec.iter().any(|&x| x != 0.0));
    }

    #[tokio::test]
    async fn test_embedding_service_creation() {
        let config = EmbeddingConfig::default();
        let service = EmbeddingService::new(config);

        let stats = service.cache_stats().await;
        assert_eq!(stats.len, 0);
    }

    #[tokio::test]
    async fn test_embed_text() {
        let service = EmbeddingService::new(EmbeddingConfig::default());

        let embedding = service.embed_text("test query").await;
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn test_embedding_cache_hit() {
        let service = EmbeddingService::new(EmbeddingConfig::default());

        // First call
        let result1 = service.embed_text("test").await;

        // Second call (should hit cache)
        let result2 = service.embed_text("test").await;

        assert_eq!(result1, result2);

        let stats = service.cache_stats().await;
        assert_eq!(stats.len, 1);
    }

    #[tokio::test]
    async fn test_batch_embedding() {
        let service = EmbeddingService::new(EmbeddingConfig {
            batch_size: 2,
            ..Default::default()
        });

        let texts = vec!["text 1", "text 2", "text 3"];
        let results = service.embed_batch(&texts).await;

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|v| v.len() == 384));
    }

    #[tokio::test]
    async fn test_indexed_batch() {
        let service = EmbeddingService::new(EmbeddingConfig::default());

        let texts = vec![(0usize, "text 1"), (1usize, "text 2"), (2usize, "text 3")];
        let results = service.embed_indexed_batch(&texts).await;

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 0);
        assert_eq!(results[1].0, 1);
        assert_eq!(results[2].0, 2);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let service = EmbeddingService::new(EmbeddingConfig::default());

        let _ = service.embed_text("test").await;

        let stats = service.cache_stats().await;
        assert!(stats.len > 0);

        service.clear_cache().await;

        let stats = service.cache_stats().await;
        assert_eq!(stats.len, 0);
    }

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let v3 = vec![0.0, 1.0, 0.0];

        assert!((similarity::cosine_similarity(&v1, &v2) - 1.0).abs() < 0.001);
        assert!((similarity::cosine_similarity(&v1, &v3)).abs() < 0.001);
    }

    #[test]
    fn test_top_k_similar() {
        let query = vec![1.0, 0.0];
        let v1 = vec![1.0, 0.0];
        let v2 = vec![0.0, 1.0];
        let v3 = vec![0.9, 0.1];

        let candidates = vec![(0, &v1), (1, &v2), (2, &v3)];

        let results = similarity::top_k_similar(&query, &candidates, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0); // Most similar
    }

    #[tokio::test]
    async fn test_embed_query() {
        let service = EmbeddingService::new(EmbeddingConfig::default());
        let embedding = service.embed_query("test query").await;
        assert_eq!(embedding.len(), 384);
    }
}
