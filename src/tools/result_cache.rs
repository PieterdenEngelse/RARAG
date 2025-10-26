// src/tools/result_cache.rs - PRODUCTION
// Phase 10 Optimization: Result Caching System

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Instant, Duration};
use crate::tools::ToolResult;

#[derive(Clone, Debug)]
pub struct CachedResult {
    pub result: ToolResult,
    pub timestamp: Instant,
}

pub struct ResultCache {
    cache: Arc<RwLock<HashMap<String, CachedResult>>>,
    ttl: Duration,
}

impl ResultCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Generate cache key from tool type and query
    fn cache_key(tool_type: &str, query: &str) -> String {
        format!("{}::{}", tool_type, query)
    }

    /// Get result from cache if exists and not expired
    pub async fn get(&self, tool_type: &str, query: &str) -> Option<ToolResult> {
        let cache = self.cache.read().await;
        let key = Self::cache_key(tool_type, query);
        
        if let Some(cached) = cache.get(&key) {
            // Check if expired
            if cached.timestamp.elapsed() < self.ttl {
                return Some(cached.result.clone());
            }
        }
        
        None
    }

    /// Store result in cache
    pub async fn set(&self, tool_type: &str, query: String, result: ToolResult) {
        let mut cache = self.cache.write().await;
        let key = Self::cache_key(tool_type, &query);
        
        cache.insert(key, CachedResult {
            result,
            timestamp: Instant::now(),
        });
    }

    /// Clear expired entries
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        cache.retain(|_, cached| cached.timestamp.elapsed() < self.ttl);
    }

    /// Clear entire cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache size
    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = ResultCache::new(60);
        
        let result = ToolResult {
            tool: crate::tools::ToolType::Calculator,
            success: true,
            result: "5".to_string(),
            metadata: crate::tools::ToolMetadata {
                execution_time_ms: 100,
                confidence: 0.99,
                source: None,
                cost: None,
            },
        };
        
        cache.set("calculator", "5 + 0".to_string(), result.clone()).await;
        let retrieved = cache.get("calculator", "5 + 0").await;
        
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        let cache = ResultCache::new(1);
        
        let result = ToolResult {
            tool: crate::tools::ToolType::Calculator,
            success: true,
            result: "5".to_string(),
            metadata: crate::tools::ToolMetadata {
                execution_time_ms: 100,
                confidence: 0.99,
                source: None,
                cost: None,
            },
        };
        
        cache.set("calculator", "5 + 0".to_string(), result).await;
        assert!(cache.get("calculator", "5 + 0").await.is_some());
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(cache.get("calculator", "5 + 0").await.is_none());
    }
}