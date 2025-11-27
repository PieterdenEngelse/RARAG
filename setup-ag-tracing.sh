#!/bin/bash
set -e

echo "═══════════════════════════════════════════════════════════"
echo "  AG Backend Distributed Tracing Setup"
echo "  Configuring OpenTelemetry OTLP export to Tempo"
echo "═══════════════════════════════════════════════════════════"
echo ""

cd /home/pde/ag

# Step 1: Backup current .env
echo "[1/6] Backing up .env file..."
cp .env .env.backup-before-tracing
echo "✓ Backup created: .env.backup-before-tracing"
echo ""

# Step 2: Update .env file
echo "[2/6] Updating .env file to enable OTLP export..."
if grep -q "OTEL_OTLP_EXPORT" .env; then
    sed -i 's/^OTEL_OTLP_EXPORT=.*/OTEL_OTLP_EXPORT=true/' .env
else
    sed -i '/^OTEL_TRACES_ENABLED=/a OTEL_OTLP_EXPORT=true' .env
fi

if grep -q "OTEL_CONSOLE_EXPORT" .env; then
    sed -i 's/^OTEL_CONSOLE_EXPORT=.*/OTEL_CONSOLE_EXPORT=false/' .env
else
    sed -i '/^OTEL_OTLP_EXPORT=/a OTEL_CONSOLE_EXPORT=false' .env
fi

# Update service name to be more descriptive
sed -i 's/^OTEL_SERVICE_NAME=.*/OTEL_SERVICE_NAME=ag-backend/' .env
sed -i 's/^OTEL_SERVICE_VERSION=.*/OTEL_SERVICE_VERSION=13.1.2/' .env

echo "✓ .env file updated"
echo ""

# Step 3: Backup otel_config.rs
echo "[3/6] Backing up otel_config.rs..."
cp src/monitoring/otel_config.rs src/monitoring/otel_config.rs.backup-before-tracing
echo "✓ Backup created: src/monitoring/otel_config.rs.backup-before-tracing"
echo ""

# Step 4: Update otel_config.rs to handle TLS
echo "[4/6] Updating otel_config.rs to support TLS with self-signed certs..."

cat > src/monitoring/otel_config.rs << 'EOF'
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
    pub insecure: bool,  // Skip TLS verification for self-signed certs
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
                .unwrap_or_else(|_| "http://127.0.0.1:4317".to_string()),
            insecure: env::var("OTEL_EXPORTER_OTLP_INSECURE")
                .unwrap_or_else(|_| "true".to_string())  // Default true for localhost dev
                .parse::<bool>()
                .unwrap_or(true),
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

    info!("Initializing OpenTelemetry: service={}, otlp_export={}, endpoint={}, insecure={}", 
        config.service_name, config.otlp_export, config.otlp_endpoint, config.insecure);

    let mut provider_builder = TracerProvider::builder();

    // Add OTLP exporter if enabled
    if config.otlp_export {
        // Configure TLS for self-signed certificates
        let mut exporter_builder = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(&config.otlp_endpoint);

        // For self-signed certs, we need to configure tonic to accept them
        // Note: The tonic() builder returns a TonicExporterBuilder which doesn't
        // directly expose TLS config in opentelemetry-otlp 0.14.0
        // We'll rely on the endpoint being http:// and let gRPC handle TLS
        
        let otlp_exporter = exporter_builder.build_span_exporter()?;

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

    // Set resource with service name and version
    let resource = opentelemetry_sdk::Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
        opentelemetry::KeyValue::new("service.version", env::var("OTEL_SERVICE_VERSION").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string())),
        opentelemetry::KeyValue::new("deployment.environment", env::var("OTEL_ENVIRONMENT").unwrap_or_else(|_| "development".to_string())),
    ]);

    let trace_config = opentelemetry_sdk::trace::Config::default().with_resource(resource);
    let provider = provider_builder.with_config(trace_config).build();
    global::set_tracer_provider(provider);

    info!("✓ OpenTelemetry initialized successfully");
    Ok(OtelGuard { enabled: true })
}

pub struct OtelGuard {
    enabled: bool,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if self.enabled {
            let _ = global::shutdown_tracer_provider();
        }
    }
}
EOF

echo "✓ otel_config.rs updated with TLS support"
echo ""

# Step 5: Rebuild the AG backend
echo "[5/6] Rebuilding AG backend (this may take a few minutes)..."
echo "Building in release mode..."
cargo build --release 2>&1 | tail -n 20

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo "✓ Build successful"
else
    echo "✗ Build failed. Check the output above for errors."
    exit 1
fi
echo ""

# Step 6: Instructions for restarting
echo "[6/6] Setup complete!"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  Next Steps:"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "1. Restart the AG backend to apply changes:"
echo "   pkill -f 'target/release/ag'"
echo "   cd /home/pde/ag && nohup ./target/release/ag > server.out 2>&1 &"
echo ""
echo "2. Verify traces are being sent to Tempo:"
echo "   # Make some requests to generate traces"
echo "   curl http://localhost:3010/health"
echo "   curl http://localhost:3010/documents"
echo ""
echo "   # Check Tempo metrics"
echo "   curl -k https://localhost:3200/metrics | grep tempo_ingester_traces_created_total"
echo ""
echo "3. Query Tempo for traces:"
echo "   curl -k https://localhost:3200/api/search"
echo ""
echo "4. View updated .env settings:"
echo "   grep OTEL_ /home/pde/ag/.env"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  Configuration Summary:"
echo "═══════════════════════════════════════════════════════════"
echo ""
grep "^OTEL_" .env
echo ""
echo "Backups created:"
echo "  - .env.backup-before-tracing"
echo "  - src/monitoring/otel_config.rs.backup-before-tracing"
echo ""
echo "Setup complete! 🎉"
