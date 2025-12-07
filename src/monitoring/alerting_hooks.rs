// File: src/monitoring/alerting_hooks.rs
// Phase 15 Step 5: Alerting Hooks (Optional)
// Version: 1.0.0
// Location: src/monitoring/alerting_hooks.rs
//
// Purpose: Send webhook notifications on reindex completion
// Non-blocking alerts for operational monitoring

use serde_json::json;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

/// Webhook configuration for reindex alerts
#[derive(Debug, Clone)]
pub struct AlertingHooksConfig {
    /// Webhook URL from REINDEX_WEBHOOK_URL env var
    pub webhook_url: Option<String>,
    /// Whether alerting is enabled
    pub enabled: bool,
}

impl AlertingHooksConfig {
    /// Load alerting hooks configuration from environment
    ///
    /// Environment variables:
    /// - `REINDEX_WEBHOOK_URL`: Full URL to receive webhook POST requests
    ///   Example: `REINDEX_WEBHOOK_URL=https://example.com/alerts/reindex`
    ///   If not set, alerting is disabled (no-op)
    ///
    /// # Returns
    /// Configuration with webhook URL if set, otherwise disabled
    pub fn from_env() -> Self {
        let webhook_url = env::var("REINDEX_WEBHOOK_URL").ok();
        let enabled = webhook_url.is_some();

        if enabled {
            tracing::info!(
                webhook_url = ?webhook_url,
                "Reindex alerting hooks enabled"
            );
        } else {
            tracing::debug!("Reindex alerting hooks disabled (REINDEX_WEBHOOK_URL not set)");
        }

        Self {
            webhook_url,
            enabled,
        }
    }

    /// Check if alerting is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Reindex completion event for webhook payload
#[derive(Debug, Clone)]
pub struct ReindexCompletionEvent {
    /// "success" or "error"
    pub status: String,
    /// Operation duration in milliseconds
    pub duration_ms: u64,
    /// Number of vectors indexed
    pub vectors: u64,
    /// Number of document mappings
    pub mappings: u64,
    /// Unix timestamp of completion
    pub timestamp: u64,
}

impl ReindexCompletionEvent {
    /// Create a successful reindex event
    pub fn success(duration_ms: u64, vectors: u64, mappings: u64) -> Self {
        Self {
            status: "success".to_string(),
            duration_ms,
            vectors,
            mappings,
            timestamp: current_timestamp(),
        }
    }

    /// Create a failed reindex event
    pub fn error(duration_ms: u64, vectors: u64, mappings: u64) -> Self {
        Self {
            status: "error".to_string(),
            duration_ms,
            vectors,
            mappings,
            timestamp: current_timestamp(),
        }
    }

    /// Convert to JSON payload for webhook
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "status": self.status,
            "duration_ms": self.duration_ms,
            "vectors": self.vectors,
            "mappings": self.mappings,
            "timestamp": self.timestamp,
        })
    }
}

/// Send webhook alert (non-blocking)
///
/// # Arguments
/// * `config` - Alerting hooks configuration
/// * `event` - Reindex completion event
///
/// Spawns async task to send webhook. Failures are logged as warnings.
/// Never blocks or fails the calling request.
pub async fn send_alert(config: &AlertingHooksConfig, event: ReindexCompletionEvent) {
    if !config.is_enabled() {
        tracing::debug!("Alerting disabled, skipping webhook");
        return;
    }

    let webhook_url = match config.webhook_url.clone() {
        Some(url) => url,
        None => {
            tracing::warn!("Alerting enabled but no webhook URL configured");
            return;
        }
    };

    let payload = event.to_json();
    let status = event.status.clone();
    let duration_ms = event.duration_ms;

    // Spawn non-blocking task
    tokio::spawn(async move {
        match send_webhook(&webhook_url, &payload).await {
            Ok(_) => {
                tracing::info!(
                    webhook_url = %webhook_url,
                    status = %status,
                    duration_ms = duration_ms,
                    "Reindex alert webhook sent successfully"
                );
            }
            Err(e) => {
                tracing::warn!(
                    webhook_url = %webhook_url,
                    status = %status,
                    error = ?e,
                    "Failed to send reindex alert webhook (non-fatal)"
                );
            }
        }
    });
}

/// Internal: Actually send the webhook request
async fn send_webhook(
    url: &str,
    payload: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(payload)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "HTTP {}: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        )
        .into())
    }
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_disabled_by_default() {
        std::env::remove_var("REINDEX_WEBHOOK_URL");
        let config = AlertingHooksConfig::from_env();
        assert!(!config.is_enabled());
        assert!(config.webhook_url.is_none());
    }

    #[test]
    fn test_config_enabled_when_url_set() {
        std::env::set_var("REINDEX_WEBHOOK_URL", "https://example.com/alert");
        let config = AlertingHooksConfig::from_env();
        assert!(config.is_enabled());
        assert_eq!(
            config.webhook_url,
            Some("https://example.com/alert".to_string())
        );
        std::env::remove_var("REINDEX_WEBHOOK_URL");
    }

    #[test]
    fn test_success_event() {
        let event = ReindexCompletionEvent::success(1234, 5000, 100);
        assert_eq!(event.status, "success");
        assert_eq!(event.duration_ms, 1234);
        assert_eq!(event.vectors, 5000);
        assert_eq!(event.mappings, 100);
    }

    #[test]
    fn test_error_event() {
        let event = ReindexCompletionEvent::error(567, 2000, 50);
        assert_eq!(event.status, "error");
        assert_eq!(event.duration_ms, 567);
    }

    #[test]
    fn test_event_to_json() {
        let event = ReindexCompletionEvent::success(1000, 5000, 100);
        let json = event.to_json();
        assert_eq!(json["status"], "success");
        assert_eq!(json["duration_ms"], 1000);
        assert_eq!(json["vectors"], 5000);
        assert_eq!(json["mappings"], 100);
        assert!(json["timestamp"].is_u64());
    }

    #[test]
    fn test_timestamp_is_positive() {
        let event = ReindexCompletionEvent::success(0, 0, 0);
        assert!(event.timestamp > 0);
    }
}
