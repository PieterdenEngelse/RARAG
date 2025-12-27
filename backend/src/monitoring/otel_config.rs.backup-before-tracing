use std::env;
use opentelemetry::global;
use opentelemetry_sdk::trace::TracerProvider;
use tracing::info;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime::Tokio as OtelTokioRuntime;

#[derive(Debug, Clone)]
pub struct OtelConfig {
    pub service_name: String,
    pub otlp_export: bool,
    pub console_export: bool,
    pub otlp_endpoint: String,
    /// Master enable switch for OTEL tracing. When false, OTEL is entirely disabled
    /// and no exporters or tracer providers are configured.
    pub enabled: bool,
}

impl OtelConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let enabled = env::var("OTEL_TRACES_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        OtelConfig {
            service_name: env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "agentic-rag".to_string()),
            otlp_export: env::var("OTEL_OTLP_EXPORT")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
            console_export: env::var("OTEL_CONSOLE_EXPORT")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
            otlp_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:4318".to_string()),
            enabled,
        }
    }
}

pub fn init_otel(config: &OtelConfig) -> Result<OtelGuard, Box<dyn std::error::Error>> {
    if !config.enabled {
        // OTEL entirely disabled; return a no-op guard.
        info!("OpenTelemetry disabled via OTEL_TRACES_ENABLED=false");
        return Ok(OtelGuard { enabled: false });
    }

    info!("Initializing OpenTelemetry: service={}, otlp_export={}, endpoint={}", 
        config.service_name, config.otlp_export, config.otlp_endpoint);

    let mut provider_builder = TracerProvider::builder();

    // Add OTLP exporter if enabled
    if config.otlp_export {
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(&config.otlp_endpoint)
            .build_span_exporter()?;

        let batch_processor = opentelemetry_sdk::trace::BatchSpanProcessor::builder(otlp_exporter, OtelTokioRuntime)
            .with_max_export_batch_size(512)
            .build();

        provider_builder = provider_builder.with_span_processor(batch_processor);
        info!("✓ OTLP exporter configured: {}", config.otlp_endpoint);
    }

    // Add console exporter if enabled
    if config.console_export {
        let stdout_exporter = opentelemetry_stdout::SpanExporter::default();
        let batch_processor = opentelemetry_sdk::trace::BatchSpanProcessor::builder(stdout_exporter, OtelTokioRuntime)
            .build();
        
        provider_builder = provider_builder.with_span_processor(batch_processor);
        info!("✓ Console exporter configured");
    }

    // Set resource with service name
    let resource = opentelemetry_sdk::Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
        opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]);

    // In opentelemetry_sdk 0.21, resource is set via Config on the builder
    let trace_config = opentelemetry_sdk::trace::Config::default().with_resource(resource);
    let provider = provider_builder.with_config(trace_config).build();
    global::set_tracer_provider(provider);

    info!("✓ OpenTelemetry initialized");
    Ok(OtelGuard { enabled: true })
}

pub struct OtelGuard {
    enabled: bool,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        // Best-effort shutdown: only attempt to flush spans if OTEL was actually enabled.
        // This avoids surprising work during process teardown when tracing was disabled
        // via OTEL_TRACES_ENABLED=false.
        if self.enabled {
            let _ = global::shutdown_tracer_provider();
        }
    }
}