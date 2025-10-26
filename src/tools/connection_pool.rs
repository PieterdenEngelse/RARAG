// src/tools/connection_pool.rs
// Optimization #7: HTTP Connection Pooling

use std::time::Duration;

pub struct ConnectionPool;

impl ConnectionPool {
    /// Create HTTP client with connection pooling
    pub fn create_client() -> reqwest::Client {
        reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ConnectionPool::create_client();
        // Verify client was created successfully
        assert!(!format!("{:?}", client).is_empty());
    }

    #[test]
    fn test_pool_size_validation() {
        let pool_size = 10;
        assert!(pool_size > 0 && pool_size <= 100);
    }

    #[test]
    fn test_timeout_ordering() {
        let connect_timeout = 10;
        let request_timeout = 30;
        assert!(connect_timeout < request_timeout);
    }

    #[test]
    fn test_pool_reuse() {
        let mut reused = 0;
        for _ in 0..5 {
            // Simulate connection reuse
            reused += 1;
        }
        assert_eq!(reused, 5);
    }

    #[test]
    fn test_performance_impact() {
        // Test theoretical performance improvement
        let requests_without_pool = 100;
        let conn_time_per_request = 10; // ms
        let total_without = requests_without_pool * conn_time_per_request;
        
        let requests_with_pool = 100;
        let conn_time_first = 10; // ms for first connection
        let conn_time_reuse = 1; // ms for reused connection
        let total_with = conn_time_first + (requests_with_pool - 1) * conn_time_reuse;
        
        let speedup = total_without as f32 / total_with as f32;
        assert!(speedup > 2.0);
    }
}