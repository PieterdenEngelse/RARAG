// examples/complete_demo.rs
// Complete end-to-end demo: ingest data → store → query → answer

use ag::memory::{VectorStore, VectorRecord, RagQueryPipeline, RagQueryRequest, RagConfig};
use ag::embedder::{EmbeddingService, EmbeddingConfig};
use ag::memory::MockLLMProvider;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("\n🚀 RAG Demo: Complete End-to-End Pipeline\n");

    // Step 1: Create services
    println!("Step 1: Initializing services...");
    let embedding_service = Arc::new(EmbeddingService::new(
        EmbeddingConfig::default(),
    ));

    let vector_store = Arc::new(RwLock::new(
        VectorStore::with_defaults()?
    ));

    let llm_provider = Arc::new(MockLLMProvider::new());

    // Step 2: Create sample documents
    println!("Step 2: Creating sample documents...\n");
    let documents = vec![
        (
            "doc_1",
            "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.",
        ),
        (
            "doc_2",
            "The Rust compiler is incredibly helpful and catches many bugs at compile time before they become runtime errors.",
        ),
        (
            "doc_3",
            "Cargo is Rust's package manager and build system. It makes managing dependencies and building projects simple.",
        ),
        (
            "doc_4",
            "Tokio is an async runtime for Rust that enables writing fast, reliable, and scalable network applications.",
        ),
        (
            "doc_5",
            "Memory safety is one of Rust's core principles. The borrow checker ensures memory is managed safely without a garbage collector.",
        ),
    ];

    // Step 3: Embed and store documents
    println!("Step 3: Embedding and storing documents...");
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
            println!("  ✓ Stored: {} ({})", doc_id, content.len());
        }
        
        let stats = store.stats().await;
        println!("\nVector Store Stats:");
        println!("  Total records: {}", stats.total_records);
        println!("  Max capacity: {}", stats.max_vectors);
        println!("  Utilization: {:.2}%", stats.utilization * 100.0);
    }

    // Step 4: Create RAG pipeline
    println!("\nStep 4: Creating RAG pipeline...");
    let rag_pipeline = RagQueryPipeline::new(
        embedding_service.clone(),
        vector_store.clone(),
        llm_provider,
        RagConfig::default(),
    );

    // Step 5: Run queries
    println!("\nStep 5: Running queries...\n");
    
    let queries = vec![
        "What is Rust?",
        "Tell me about Cargo",
        "Explain memory safety in Rust",
        "What is Tokio?",
    ];

    for query in queries {
        println!("Query: {}", query);
        println!("{}", "-".repeat(60));
        
        let req = RagQueryRequest {
            query: query.to_string(),
            top_k: 3,
            include_sources: true,
        };

        match rag_pipeline.query(&req).await {
            Ok(response) => {
                println!("Answer: {}\n", response.answer);
                println!("Context chunks used: {}", response.total_chunks_used);
                println!("Sources: {:?}", response.sources);
                println!("Chunks:");
                for chunk in &response.context_chunks {
                    println!("  - {} (similarity: {:.3})", chunk.chunk_id, chunk.similarity_score);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        println!();
    }

    // Step 6: Final metrics
    println!("Step 6: Final Metrics");
    println!("{}", "-".repeat(60));
    {
        let store = vector_store.read().await;
        let stats = store.stats().await;
        let metrics = &stats.metrics;
        
        println!("Total insertions: {}", metrics.total_insertions);
        println!("Total evictions: {}", metrics.total_evictions);
        println!("Lookup hits: {}", metrics.lookup_hits);
        println!("Lookup misses: {}", metrics.lookup_misses);
        println!("Hit rate: {:.2}%", metrics.hit_rate() * 100.0);
        println!("Peak vectors: {}", metrics.peak_vectors);
    }

    println!("\n✅ Demo completed successfully!\n");
    Ok(())
}