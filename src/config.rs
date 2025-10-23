// src/config.rs
use std::env;

#[derive(Debug)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
}

impl ApiConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let host = env::var("BACKEND_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("BACKEND_PORT")
            .unwrap_or_else(|_| "3010".to_string())
            .parse()
            .expect("BACKEND_PORT must be a valid u16");
        Self { host, port }
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}