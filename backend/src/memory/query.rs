// src/memory/query.rs
// Phase 5: RAG Query Pipeline
// Retrieval + Context Assembly + LLM Generation (Phi 3.5)

use crate::embedder::EmbeddingService;
use crate::memory::llm_provider::LLMProvider;
use crate::memory::VectorStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// RAG query request
#[derive(Debug, Deserialize)]
pub struct RagQueryRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default)]
    pub include_sources: bool,
}

/// Context chunk with metadata
#[derive(Debug, Clone, Serialize)]
pub struct ContextChunk {
    pub chunk_id: String,
    pub document_id: String,
    pub content: String,
    pub similarity_score: f32,
    pub chunk_index: usize,
    pub source: String,
}

/// RAG query response
#[derive(Debug, Serialize)]
pub struct RagQueryResponse {
    pub query: String,
    pub answer: String,
    pub context_chunks: Vec<ContextChunk>,
    pub total_chunks_used: usize,
    pub sources: Vec<String>,
}

/// Configuration for RAG pipeline
#[derive(Debug, Clone)]
pub struct RagConfig {
    pub top_k: usize,
    pub similarity_threshold: f32,
    pub max_context_length: usize,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            top_k: 5,
            similarity_threshold: 0.3,
            max_context_length: 2000,
        }
    }
}

fn default_top_k() -> usize {
    5
}

/// RAG Query Pipeline
pub struct RagQueryPipeline {
    embedding_service: std::sync::Arc<EmbeddingService>,
    vector_store: std::sync::Arc<tokio::sync::RwLock<VectorStore>>,
    llm_provider: Arc<dyn LLMProvider>,
    config: RagConfig,
}

impl RagQueryPipeline {
    /// Create a new RAG pipeline with LLM provider
    pub fn new(
        embedding_service: std::sync::Arc<EmbeddingService>,
        vector_store: std::sync::Arc<tokio::sync::RwLock<VectorStore>>,
        llm_provider: Arc<dyn LLMProvider>,
        config: RagConfig,
    ) -> Self {
        info!(
            llm_model = llm_provider.model_name(),
            "Initializing RAG pipeline"
        );
        Self {
            embedding_service,
            vector_store,
            llm_provider,
            config,
        }
    }

    /// Execute the RAG query pipeline
    pub async fn query(&self, req: &RagQueryRequest) -> Result<RagQueryResponse, RagError> {
        info!(query = %req.query, top_k = req.top_k, "Starting RAG query");

        // Step 1: Embed the query
        debug!("Step 1: Embedding query");
        let query_embedding = self.embedding_service.embed_query(&req.query).await;

        // Step 2: Search vector store
        debug!("Step 2: Searching vector store");
        let mut store = self.vector_store.write().await;
        let search_results = store
            .search(&query_embedding, req.top_k)
            .await
            .map_err(|e| RagError::SearchFailed(e.to_string()))?;

        // Step 3: Filter by similarity threshold
        debug!(
            "Step 3: Filtering by threshold ({})",
            self.config.similarity_threshold
        );
        let filtered_results: Vec<_> = search_results
            .into_iter()
            .filter(|r| r.similarity_score >= self.config.similarity_threshold)
            .collect();

        if filtered_results.is_empty() {
            info!("No results found above similarity threshold");
            return Ok(RagQueryResponse {
                query: req.query.clone(),
                answer: "No relevant information found.".to_string(),
                context_chunks: vec![],
                total_chunks_used: 0,
                sources: vec![],
            });
        }

        // Step 4: Assemble context
        debug!("Step 4: Assembling context");
        let context_chunks: Vec<ContextChunk> = filtered_results
            .iter()
            .map(|r| ContextChunk {
                chunk_id: r.chunk_id.clone(),
                document_id: r.document_id.clone(),
                content: r.content.clone(),
                similarity_score: r.similarity_score,
                chunk_index: r.chunk_index,
                source: String::new(), // Will be filled from metadata if available
            })
            .collect();

        let context = self.assemble_context(&context_chunks);

        // Step 5: Generate answer with LLM
        debug!("Step 5: Generating answer with LLM");
        let answer = self.generate_answer(&req.query, &context).await?;

        // Step 6: Extract unique sources
        debug!("Step 6: Extracting sources");
        let mut sources: Vec<String> = context_chunks
            .iter()
            .map(|c| c.document_id.clone())
            .collect();
        sources.dedup();

        let total_chunks = context_chunks.len();

        info!(
            query = %req.query,
            chunks_used = total_chunks,
            sources_count = sources.len(),
            "RAG query completed"
        );

        Ok(RagQueryResponse {
            query: req.query.clone(),
            answer,
            context_chunks,
            total_chunks_used: total_chunks,
            sources,
        })
    }

    /// Assemble context from chunks
    fn assemble_context(&self, chunks: &[ContextChunk]) -> String {
        let mut context = String::new();
        let mut current_length = 0;

        for chunk in chunks {
            if current_length + chunk.content.len() > self.config.max_context_length {
                context.push_str("\n[... context truncated ...]");
                break;
            }

            context.push_str(&format!(
                "From {} (chunk {}): {}\n\n",
                chunk.document_id, chunk.chunk_index, chunk.content
            ));
            current_length += chunk.content.len();
        }

        context
    }

    /// Generate answer from query and context using LLM
    async fn generate_answer(&self, query: &str, context: &str) -> Result<String, RagError> {
        debug!("Step 5: Generating answer with LLM");

        // Build prompt template
        let prompt = format!(
            r#"You are a helpful assistant. Answer the following question based on the provided context.

Question: {}

Context:
{}

Answer:"#,
            query, context
        );

        // Call LLM provider
        self.llm_provider
            .generate(&prompt)
            .await
            .map_err(|e| RagError::LLMGenerationFailed(e.to_string()))
    }
}

/// Error types for RAG operations
#[derive(Debug, Clone)]
pub enum RagError {
    SearchFailed(String),
    EmbeddingFailed(String),
    NoResultsFound,
    ContextAssemblyFailed(String),
    LLMGenerationFailed(String),
}

impl std::fmt::Display for RagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SearchFailed(msg) => write!(f, "Search failed: {}", msg),
            Self::EmbeddingFailed(msg) => write!(f, "Embedding failed: {}", msg),
            Self::NoResultsFound => write!(f, "No results found"),
            Self::ContextAssemblyFailed(msg) => write!(f, "Context assembly failed: {}", msg),
            Self::LLMGenerationFailed(msg) => write!(f, "LLM generation failed: {}", msg),
        }
    }
}

impl std::error::Error for RagError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rag_config_default() {
        let config = RagConfig::default();
        assert_eq!(config.top_k, 5);
        assert_eq!(config.similarity_threshold, 0.3);
        assert_eq!(config.max_context_length, 2000);
    }

    #[test]
    fn test_context_assembly() {
        struct MockLLM;

        #[async_trait::async_trait]
        impl LLMProvider for MockLLM {
            async fn generate(
                &self,
                _prompt: &str,
            ) -> Result<String, crate::memory::llm_provider::LLMError> {
                Ok("test".to_string())
            }
            fn model_name(&self) -> &str {
                "mock"
            }
        }

        let pipeline = RagQueryPipeline::new(
            std::sync::Arc::new(EmbeddingService::new(
                crate::embedder::EmbeddingConfig::default(),
            )),
            std::sync::Arc::new(tokio::sync::RwLock::new(
                crate::memory::VectorStore::with_defaults().unwrap(),
            )),
            std::sync::Arc::new(MockLLM),
            RagConfig::default(),
        );

        let chunks = vec![
            ContextChunk {
                chunk_id: "c1".to_string(),
                document_id: "doc1".to_string(),
                content: "First chunk content".to_string(),
                similarity_score: 0.9,
                chunk_index: 0,
                source: "test.txt".to_string(),
            },
            ContextChunk {
                chunk_id: "c2".to_string(),
                document_id: "doc1".to_string(),
                content: "Second chunk content".to_string(),
                similarity_score: 0.8,
                chunk_index: 1,
                source: "test.txt".to_string(),
            },
        ];

        let context = pipeline.assemble_context(&chunks);
        assert!(context.contains("First chunk content"));
        assert!(context.contains("Second chunk content"));
        assert!(context.contains("doc1"));
    }

    #[tokio::test]
    async fn test_generate_answer_placeholder() {
        // Create a mock LLM provider for testing
        struct MockLLM;

        #[async_trait::async_trait]
        impl LLMProvider for MockLLM {
            async fn generate(
                &self,
                _prompt: &str,
            ) -> Result<String, crate::memory::llm_provider::LLMError> {
                Ok("This is a test answer".to_string())
            }
            fn model_name(&self) -> &str {
                "mock"
            }
        }

        let pipeline = RagQueryPipeline::new(
            std::sync::Arc::new(EmbeddingService::new(
                crate::embedder::EmbeddingConfig::default(),
            )),
            std::sync::Arc::new(tokio::sync::RwLock::new(
                crate::memory::VectorStore::with_defaults().unwrap(),
            )),
            std::sync::Arc::new(MockLLM),
            RagConfig::default(),
        );

        let answer = pipeline.generate_answer("test query", "test context").await;

        assert!(answer.is_ok());
        let ans = answer.unwrap();
        assert!(ans.contains("test answer"));
    }

    #[test]
    fn test_rag_error_display() {
        let err = RagError::SearchFailed("test error".to_string());
        assert_eq!(format!("{}", err), "Search failed: test error");
    }
}
