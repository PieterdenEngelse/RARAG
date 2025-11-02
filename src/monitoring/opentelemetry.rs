use std::env;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .json();

    // Optional: OTLP endpoint (default key)
    let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

    #[cfg(feature = "otel")] // guard if you later add a feature flag
    let otlp_layer = {
        use opentelemetry::sdk::export::trace::SpanExporter;
        use opentelemetry::sdk::trace as sdktrace;
        use opentelemetry_otlp::WithExportConfig;

        if let Some(endpoint) = otlp_endpoint.clone() {
            let exporter = opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(endpoint);
            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .install_simple()
                .ok();

            tracer.map(|t| tracing_opentelemetry::layer().with_tracer(t))
        } else {
            None
        }
    };

    #[cfg(not(feature = "otel"))]
    let otlp_layer: Option<tracing_opentelemetry::OpenTelemetryLayer<
        tracing_subscriber::Registry,
        opentelemetry::sdk::trace::Tracer,
    >> = None;

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(otlp_layer);

    subscriber.init();
}
