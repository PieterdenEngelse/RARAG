//! Monitoring module for agentic-rag
//! 
//! Provides:
//! - Structured logging with tracing
//! - Prometheus metrics collection
//! - Health checks endpoints
//! - Performance instrumentation
//!
//! INSTALLER IMPACT:
//! - Creates ~/.agentic-rag/logs/ directory
//! - Requires RUST_LOG environment variable
//! - Requires MONITORING_ENABLED=true environment variable

pub mod config;
pub mod metrics;
pub mod tracing_config;
pub mod health;
pub mod handlers;

pub use config::MonitoringConfig;
pub use metrics::MetricsRegistry;
pub use health::HealthStatus;

use std::sync::Arc;
use std::time::Instant;

/// Monitoring context shared across the application
#[derive(Clone)]
pub struct MonitoringContext {
    pub config: MonitoringConfig,
    pub metrics: Arc<MetricsRegistry>,
    pub health: Arc<health::HealthTracker>,
    pub startup_time: Instant,
}

impl MonitoringContext {
    /// Initialize monitoring system
    /// 
    /// INSTALLER IMPACT:
    /// - Must be called before starting API server
    /// - Creates log directories
    /// - Initializes tracing subscriber
    pub fn new(config: MonitoringConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize tracing (logging)
        let _guard = tracing_config::init_tracing(&config)?;
        
        // Initialize metrics
        let metrics = Arc::new(MetricsRegistry::new(&config));
        
        // Initialize health tracker
        let health = Arc::new(health::HealthTracker::new());
        
        let startup_time = Instant::now();
        
        tracing::info!("Monitoring system initialized");
        
        Ok(Self {
            config,
            metrics,
            health,
            startup_time,
        })
    }
    
    /// Record startup completion
    /// 
    /// INSTALLER IMPACT:
    /// - Must be called after server is listening
    /// - Records startup duration in metrics
    /// - Marks system as ready
    pub fn startup_complete(&self) {
        let startup_duration = self.startup_time.elapsed();
        self.metrics.record_startup_time(startup_duration);
        self.health.mark_ready();
        
        tracing::info!(
            duration_ms = startup_duration.as_millis(),
            "Application startup complete"
        );
    }
    
    /// Get current health status
    pub fn health_status(&self) -> HealthStatus {
        self.health.get_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monitoring_context_creation() {
        let config = MonitoringConfig::default();
        let ctx = MonitoringContext::new(config);
        assert!(ctx.is_ok());
    }
}