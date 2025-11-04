use std::{env, time::Duration};
use tokio::time::sleep;

#[tokio::test]
async fn rate_limit_per_ip_token_bucket() {
    // Choose a port; if busy, adjust
    let port: u16 = 40123;
    env::set_var("BACKEND_HOST", "127.0.0.1");
    env::set_var("BACKEND_PORT", port.to_string());
    env::set_var("RATE_LIMIT_ENABLED", "true");
    env::set_var("RATE_LIMIT_QPS", "1");
    env::set_var("RATE_LIMIT_BURST", "3");
    env::set_var("SKIP_INITIAL_INDEXING", "true");

    // Start server in background
    tokio::spawn(async move {
        let config = ag::config::ApiConfig::from_env();
        let pm = &config.path_manager;
        let retriever = ag::Retriever::new_with_paths(
            pm.index_path("tantivy"),
            pm.vector_store_path()
        ).expect("retriever init");
        let retriever = std::sync::Arc::new(std::sync::Mutex::new(retriever));
        ag::api::set_retriever_handle(std::sync::Arc::clone(&retriever));
        ag::api::start_api_server(&config).await.unwrap();
    });

    // Wait a bit for server to bind
    sleep(Duration::from_millis(800)).await;

    let client = reqwest::Client::new();

    // Health warmup
    let _ = client.get(format!("http://127.0.0.1:{}/health", port)).send().await.unwrap();

    // Fire 8 requests
    let mut codes = Vec::new();
    for _ in 0..8 {
        let resp = client
            .get(format!("http://127.0.0.1:{}/search?q=hi", port))
            .send().await.unwrap();
        codes.push(resp.status().as_u16());
    }

    assert_eq!(codes[0], 200);
    assert_eq!(codes[1], 200);
    assert_eq!(codes[2], 200);
    for i in 3..codes.len() {
        assert_eq!(codes[i], 429, "Unexpected code at {}: {:?}", i, codes);
    }
}