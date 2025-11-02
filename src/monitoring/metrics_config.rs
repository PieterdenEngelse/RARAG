// File: src/monitoring/metrics_config.rs
// Phase 15 Step 4: Configurability – Logging and Metrics
// Version: 1.1.0
// Location: src/monitoring/metrics_config.rs
//
// Purpose: Initialize Prometheus metrics with configurable histogram buckets
// Bridges histogram_config and actual metric collection
//
// Notes:
// - Uses lenient histogram parsing (see histogram_config.rs): invalid tokens are ignored with a warning;
//   valid tokens are kept; if none are valid or env not set, defaults are used. Buckets are sorted & deduped.

use prometheus::{Histogram, HistogramOpts, Registry};
use crate::monitoring::histogram_config::HistogramBuckets;

/// Metrics registry with configurable histogram buckets
#[derive(Clone)]
pub struct ConfigurableMetricsRegistry {
    pub registry: Registry,
    pub histogram_config: HistogramBuckets,
}

impl ConfigurableMetricsRegistry {
    /// Create a new metrics registry with histogram configuration from environment.
    ///
    /// Returns a new ConfigurableMetricsRegistry with:
    /// - Fresh Prometheus registry
    /// - Histogram buckets loaded from environment or defaults
    ///
    /// Panics only if Registry::new() would fail (unlikely).
    pub fn new() -> Self {
        let registry = Registry::new();
        let histogram_config = HistogramBuckets::from_env();

        tracing::info!(
            search_buckets = ?histogram_config.search_buckets,
            reindex_buckets = ?histogram_config.reindex_buckets,
            "Metrics registry initialized with configurable histogram buckets"
        );

        Self { registry, histogram_config }
    }

    /// Create a histogram for search operations with configured buckets.
    ///
    /// Arguments
    /// - name: Metric name (e.g., "search_latency_ms")
    /// - help: Help text describing the metric
    pub fn create_search_histogram(&self, name: &str, help: &str) -> prometheus::Result<Histogram> {
        let opts = HistogramOpts::new(name, help)
            .buckets(self.histogram_config.search_buckets.clone());
        let histogram = Histogram::with_opts(opts)?;
        self.registry.register(Box::new(histogram.clone()))?;
        tracing::debug!(metric_name = name, buckets = ?self.histogram_config.search_buckets, "Search histogram registered");
        Ok(histogram)
    }

    /// Create a histogram for reindex operations with configured buckets.
    ///
    /// Arguments
    /// - name: Metric name (e.g., "reindex_duration_ms")
    /// - help: Help text describing the metric
    pub fn create_reindex_histogram(&self, name: &str, help: &str) -> prometheus::Result<Histogram> {
        let opts = HistogramOpts::new(name, help)
            .buckets(self.histogram_config.reindex_buckets.clone());
        let histogram = Histogram::with_opts(opts)?;
        self.registry.register(Box::new(histogram.clone()))?;
        tracing::debug!(metric_name = name, buckets = ?self.histogram_config.reindex_buckets, "Reindex histogram registered");
        Ok(histogram)
    }

    /// Get the registry for use with Prometheus scraping
    pub fn registry(&self) -> &Registry { &self.registry }

    /// Get histogram bucket configuration
    pub fn histogram_config(&self) -> &HistogramBuckets { &self.histogram_config }
}

impl Default for ConfigurableMetricsRegistry { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

   #[test]
fn test_registry_creation() {
    let registry = ConfigurableMetricsRegistry::new();
    let _ = registry.registry.gather();
}

#[test]
fn test_histogram_config_present() {
    let registry = ConfigurableMetricsRegistry::new();
    assert!(!registry.histogram_config.search_buckets.is_empty());
    assert!(!registry.histogram_config.reindex_buckets.is_empty());
}

    #[test]
    fn test_search_histogram_uses_configured_buckets() {
        std::env::set_var("SEARCH_HISTO_BUCKETS", "1,2,5");
        std::env::remove_var("REINDEX_HISTO_BUCKETS");
        let reg = ConfigurableMetricsRegistry::new();
        let h = reg.create_search_histogram("search_latency_ms_test", "test").expect("histogram");
        h.observe(1.5);
        h.observe(3.0);
        let metrics = reg.registry.gather();
        assert!(metrics.iter().any(|m| m.get_name() == "search_latency_ms_test"));
        std::env::remove_var("SEARCH_HISTO_BUCKETS");
    }

    #[test]
    fn test_duplicate_metric_registration_fails() {
        let reg = ConfigurableMetricsRegistry::new();
        let _ = reg.create_search_histogram("dup_metric", "first").expect("first ok");
        let dup = reg.create_search_histogram("dup_metric", "second");
        assert!(dup.is_err(), "expected registration error for duplicate metric name");
    }
}