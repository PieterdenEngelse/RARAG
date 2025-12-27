// File: src/monitoring/performance_analysis.rs
// Phase 16: Distributed Tracing - Performance Analysis
// Version: 1.0.0
// Location: src/monitoring/performance_analysis.rs
//
// Purpose: Track and analyze operation performance metrics
// Provides percentiles, latency analysis, bottleneck detection

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Operation performance metrics
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub operation: String,
    pub duration_ms: u64,
    pub timestamp: u64,
    pub status: OperationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationStatus {
    Success,
    Error,
    Timeout,
}

/// Performance analyzer for latency tracking
pub struct PerformanceAnalyzer {
    metrics: Arc<RwLock<HashMap<String, Vec<OperationMetrics>>>>,
    max_samples: usize,
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    ///
    /// # Arguments
    /// * `max_samples` - Maximum samples per operation (default: 10000)
    pub fn new(max_samples: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            max_samples,
        }
    }

    /// Record an operation's performance
    pub fn record(&self, operation: &str, duration: Duration, status: OperationStatus) {
        let metric = OperationMetrics {
            operation: operation.to_string(),
            duration_ms: duration.as_millis() as u64,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            status,
        };

        if let Ok(mut metrics) = self.metrics.write() {
            let samples = metrics
                .entry(operation.to_string())
                .or_insert_with(Vec::new);

            // Keep only recent samples
            if samples.len() >= self.max_samples {
                samples.remove(0);
            }

            samples.push(metric);
        }
    }

    /// Get percentile latency for an operation
    ///
    /// # Arguments
    /// * `operation` - Operation name
    /// * `percentile` - Percentile (0-100)
    pub fn percentile(&self, operation: &str, percentile: f64) -> Option<u64> {
        if let Ok(metrics) = self.metrics.read() {
            if let Some(samples) = metrics.get(operation) {
                let mut durations: Vec<u64> = samples.iter().map(|m| m.duration_ms).collect();

                if durations.is_empty() {
                    return None;
                }

                durations.sort_unstable();
                let index = ((percentile / 100.0) * (durations.len() - 1) as f64) as usize;
                return Some(durations[index]);
            }
        }
        None
    }

    /// Get average latency for an operation
    pub fn average(&self, operation: &str) -> Option<u64> {
        if let Ok(metrics) = self.metrics.read() {
            if let Some(samples) = metrics.get(operation) {
                if samples.is_empty() {
                    return None;
                }
                let sum: u64 = samples.iter().map(|m| m.duration_ms).sum();
                return Some(sum / samples.len() as u64);
            }
        }
        None
    }

    /// Get min latency for an operation
    pub fn min(&self, operation: &str) -> Option<u64> {
        if let Ok(metrics) = self.metrics.read() {
            if let Some(samples) = metrics.get(operation) {
                return samples.iter().map(|m| m.duration_ms).min();
            }
        }
        None
    }

    /// Get max latency for an operation
    pub fn max(&self, operation: &str) -> Option<u64> {
        if let Ok(metrics) = self.metrics.read() {
            if let Some(samples) = metrics.get(operation) {
                return samples.iter().map(|m| m.duration_ms).max();
            }
        }
        None
    }

    /// Get error rate for an operation (0.0-1.0)
    pub fn error_rate(&self, operation: &str) -> Option<f64> {
        if let Ok(metrics) = self.metrics.read() {
            if let Some(samples) = metrics.get(operation) {
                if samples.is_empty() {
                    return None;
                }
                let errors = samples
                    .iter()
                    .filter(|m| m.status != OperationStatus::Success)
                    .count();
                return Some(errors as f64 / samples.len() as f64);
            }
        }
        None
    }

    /// Get performance summary for an operation
    pub fn summary(&self, operation: &str) -> Option<PerformanceSummary> {
        Some(PerformanceSummary {
            operation: operation.to_string(),
            count: self.count(operation)?,
            min_ms: self.min(operation)?,
            max_ms: self.max(operation)?,
            avg_ms: self.average(operation)?,
            p50_ms: self.percentile(operation, 50.0)?,
            p95_ms: self.percentile(operation, 95.0)?,
            p99_ms: self.percentile(operation, 99.0)?,
            error_rate: self.error_rate(operation)?,
        })
    }

    /// Get sample count for an operation
    fn count(&self, operation: &str) -> Option<usize> {
        if let Ok(metrics) = self.metrics.read() {
            return metrics.get(operation).map(|s| s.len());
        }
        None
    }

    /// Clear all metrics
    pub fn clear(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.clear();
        }
    }

    /// Get all operation names
    pub fn operations(&self) -> Vec<String> {
        if let Ok(metrics) = self.metrics.read() {
            return metrics.keys().cloned().collect();
        }
        Vec::new()
    }
}

/// Performance summary for an operation
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub operation: String,
    pub count: usize,
    pub min_ms: u64,
    pub max_ms: u64,
    pub avg_ms: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub error_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = PerformanceAnalyzer::new(1000);
        assert_eq!(analyzer.operations().len(), 0);
    }

    #[test]
    fn test_record_operation() {
        let analyzer = PerformanceAnalyzer::new(1000);
        analyzer.record(
            "search",
            Duration::from_millis(50),
            OperationStatus::Success,
        );
        analyzer.record(
            "search",
            Duration::from_millis(100),
            OperationStatus::Success,
        );

        assert_eq!(analyzer.count("search"), Some(2));
    }

    #[test]
    fn test_average_latency() {
        let analyzer = PerformanceAnalyzer::new(1000);
        analyzer.record(
            "search",
            Duration::from_millis(50),
            OperationStatus::Success,
        );
        analyzer.record(
            "search",
            Duration::from_millis(100),
            OperationStatus::Success,
        );

        assert_eq!(analyzer.average("search"), Some(75));
    }

    #[test]
    fn test_percentile_latency() {
        let analyzer = PerformanceAnalyzer::new(1000);
        for i in 1..=100 {
            analyzer.record("search", Duration::from_millis(i), OperationStatus::Success);
        }

        assert_eq!(analyzer.percentile("search", 50.0), Some(50));
        assert_eq!(analyzer.percentile("search", 95.0), Some(95));
        assert_eq!(analyzer.percentile("search", 99.0), Some(99));
    }

    #[test]
    fn test_error_rate() {
        let analyzer = PerformanceAnalyzer::new(1000);
        analyzer.record(
            "search",
            Duration::from_millis(50),
            OperationStatus::Success,
        );
        analyzer.record("search", Duration::from_millis(100), OperationStatus::Error);
        analyzer.record(
            "search",
            Duration::from_millis(75),
            OperationStatus::Success,
        );

        let error_rate = analyzer.error_rate("search").unwrap();
        assert!((error_rate - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_min_max() {
        let analyzer = PerformanceAnalyzer::new(1000);
        analyzer.record(
            "search",
            Duration::from_millis(50),
            OperationStatus::Success,
        );
        analyzer.record(
            "search",
            Duration::from_millis(150),
            OperationStatus::Success,
        );
        analyzer.record(
            "search",
            Duration::from_millis(100),
            OperationStatus::Success,
        );

        assert_eq!(analyzer.min("search"), Some(50));
        assert_eq!(analyzer.max("search"), Some(150));
    }

    #[test]
    fn test_clear_metrics() {
        let analyzer = PerformanceAnalyzer::new(1000);
        analyzer.record(
            "search",
            Duration::from_millis(50),
            OperationStatus::Success,
        );
        assert_eq!(analyzer.count("search"), Some(1));

        analyzer.clear();
        assert_eq!(analyzer.count("search"), None);
    }
}
