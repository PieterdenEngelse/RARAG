//! Frontend analytics tracking
//!
//! Tracks:
//! - API calls and performance
//! - Component lifecycle events
//! - User interactions
//! - Errors and exceptions

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Once;

static INIT: Once = Once::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCall {
    pub endpoint: String,
    pub method: String,
    pub status: u16,
    pub duration_ms: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentEvent {
    pub component: String,
    pub event: String, // "mount", "unmount", "error"
    pub timestamp: String,
}

pub struct Analytics;

impl Analytics {
    /// Initialize analytics (call once at app startup)
    pub fn init() {
        INIT.call_once(|| {
            crate::monitoring::logger::Logger::log_info("Analytics initialized");
        });
    }

    /// Track an API call
    ///
    /// Usage:
    /// ```rust,ignore
    /// Analytics::track_api_call("/search", 45.5, 200);
    /// ```
    pub fn track_api_call(endpoint: &str, duration_ms: f64, status: u16) {
        let call = ApiCall {
            endpoint: endpoint.to_string(),
            method: "GET".to_string(), // Will be enhanced in Phase 15
            status,
            duration_ms,
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
        };

        // Log to console
        web_sys::console::log_1(
            &format!(
                "API Call: {} {} {}ms (status: {})",
                call.method, call.endpoint, call.duration_ms, call.status
            )
            .into(),
        );

        // Store in local buffer
        Self::store_api_call(call);
    }

    /// Track component mount
    pub fn track_component_mount(component: &str) {
        let event = ComponentEvent {
            component: component.to_string(),
            event: "mount".to_string(),
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
        };

        Self::log_component_event(&event);
    }

    /// Track component unmount
    pub fn track_component_unmount(component: &str) {
        let event = ComponentEvent {
            component: component.to_string(),
            event: "unmount".to_string(),
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
        };

        Self::log_component_event(&event);
    }

    /// Track component error
    pub fn track_component_error(component: &str, error: &str) {
        crate::monitoring::logger::Logger::error(&format!("{}: {}", component, error));

        let event = ComponentEvent {
            component: component.to_string(),
            event: format!("error: {}", error),
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
        };

        Self::log_component_event(&event);
    }

    /// Log component event
    fn log_component_event(event: &ComponentEvent) {
        web_sys::console::log_1(
            &format!("[{}] {} {}", event.timestamp, event.component, event.event).into(),
        );
    }

    /// Store API call in memory
    fn store_api_call(call: ApiCall) {
        // Store in thread-local or global state
        // Placeholder for Phase 15 enhancement
        let _ = call;
    }

    /// Get statistics
    pub fn get_statistics() -> serde_json::Value {
        json!({
            "api_calls": 0,
            "errors": 0,
            "components": [],
        })
    }

    /// Export analytics as JSON
    pub fn export() -> String {
        Self::get_statistics().to_string()
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_init() {
        Analytics::init();
        Analytics::track_api_call("/search", 45.5, 200);
        Analytics::track_component_mount("SearchComponent");
        Analytics::track_component_unmount("SearchComponent");
    }
}
