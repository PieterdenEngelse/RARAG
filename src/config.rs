// ag/src/config.rs v13.1.2 - UPDATED with PathManager
use std::env;
use crate::path_manager::PathManager;

#[derive(Debug)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    // Phase 12: Redis L3 Cache configuration
    pub redis_enabled: bool,
    pub redis_url: Option<String>,
    pub redis_ttl: u64,
    // Phase 13: PathManager for centralized paths
    pub path_manager: PathManager,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        
        let host = env::var("BACKEND_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("BACKEND_PORT")
            .unwrap_or_else(|_| "3010".to_string())
            .parse()
            .expect("BACKEND_PORT must be a valid u16");
        
        // Phase 12: Redis configuration
        let redis_enabled = env::var("REDIS_ENABLED")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);
        let redis_url = env::var("REDIS_URL").ok();
        let redis_ttl = env::var("REDIS_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600);
        
        // Phase 13: Initialize PathManager
        let path_manager = PathManager::new()?;
        
        Ok(Self {
            host,
            port,
            redis_enabled,
            redis_url,
            redis_ttl,
            path_manager,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}