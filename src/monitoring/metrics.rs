//! Prometheus metrics collection
//! 
//! Tracks:
//! - API request latency (histogram)
//! - API errors (counter)
//! - Startup duration (gauge)
//! - Database query performance (histogram)
//! - Active connections (gauge)
//! - Memory usage (gauge)

use prometheus::{
    Counter, CounterVec, Gauge, HistogramVec, 
    Registry, Result as PrometheusResult, TextEncoder,
};
use super::config::MonitoringConfig;

pub struct MetricsRegistry {
    registry: Registry,
    
    // API metrics
    pub api_request_duration: HistogramVec,
    pub api_errors: CounterVec,
    pub api_requests_total: CounterVec,
    
    // Startup metrics
    pub startup_duration_ms: Gauge,
    
    // Database metrics
    pub db_query_duration: HistogramVec,
    pub db_connections_active: Gauge,
    pub db_errors: Counter,
    
    // System metrics
    pub memory_usage_bytes: Gauge,
    pub uptime_seconds: Gauge,
}

impl MetricsRegistry {
    /// Create new metrics registry
    pub fn new(_config: &MonitoringConfig) -> Self {
        let registry = Registry::new();
        
        // API request latency histogram (buckets in ms: 10, 50, 100, 500, 1000, 5000)
        let api_request_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "api_request_duration_ms",
                "API request latency in milliseconds",
            )
            .buckets(vec![10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0]),
            &["endpoint", "method", "status"],
        ).expect("Failed to create api_request_duration metric");
        registry.register(Box::new(api_request_duration.clone()))
            .expect("Failed to register api_request_duration");
        
        // API errors counter
        let api_errors = CounterVec::new(
            prometheus::opts!("api_errors_total", "Total API errors"),
            &["endpoint", "status_code"],
        ).expect("Failed to create api_errors metric");
        registry.register(Box::new(api_errors.clone()))
            .expect("Failed to register api_errors");
        
        // API requests total counter
        let api_requests_total = CounterVec::new(
            prometheus::opts!("api_requests_total", "Total API requests"),
            &["endpoint", "method", "status"],
        ).expect("Failed to create api_requests_total metric");
        
        // Startup duration gauge
        let startup_duration_ms = Gauge::new(
            "startup_duration_ms",
            "Application startup duration in milliseconds",
        ).expect("Failed to create startup_duration_ms metric");
        registry.register(Box::new(startup_duration_ms.clone()))
            .expect("Failed to register startup_duration_ms");
        
        // Database query duration histogram
        let db_query_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "db_query_duration_ms",
                "Database query duration in milliseconds",
            )
            .buckets(vec![10.0, 50.0, 100.0, 500.0, 1000.0]),
            &["query_type"],
        ).expect("Failed to create db_query_duration metric");
        registry.register(Box::new(db_query_duration.clone()))
            .expect("Failed to register db_query_duration");
        
        // Active database connections gauge
        let db_connections_active = Gauge::new(
            "db_connections_active",
            "Currently active database connections",
        ).expect("Failed to create db_connections_active metric");
        registry.register(Box::new(db_connections_active.clone()))
            .expect("Failed to register db_connections_active");
        
        // Database errors counter
        let db_errors = Counter::new(
            "db_errors_total",
            "Total database errors",
        ).expect("Failed to create db_errors metric");
        registry.register(Box::new(db_errors.clone()))
            .expect("Failed to register db_errors");
        
        // Memory usage gauge
        let memory_usage_bytes = Gauge::new(
            "memory_usage_bytes",
            "Current memory usage in bytes",
        ).expect("Failed to create memory_usage_bytes metric");
        registry.register(Box::new(memory_usage_bytes.clone()))
            .expect("Failed to register memory_usage_bytes");
        
        // Uptime gauge
        let uptime_seconds = Gauge::new(
            "uptime_seconds",
            "Application uptime in seconds",
        ).expect("Failed to create uptime_seconds metric");
        registry.register(Box::new(uptime_seconds.clone()))
            .expect("Failed to register uptime_seconds");
        
        Self {
            registry,
            api_request_duration,
            api_errors,
            api_requests_total,
            startup_duration_ms,
            db_query_duration,
            db_connections_active,
            db_errors,
            memory_usage_bytes,
            uptime_seconds,
        }
    }
    
    /// Record API request
    pub fn record_api_request(
        &self,
        endpoint: &str,
        method: &str,
        status_code: u16,
        duration_ms: f64,
    ) {
        let status = match status_code {
            200..=299 => "2xx",
            300..=399 => "3xx",
            400..=499 => "4xx",
            _ => "5xx",
        };
        
        self.api_request_duration
            .with_label_values(&[endpoint, method, status])
            .observe(duration_ms);
        
        self.api_requests_total
            .with_label_values(&[endpoint, method, status])
            .inc();
        
        if status_code >= 400 {
            self.api_errors
                .with_label_values(&[endpoint, &status_code.to_string()])
                .inc();
        }
    }
    
    /// Record startup time
    pub fn record_startup_time(&self, duration: std::time::Duration) {
        self.startup_duration_ms.set(duration.as_millis() as f64);
    }
    
    /// Record database query
    pub fn record_db_query(&self, query_type: &str, duration_ms: f64) {
        self.db_query_duration
            .with_label_values(&[query_type])
            .observe(duration_ms);
    }
    
    /// Update active database connections
    pub fn set_active_connections(&self, count: f64) {
        self.db_connections_active.set(count);
    }
    
    /// Increment database errors
    pub fn inc_db_errors(&self) {
        self.db_errors.inc();
    }
    
    /// Update memory usage
    pub fn set_memory_usage(&self, bytes: f64) {
        self.memory_usage_bytes.set(bytes);
    }
    
    /// Update uptime
    pub fn set_uptime(&self, seconds: f64) {
        self.uptime_seconds.set(seconds);
    }
    
    /// Export metrics in Prometheus text format
    pub fn export(&self) -> PrometheusResult<String> {
        let encoder = TextEncoder::new();
        encoder.encode_to_string(&self.registry.gather())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_registry_creation() {
        let config = super::super::config::MonitoringConfig::default();
        let metrics = MetricsRegistry::new(&config);
        
        // Should export successfully
        let export = metrics.export();
        assert!(export.is_ok());
    }
    
    #[test]
    fn test_record_api_request() {
        let config = super::super::config::MonitoringConfig::default();
        let metrics = MetricsRegistry::new(&config);
        
        metrics.record_api_request("/search", "GET", 200, 45.5);
        metrics.record_api_request("/search", "GET", 500, 1500.0);
        
        let export = metrics.export().unwrap();
        assert!(export.contains("api_request_duration_ms"));
        assert!(export.contains("api_requests_total"));
    }
}