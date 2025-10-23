use std::sync::{Arc, Mutex};
use ag::api::start_api_server;
use ag::config::ApiConfig; // ← added
use ag::retriever::Retriever;
use ag::index;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    // Load configuration first
    let config = ApiConfig::from_env();

    println!("📦 Initializing Retriever...");
    
    // Create retriever and handle potential errors
    let retriever = Arc::new(Mutex::new(
        Retriever::new("./tantivy_index")
            .expect("Failed to initialize retriever")
    ));
    
    // Set global retriever handle for API
    ag::api::set_retriever_handle(Arc::clone(&retriever));
    
    // Optional: reindex all documents at startup
    {
        let mut retriever_guard = retriever.lock().unwrap();
        index::index_all_documents(&mut *retriever_guard, ag::api::UPLOAD_DIR);
    }
    
    println!("🚀 Starting API server on http://{} ...", config.bind_addr());
    start_api_server(&config).await // ← now passes config
}