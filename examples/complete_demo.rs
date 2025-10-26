// examples/complete_demo.rs
use ag::memory::{VectorStore, VectorRecord, RagQueryPipeline, RagQueryRequest, RagConfig, LLMProvider};
use ag::embedder::{EmbeddingService, EmbeddingConfig};
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

struct DemoLLM;

#[async_trait]
impl LLMProvider for DemoLLM {
    async fn generate(&self, _prompt: &str) -> Result<String, ag::memory::LLMError> {
        Ok("Demo answer based on context.".to_string())
    }
    fn model_name(&self) -> &str { "demo" }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    println!("\n🚀 RAG Demo\n");

    let embedding_service = Arc::new(EmbeddingService::new(EmbeddingConfig::default()));
    let vector_store = Arc::new(RwLock::new(VectorStore::with_defaults()?));
    let llm_provider = Arc::new(DemoLLM);

    let documents = vec![
        ("doc_1", "Rust is a systems programming language."),
        ("doc_2", "Cargo is the package manager."),
        ("doc_3", "Memory safety is core."),
    ];

    {
        let mut store = vector_store.write().await;
        for (doc_id, content) in &documents {
            let embedding = embedding_service.embed_text(content).await;
            let record = VectorRecord::new(
                format!("chunk_{}", doc_id),
                doc_id.to_string(),
                content.to_string(),
                embedding,
                0,
                content.split_whitespace().count(),
                "demo".to_string(),
                chrono::Utc::now().timestamp(),
            );
            store.add_record(record).await?;
            println!("  ✓ Stored: {}", doc_id);
        }
        let stats = store.stats().await;
        println!("\nRecords: {}", stats.total_records);
    }

    let rag_pipeline = RagQueryPipeline::new(
        embedding_service.clone(),
        vector_store.clone(),
        llm_provider,
        RagConfig::default(),
    );

    let req = RagQueryRequest {
        query: "What is Rust?".to_string(),
        top_k: 3,
        include_sources: true,
    };

    match rag_pipeline.query(&req).await {
        Ok(response) => println!("Answer: {}", response.answer),
        Err(e) => println!("Error: {}", e),
    }

    println!("\n✅ Demo complete!\n");
    Ok(())
}
