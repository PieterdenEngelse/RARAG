// src/cache/invalidation.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationTrigger {
    TTLExpired,
    ExplicitDelete,
    DependencyChange { dependency_id: String },
    MemoryUpdate { agent_id: String },
    DocumentUpdate { doc_id: String },
    SystemMaintenance,
}

pub struct CacheInvalidator {
    // Track dependencies: key -> [dependent_keys]
    dependencies: Arc<Mutex<HashMap<String, HashSet<String>>>>,
}

impl CacheInvalidator {
    pub async fn invalidate_by_trigger(&self, trigger: InvalidationTrigger, cache: &dyn CacheBackend) -> Result<()> {
        match trigger {
            InvalidationTrigger::DocumentUpdate { doc_id } => {
                // Invalidate all chunks from this document
                let chunk_keys = self.get_affected_keys(&format!("doc:{}", doc_id)).await?;
                for key in chunk_keys {
                    cache.delete(&key).await?;
                }
            }
            InvalidationTrigger::MemoryUpdate { agent_id } => {
                // Invalidate agent's search results
                let search_keys = self.get_affected_keys(&format!("agent:{}", agent_id)).await?;
                for key in search_keys {
                    cache.delete(&key).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn get_affected_keys(&self, trigger_key: &str) -> Result<Vec<String>> {
        let deps = self.dependencies.lock().await;
        Ok(deps.get(trigger_key).cloned().unwrap_or_default().into_iter().collect())
    }
}