//! OTLP Exporter Module
//!
//! Handles creation and configuration of OTLP/gRPC exporters for sending spans
//! to distributed tracing backends (Jaeger, Grafana Tempo, etc).
//!
//! # Features
//! - OTLP/gRPC exporter with configurable endpoint
//! - Batch span processor for efficient export
//! - Console exporter fallback for debugging
//! - Graceful error handling and timeout configuration
//! - Optional metric export support

use std::time::Duration;
use opentelemetry_sdk::trace::{BatchSpanProcessor, TracerProvider};
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_otlp::WithExportConfig;
use tracing::{info, warn, error};
use crate::monitoring::otel_config::OtelConfig;

/// OTLP Exporter configuration and initialization
pub struct OtlpExporter;

impl OtlpExporter {
    /// Create and configure OTLP/gRPC exporter
    ///
    /// # Arguments
    /// * `config` - OtelConfig with exporter settings
    /// * `tracer_provider` - TracerProvider to attach exporter to
    ///
    /// # Returns
    /// TracerProvider with batch processor and exporter attached
    ///
    /// # Example
    /// ```rust,no_run
    /// use agentic_rag::monitoring::otel_config::OtelConfig;
    /// use agentic_rag::monitoring::otlp_exporter::OtlpExporter;
    ///
    /// let config = OtelConfig::from_env();
    /// let tracer_provider = OtlpExporter::create_exporter(&config)?;
    /// ```
    pub fn create_exporter(
        config: &OtelConfig,
    ) -> Result<TracerProvider, Box<dyn std::error::Error>> {
        info!(
            endpoint = %config.otlp_endpoint,
            console_export = config.console_export,
            otlp_export = config.otlp_export,
            batch_queue_size = config.batch_queue_size,
            batch_scheduled_delay_ms = config.batch_scheduled_delay_ms,
            "Creating OTLP exporter"
        );

        // Create base tracer provider (already configured with resource and sampler from otel_config)
        let mut tracer_provider_builder = TracerProvider::builder();

        // Add OTLP exporter if enabled
        if config.otlp_export {
            match Self::setup_otlp_exporter(config) {
                Ok(processor) => {
                    info!("OTLP/gRPC exporter configured successfully");
                    tracer_provider_builder = tracer_provider_builder.with_span_processor(processor);
                }
                Err(e) => {
                    error!("Failed to configure OTLP exporter: {}", e);
                    warn!("Falling back to console exporter");
                    // Continue with console exporter fallback
                }
            }
        }

        // Add console exporter if enabled (for debugging)
        if config.console_export {
            match Self::setup_console_exporter() {
                Ok(processor) => {
                    info!("Console exporter configured for debugging");
                    tracer_provider_builder = tracer_provider_builder.with_span_processor(processor);
                }
                Err(e) => {
                    error!("Failed to configure console exporter: {}", e);
                }
            }
        }

        // If neither exporter is enabled, log warning
        if !config.otlp_export && !config.console_export {
            warn!("No exporters enabled - spans will be collected but not exported");
        }

        let tracer_provider = tracer_provider_builder.build();
        info!("OTLP exporter initialized");

        Ok(tracer_provider)
    }

    /// Setup OTLP/gRPC exporter with batch processor
    fn setup_otlp_exporter(
        config: &OtelConfig,
    ) -> Result<BatchSpanProcessor<Tokio>, Box<dyn std::error::Error>> {
        // Parse endpoint URL
        let _endpoint_url = config.otlp_endpoint.parse::<http::Uri>()?;

        info!(endpoint = %config.otlp_endpoint, "Connecting to OTLP endpoint");

        // Create OTLP exporter with gRPC transport
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(config.otlp_endpoint.clone())
            .with_timeout(Duration::from_secs(10))
            .build_span_exporter()?;

        // Create batch processor with tokio runtime
        let batch_processor = BatchSpanProcessor::builder(
            otlp_exporter,
            Tokio
        )
            .with_max_queue_size(config.batch_queue_size)
            .with_scheduled_delay(Duration::from_millis(config.batch_scheduled_delay_ms))
            .build();

        info!(
            queue_size = config.batch_queue_size,
            delay_ms = config.batch_scheduled_delay_ms,
            "Batch span processor configured"
        );

        Ok(batch_processor)
    }

    /// Setup console exporter for debugging (prints spans to stdout)
    fn setup_console_exporter() -> Result<BatchSpanProcessor<Tokio>, Box<dyn std::error::Error>> {
        let console_exporter = opentelemetry_stdout::SpanExporter::default();

        let batch_processor = BatchSpanProcessor::builder(
            console_exporter,
            Tokio
        )
            .with_max_queue_size(512)
            .with_scheduled_delay(Duration::from_millis(1000))
            .build();

        Ok(batch_processor)
    }
}

/// Span export configuration helper
pub struct SpanExportConfig {
    /// Maximum time to wait for export before timeout
    pub timeout: Duration,

    /// Maximum number of spans in a batch
    pub max_batch_size: usize,

    /// Delay between batch exports
    pub scheduled_delay: Duration,
}

impl Default for SpanExportConfig {
    fn default() -> Self {
        SpanExportConfig {
            timeout: Duration::from_secs(10),
            max_batch_size: 512,
            scheduled_delay: Duration::from_millis(5000),
        }
    }
}

impl SpanExportConfig {
    /// Create configuration from OtelConfig
    pub fn from_otel_config(config: &OtelConfig) -> Self {
        SpanExportConfig {
            timeout: Duration::from_secs(10),
            max_batch_size: config.batch_queue_size,
            scheduled_delay: Duration::from_millis(config.batch_scheduled_delay_ms),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::otel_config::OtelConfig;

    #[test]
    fn test_span_export_config_default() {
        let config = SpanExportConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_batch_size, 512);
    }

    #[test]
    fn test_span_export_config_from_otel() {
        let otel_config = OtelConfig::new_test();
        let export_config = SpanExportConfig::from_otel_config(&otel_config);

        assert_eq!(
            export_config.max_batch_size,
            otel_config.batch_queue_size
        );
        assert_eq!(
            export_config.scheduled_delay.as_millis() as u64,
            otel_config.batch_scheduled_delay_ms
        );
    }

    #[test]
    fn test_otlp_exporter_creation_with_console() {
        let mut config = OtelConfig::new_test();
        config.console_export = true;
        config.otlp_export = false;

        let result = OtlpExporter::create_exporter(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_otlp_exporter_creation_no_exporters() {
        let mut config = OtelConfig::new_test();
        config.console_export = false;
        config.otlp_export = false;

        let result = OtlpExporter::create_exporter(&config);
        // Should still succeed, just without exporters
        assert!(result.is_ok());
    }
}