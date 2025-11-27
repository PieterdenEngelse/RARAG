use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{info, debug, error, warn};

use ag::api::start_api_server;
use ag::config::ApiConfig;
use ag::retriever::Retriever;
use ag::cache::redis_cache::RedisCache;
use ag::index;
use ag::db::schema_init::SchemaInitializer;
use ag::monitoring::MonitoringConfig;
use ag::monitoring::tracing_config::init_tracing;
use ag::monitoring::metrics;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let startup_instant = Instant::now();
    
    // ─────────────────────────────────────────────────────────────
    // PHASE 1: Load Environment & Initialize Monitoring
    // ─────────────────────────────────────────────────────────────
    
    dotenvy::dotenv().ok();
    
    // Load monitoring config from environment
    let monitoring_config = MonitoringConfig::from_env();

    // Create logs directory
    std::fs::create_dir_all(&monitoring_config.log_dir)
        .expect("Failed to create log directory");
    info!("📝 Log directory: {}", monitoring_config.log_dir.display());

    // Initialize tracing/logging
    let _tracing_guard = init_tracing(&monitoring_config)
        .expect("Failed to initialize tracing");
    
    info!("🚀 Starting agentic-rag v{}", env!("CARGO_PKG_VERSION"));


    // Initialize OpenTelemetry for distributed tracing (Phase 16)
    let otel_config = ag::monitoring::otel_config::OtelConfig::from_env();
    let _otel_guard = ag::monitoring::otel_config::init_otel(&otel_config)
        .expect("Failed to initialize OpenTelemetry");
    info!("🔍 OpenTelemetry initialized");
    debug!("Monitoring enabled: {}", monitoring_config.enabled);
    
    // ─────────────────────────────────────────────────────────────
    // PHASE 1.5: Start Trace-Based Alerting (Background Task)
    // ─────────────────────────────────────────────────────────────
    
    let trace_alert_config = ag::monitoring::TraceAlertingConfig::from_env();
    if trace_alert_config.is_enabled() {
        let _alert_handle = ag::monitoring::start_trace_alerting(trace_alert_config);
        info!("🔔 Trace-based alerting started");
    } else {
        debug!("Trace-based alerting disabled (set TEMPO_ENABLED=true to enable)");
    }
    
    // ─────────────────────────────────────────────────────────────
    // PHASE 1.6: Start Resource Attribution (Background Task)
    // ─────────────────────────────────────────────────────────────
    
    let resource_config = ag::monitoring::ResourceAttributionConfig::from_env();
    if resource_config.is_enabled() {
        let _resource_handle = ag::monitoring::start_resource_attribution(resource_config);
        info!("📊 Resource attribution started");
    } else {
        debug!("Resource attribution disabled (set RESOURCE_ATTRIBUTION_ENABLED=false to disable)");
    }
    
    // ─────────────────────────────────────────────────────────────
    // PHASE 2: Load Configuration with Tracing
    // ─────────────────────────────────────────────────────────────
    debug!("Monitoring config: enabled={}, file_logging={}", 
        monitoring_config.enabled, 
        monitoring_config.enable_file_logging);

    let _config_start = Instant::now();
    debug!("Loading configuration with PathManager...");
    
    let config = ApiConfig::from_env();
    
    let pm = &config.path_manager;
    info!("🏠 AG_HOME: {}", pm.base_dir().display());
    debug!("DB path: {}", pm.db_path("documents").display());
    debug!("Index path: {}", pm.index_path("tantivy").display());
    
    // ─────────────────────────────────────────────────────────────
    // PHASE 3: Initialize Database
    // ─────────────────────────────────────────────────────────────
    
    let db_start = Instant::now();
    info!("📦 Initializing database schema...");
    
    let _db_conn = match (|| -> std::io::Result<Arc<Mutex<rusqlite::Connection>>> {
        let conn = rusqlite::Connection::open(pm.db_path("documents"))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        
        SchemaInitializer::init(&conn)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        
        Ok(Arc::new(Mutex::new(conn)))
    })() {
        Ok(conn) => {
            let duration_ms = db_start.elapsed().as_millis() as u64;
            info!(duration_ms = duration_ms, "✓ Database initialized");
            conn
        }
        Err(e) => {
            error!(error = %e, "Failed to initialize database");
            return Err(e);
        }
    };
    
    // ─────────────────────────────────────────────────────────────
    // PHASE 4: Initialize Retriever with PathManager
    // ─────────────────────────────────────────────────────────────
    
    let retriever_start = Instant::now();
    info!("📦 Initializing Retriever with PathManager...");
    
    let mut retriever = match Retriever::new_with_paths(
        pm.index_path("tantivy"),
        pm.vector_store_path()
    ) {
        Ok(ret) => {
            let duration_ms = retriever_start.elapsed().as_millis() as u64;
            info!(duration_ms = duration_ms, "✓ Retriever initialized");
            // Initialize Prometheus app_info and initial gauges
            metrics::APP_INFO.set(1);
            metrics::refresh_retriever_gauges(&ret);
            ret
        }
        Err(e) => {
            error!(error = %e, "Failed to initialize retriever");
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    };
    
    // ─────────────────────────────────────────────────────────────
    // PHASE 5: Initialize Redis L3 Cache (if enabled)
    // ─────────────────────────────────────────────────────────────
    
    if config.redis_enabled {
        let redis_start = Instant::now();
        info!("📡 Initializing Redis L3 cache...");
        
        match RedisCache::new(
            config.redis_url.as_deref().unwrap_or("redis://127.0.0.1:6379/"),
            config.redis_ttl,
        ).await {
            Ok(redis_cache) => {
                let duration_ms = redis_start.elapsed().as_millis() as u64;
                retriever.set_l3_cache(redis_cache);
                info!(duration_ms = duration_ms, "✅ Redis L3 cache initialized");
            }
            Err(e) => {
                warn!(error = %e, "Failed to initialize Redis L3 cache");
                warn!("Continuing without L3 cache...");
            }
        }
    } else {
        debug!("Redis L3 cache disabled (set REDIS_ENABLED=true to enable)");
    }
    
    // ─────────────────────────────────────────────────────────────
    // PHASE 6: Prepare Retriever for API
    // ─────────────────────────────────────────────────────────────
    
    let retriever = Arc::new(Mutex::new(retriever));
    ag::api::set_retriever_handle(Arc::clone(&retriever));
    
    // ─────────────────────────────────────────────────────────────
    // PHASE 7: Spawn Background Indexing (NON-BLOCKING) - v2.1.0
    // ─────────────────────────────────────────────────────────────

    let skip_initial = config.skip_initial_indexing;

    if skip_initial {
        info!("⏭️  Skipping initial indexing due to SKIP_INITIAL_INDEXING=true");
    } else {
        info!("📚 Starting background indexing (non-blocking)...");
        
        let retriever_clone = Arc::clone(&retriever);
        let upload_dir = ag::api::UPLOAD_DIR.to_string();
        
        // Spawn as background task - doesn't block server startup
        actix_web::rt::spawn(async move {
            let indexing_start = Instant::now();
            debug!("Background indexing task started");
            
            match retriever_clone.lock() {
                Ok(mut ret) => {
                    // Call indexing synchronously within the async task
                    if let Err(e) = index::index_all_documents(&mut *ret, &upload_dir) {
                        error!("Background indexing failed: {}", e);
                    } else {
                        let duration_ms = indexing_start.elapsed().as_millis() as u64;
                        info!(duration_ms = duration_ms, "✓ Background indexing completed");
                        metrics::refresh_retriever_gauges(&ret);
                    }
                }
                Err(e) => {
                    error!("Failed to acquire retriever lock for background indexing: {}", e);
                }
            }
        });
    }
    
    // ─────────────────────────────────────────────────────────────
    // PHASE 8: Start Server Immediately (Server Ready Before Indexing Done)
    // ─────────────────────────────────────────────────────────────
    
    let total_startup_ms = startup_instant.elapsed().as_millis() as u64;
    
    info!("═══════════════════════════════════════════════════════════");
    info!("🎉 Application Started Successfully!");
    info!("   Version: {}", env!("CARGO_PKG_VERSION"));
    info!("   Startup Time: {}ms (server ready, indexing in background)", total_startup_ms);
    metrics::STARTUP_DURATION_MS.set(total_startup_ms as i64);
    info!("   Server: http://{}", config.bind_addr());
    info!("   Health: http://{}/monitoring/health", config.bind_addr());
    info!("   Metrics: http://{}/monitoring/metrics", config.bind_addr());
    info!("   Ready: http://{}/monitoring/ready", config.bind_addr());
    if skip_initial {
        info!("   Note: Initial indexing skipped. Use POST /reindex/async to index.");
    } else {
        info!("   Note: Background indexing in progress. Check /index/info for status.");
    }
    info!("═══════════════════════════════════════════════════════════");
    
    info!("🚀 Starting API server on http://{} ...", config.bind_addr());
    
    start_api_server(&config).await
}