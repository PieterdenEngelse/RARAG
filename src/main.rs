// ag/src/main.rs v13.1.2 - UPDATED with PathManager + keep Redis
use std::sync::{Arc, Mutex};
use ag::api::start_api_server;
use ag::config::ApiConfig;
use ag::retriever::Retriever;
use ag::cache::redis_cache::RedisCache;
use ag::index;
use ag::db::schema_init::SchemaInitializer;
use tracing_subscriber;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    
    // Load configuration first (now includes PathManager)
    let config = ApiConfig::from_env()
        .expect("Failed to load configuration");
    
    let pm = &config.path_manager;
    
    println!("🚀 AG_HOME: {}", pm.base_dir().display());
    println!("📦 Initializing database...");
    
    // Initialize database schema
    let db_conn = Arc::new(Mutex::new(
        rusqlite::Connection::open(pm.db_path("documents"))
            .expect("Failed to open database")
    ));
    {
        let conn = db_conn.lock().unwrap();
        SchemaInitializer::init(&conn)
            .expect("Failed to initialize schema");
    }

    println!("📦 Initializing Retriever...");
    
    // Create retriever with PathManager paths (NO hardcoded ./tantivy_index)
    let mut retriever = Retriever::new_with_paths(
        pm.index_path("tantivy"),
        pm.vector_store_path()
    )
    .expect("Failed to initialize retriever");
    
    // Phase 12 Step 3: Initialize Redis L3 cache if enabled
    if config.redis_enabled {
        println!("📡 Initializing Redis L3 cache...");
        match RedisCache::new(
            config.redis_url.as_deref().unwrap_or("redis://127.0.0.1:6379/"),
            config.redis_ttl,
        ).await {
            Ok(redis_cache) => {
                retriever.set_l3_cache(redis_cache);
                println!("✅ Redis L3 cache initialized");
            }
            Err(e) => {
                println!("⚠️ Failed to initialize Redis L3 cache: {}", e);
                println!("Continuing without L3 cache...");
            }
        }
    } else {
        println!("⏭️ Redis L3 cache disabled (set REDIS_ENABLED=true to enable)");
    }
    
    let retriever = Arc::new(Mutex::new(retriever));
    
    // Set global retriever handle for API
    ag::api::set_retriever_handle(Arc::clone(&retriever));
    
    // Optional: reindex all documents at startup
    {
        let mut retriever_guard = retriever.lock().unwrap();
        index::index_all_documents(&mut *retriever_guard, ag::api::UPLOAD_DIR);
    }
    
    println!("🚀 Starting API server on http://{} ...", config.bind_addr());
    start_api_server(&config).await
}