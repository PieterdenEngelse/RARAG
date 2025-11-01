use std::env;
use crate::path_manager::PathManager;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    // Network
    pub host: String,
    pub port: u16,
    
    // Phase 15 - Reliability & Observability
    pub skip_initial_indexing: bool,
    pub index_in_ram: bool,
    pub reindex_webhook_url: Option<String>,
    pub rate_limit_enabled: bool,
    
    // Path Management
    pub path_manager: PathManager,
    
    // Redis L3 Cache
    pub redis_enabled: bool,
    pub redis_url: Option<String>,
    pub redis_ttl: u64,
}

impl ApiConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        
        // Network configuration
        let host = env::var("BACKEND_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        
        let port = env::var("BACKEND_PORT")
            .unwrap_or_else(|_| "3010".to_string())
            .parse()
            .expect("BACKEND_PORT must be a valid u16");
        
        // Phase 15 - Reliability & Observability
        let skip_initial_indexing = env::var("SKIP_INITIAL_INDEXING")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);
        
        let index_in_ram = env::var("INDEX_IN_RAM")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);
        
        let reindex_webhook_url = env::var("REINDEX_WEBHOOK_URL").ok();
        
        let rate_limit_enabled = env::var("RATE_LIMIT_ENABLED")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);
        
        // Path Management
        let path_manager = PathManager::new()
            .expect("Failed to initialize PathManager");
        
        // Redis L3 Cache
        let redis_enabled = env::var("REDIS_ENABLED")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);
        
        let redis_url = env::var("REDIS_URL").ok();
        
        let redis_ttl = env::var("REDIS_TTL")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .unwrap_or(3600);
        
        Self {
            host,
            port,
            skip_initial_indexing,
            index_in_ram,
            reindex_webhook_url,
            rate_limit_enabled,
            path_manager,
            redis_enabled,
            redis_url,
            redis_ttl,
        }
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}