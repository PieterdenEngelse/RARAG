//! Health check endpoints
//!
//! Provides:
//! - GET /monitoring/health - Full health status
//! - GET /monitoring/ready - Readiness probe (K8s compatible)
//! - GET /monitoring/live - Liveness probe (K8s compatible)
//!
//! INSTALLER IMPACT:
//! - Installer must call /health endpoint to verify startup
//! - Should check "status" field == "healthy"

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentStatus {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "degraded")]
    Degraded,
    #[serde(rename = "unhealthy")]
    Unhealthy,
}

impl std::fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentStatus::Healthy => write!(f, "healthy"),
            ComponentStatus::Degraded => write!(f, "degraded"),
            ComponentStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: ComponentStatus,
    pub timestamp: String,
    pub uptime_seconds: f64,
    pub components: ComponentHealth,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub api: ComponentStatus,
    pub database: ComponentStatus,
    pub configuration: ComponentStatus,
    pub logging: ComponentStatus,
}

impl Default for ComponentHealth {
    fn default() -> Self {
        Self {
            api: ComponentStatus::Unhealthy,
            database: ComponentStatus::Unhealthy,
            configuration: ComponentStatus::Unhealthy,
            logging: ComponentStatus::Unhealthy,
        }
    }
}

/// Tracks application health
pub struct HealthTracker {
    is_ready: AtomicBool,
    is_live: AtomicBool,
    components: parking_lot::RwLock<ComponentHealth>,
    startup_time: std::time::Instant,
}

impl HealthTracker {
    /// Create new health tracker
    pub fn new() -> Self {
        Self {
            is_ready: AtomicBool::new(false),
            is_live: AtomicBool::new(true), // Always live until told otherwise
            components: parking_lot::RwLock::new(ComponentHealth::default()),
            startup_time: std::time::Instant::now(),
        }
    }

    /// Mark system as ready
    ///
    /// INSTALLER IMPACT:
    /// - Call after all components initialized
    /// - /ready endpoint will return 200 after this
    pub fn mark_ready(&self) {
        self.is_ready.store(true, Ordering::SeqCst);
        tracing::info!("System marked as ready");
    }

    /// Mark system as not ready
    pub fn mark_not_ready(&self) {
        self.is_ready.store(false, Ordering::SeqCst);
        tracing::warn!("System marked as not ready");
    }

    /// Mark system as not live (will restart if running in container)
    pub fn mark_not_live(&self) {
        self.is_live.store(false, Ordering::SeqCst);
        tracing::error!("System marked as not live");
    }

    /// Update component status
    pub fn set_component_status(&self, component: &str, status: ComponentStatus) {
        let mut components = self.components.write();
        match component {
            "api" => components.api = status.clone(),
            "database" => components.database = status.clone(),
            "configuration" => components.configuration = status.clone(),
            "logging" => components.logging = status.clone(),
            _ => {}
        }
        tracing::debug!(component, status = %status, "Component status updated");
    }

    /// Get current health status
    pub fn get_status(&self) -> HealthStatus {
        let components = self.components.read().clone();

        // Overall status is worst component status
        let overall_status = match components {
            _ if components.api == ComponentStatus::Unhealthy
                || components.database == ComponentStatus::Unhealthy
                || components.configuration == ComponentStatus::Unhealthy =>
            {
                ComponentStatus::Unhealthy
            }
            _ if components.api == ComponentStatus::Degraded
                || components.database == ComponentStatus::Degraded
                || components.configuration == ComponentStatus::Degraded =>
            {
                ComponentStatus::Degraded
            }
            _ => ComponentStatus::Healthy,
        };

        let uptime = self.startup_time.elapsed().as_secs_f64();

        HealthStatus {
            status: overall_status,
            timestamp: chrono::Utc::now().to_rfc3339(),
            uptime_seconds: uptime,
            components,
            message: None,
        }
    }

    /// Check if system is ready
    pub fn is_ready(&self) -> bool {
        self.is_ready.load(Ordering::SeqCst)
    }

    /// Check if system is live
    pub fn is_live(&self) -> bool {
        self.is_live.load(Ordering::SeqCst)
    }
}

impl Default for HealthTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_tracker_creation() {
        let tracker = HealthTracker::new();
        assert!(!tracker.is_ready());
        assert!(tracker.is_live());
    }

    #[test]
    fn test_mark_ready() {
        let tracker = HealthTracker::new();
        tracker.mark_ready();
        assert!(tracker.is_ready());
    }

    #[test]
    fn test_component_status_update() {
        let tracker = HealthTracker::new();
        tracker.set_component_status("api", ComponentStatus::Healthy);
        tracker.set_component_status("database", ComponentStatus::Degraded);

        let status = tracker.get_status();
        assert_eq!(status.components.api, ComponentStatus::Healthy);
        assert_eq!(status.components.database, ComponentStatus::Degraded);
    }

    #[test]
    fn test_overall_health_calculation() {
        let tracker = HealthTracker::new();

        // All healthy
        tracker.set_component_status("api", ComponentStatus::Healthy);
        tracker.set_component_status("database", ComponentStatus::Healthy);
        tracker.set_component_status("configuration", ComponentStatus::Healthy);
        tracker.set_component_status("logging", ComponentStatus::Healthy);

        assert_eq!(tracker.get_status().status, ComponentStatus::Healthy);

        // One degraded
        tracker.set_component_status("database", ComponentStatus::Degraded);
        assert_eq!(tracker.get_status().status, ComponentStatus::Degraded);

        // One unhealthy
        tracker.set_component_status("api", ComponentStatus::Unhealthy);
        assert_eq!(tracker.get_status().status, ComponentStatus::Unhealthy);
    }
}
