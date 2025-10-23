// examples/agent_example.rs
// End-to-End Agent Example - Phases 1-7 in action
// 
// This example demonstrates:
// Phase 1: Chunking documents
// Phase 2: Embedding with caching
// Phase 3: Vector store
// Phase 5: RAG query pipeline with Phi 3.5
// Phase 6: Agent memory (goals, episodes, reflections)
// Phase 7: Decision engine (autonomous execution)

use ag::embedder::{EmbeddingService, EmbeddingConfig};
use ag::memory::{
    VectorStore, RagQueryPipeline, RagConfig,
    AgentMemoryLayer, DecisionEngine, LLMConfig, create_llm_provider
};
use std::sync::Arc;
use tempfile::NamedTempFile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Agent Example - Phases 1-7\n");

    // ============ Setup ============
    println!("📚 Setting up agent infrastructure...\n");

    // Phase 2: Embedding service with caching
    let embedding_service = Arc::new(EmbeddingService::new(EmbeddingConfig::default()));
    println!("✓ Embedding service initialized (with LRU cache)");

    // Phase 3: Vector store
    let vector_store = Arc::new(tokio::sync::RwLock::new(
        VectorStore::with_defaults()?
    ));
    println!("✓ Vector store initialized");

    // Phase 5: LLM provider (Phi 3.5 via Ollama)
    let llm_config = LLMConfig::default(); // Phi 3.5
    let llm_provider_box = create_llm_provider(llm_config).await?;
    let llm_provider: Arc<dyn ag::memory::LLMProvider> = llm_provider_box.into();
    println!("✓ LLM provider ready: {}", llm_provider.model_name());

    // Phase 5: RAG Query Pipeline
    let rag_pipeline = Arc::new(RagQueryPipeline::new(
        embedding_service.clone(),
        vector_store.clone(),
        llm_provider,
        RagConfig::default(),
    ));
    println!("✓ RAG query pipeline initialized\n");

    // Phase 6: Agent Memory Layer
    let temp_db = NamedTempFile::new()?;
    let agent_memory = Arc::new(AgentMemoryLayer::new(
        "example-agent".to_string(),
        "Example Agent".to_string(),
        temp_db.path().to_path_buf(),
        vector_store.clone(),
        embedding_service.clone(),
    )?);
    println!("✓ Agent memory layer initialized");
    println!("✓ Agent ID: example-agent\n");

    // Phase 7: Decision Engine
    let decision_engine = DecisionEngine::new(rag_pipeline, agent_memory.clone());
    println!("✓ Decision engine ready\n");

    // ============ Agent Execution ============
    println!("🎯 Agent Execution Example\n");

    // Set a goal
    println!("1️⃣  Setting goal...");
    let goal = agent_memory.set_goal(
        "Learn about Rust programming language".to_string()
    )?;
    println!("   Goal ID: {}", goal.id);
    println!("   Goal: {}\n", goal.goal);

    // Execute query with decision engine
    println!("2️⃣  Executing query with autonomous decision making...");
    println!("   Query: 'What is Rust?'\n");

    match decision_engine
        .execute_query("What is Rust?", Some(goal.id.clone()))
        .await
    {
        Ok(result) => {
            println!("   ✓ Execution completed successfully\n");
            println!("   Reasoning Trace:");
            for (i, trace) in result.reasoning_trace.iter().enumerate() {
                println!("     {}. {}", i + 1, trace);
            }
            println!("\n   Answer:");
            println!("   {}\n", result.answer);
            println!("   Steps executed: {}", result.steps_executed);
            println!("   Success: {}\n", result.success);
        }
        Err(e) => {
            println!("   ❌ Error during execution: {}\n", e);
            println!("   (Note: Ensure Ollama is running with: ollama serve)\n");
        }
    }

    // ============ Agent Memory Inspection ============
    println!("3️⃣  Inspecting agent memory...\n");

    // Get agent context
    let context = agent_memory.get_agent_context()?;
    println!("   Active goals: {}", context.active_goals.len());
    for goal in &context.active_goals {
        println!("     - {}: {}", goal.id, goal.goal);
    }

    println!("\n   Recent episodes: {}", context.recent_episodes.len());
    for episode in &context.recent_episodes {
        println!("     - Query: {}", episode.query);
        println!("       Success: {}", episode.success);
    }

    println!("\n   Recent reflections: {}", context.recent_reflections.len());
    for reflection in &context.recent_reflections {
        println!("     - Type: {}", reflection.reflection_type);
        println!("       Insight: {}", reflection.insight);
    }

    // ============ Memory Learning ============
    println!("\n4️⃣  Testing memory learning...\n");

    // Ask a similar question
    println!("   Finding similar past queries for: 'Tell me about Rust'\n");
    let similar = agent_memory
        .recall_similar_episodes("Tell me about Rust", 3)
        .await?;

    if !similar.is_empty() {
        println!("   Found {} similar query(ies):", similar.len());
        for (i, episode) in similar.iter().enumerate() {
            println!("     {}. Query: {}", i + 1, episode.query);
            println!("        Response: {} (success: {})", 
                &episode.response[..episode.response.len().min(50)],
                episode.success
            );
        }
    } else {
        println!("   No similar queries found yet");
    }

    // Trigger reflection
    println!("\n5️⃣  Triggering reflection analysis...\n");
    let reflection = agent_memory.reflect_on_episodes()?;
    println!("   Reflection Type: {}", reflection.reflection_type);
    println!("   Insight: {}\n", reflection.insight);

    // ============ Summary ============
    println!("✅ Agent Example Complete!\n");
    println!("Summary:");
    println!("- Phases 1-7 working together");
    println!("- Agent learned from execution");
    println!("- Memory system functional");
    println!("- Decision engine made autonomous decisions");
    println!("\nNext steps:");
    println!("1. Check src/api/agent_routes.rs for HTTP endpoints");
    println!("2. Integrate with Phase 4 memory API");
    println!("3. Build Phase 8: Multi-agent collaboration");

    Ok(())
}

// ============ Simpler Quick Example ============
// Uncomment to use instead of main example above

/*
#[tokio::main]
async fn main_simple() -> Result<(), Box<dyn std::error::Error>> {
    // Quick 3-step example:
    
    // 1. Setup agent
    let embedding_service = Arc::new(EmbeddingService::new(EmbeddingConfig::default()));
    let vector_store = Arc::new(tokio::sync::RwLock::new(VectorStore::with_defaults()?));
    let temp_db = NamedTempFile::new()?;
    let agent_memory = Arc::new(AgentMemoryLayer::new(
        "quick-agent".to_string(),
        "Quick Agent".to_string(),
        temp_db.path().to_path_buf(),
        vector_store,
        embedding_service,
    )?);

    // 2. Set goal
    let goal = agent_memory.set_goal("Answer questions".to_string())?;
    println!("Goal set: {}", goal.goal);

    // 3. Record episodes
    agent_memory.record_episode(
        "What is AI?".to_string(),
        "AI is artificial intelligence.".to_string(),
        3,
        true,
    ).await?;

    // 4. Reflect
    let reflection = agent_memory.reflect_on_episodes()?;
    println!("Reflection: {}", reflection.insight);

    Ok(())
}
*/