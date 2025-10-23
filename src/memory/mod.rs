// src/memory/mod.rs

pub mod chunker;
pub mod vector_store;
pub mod query;
pub mod llm_provider;
pub mod agent;
pub mod decision_engine;
pub mod persistence;
pub mod multi_agent;

pub use chunker::{Chunk, ChunkMetadata, ChunkerConfig, SemanticChunker, SourceType};
pub use vector_store::{VectorStore, VectorStoreConfig, VectorRecord, SearchResult, StoreStats, VectorStoreError};
pub use query::{RagQueryPipeline, RagQueryRequest, RagQueryResponse, RagConfig, RagError, ContextChunk};
pub use llm_provider::{LLMConfig, LLMProvider, LLMError, create_llm_provider};
pub use agent::{AgentMemoryLayer, Agent, Goal, GoalStatus, Task, TaskStatus, Episode, Reflection, ReflectionType, AgentContext};
pub use decision_engine::{DecisionEngine, Decision, Tool, ExecutionPlan, ExecutionResult, PlanStep};
pub use persistence::{save_vector_store, load_vector_store, backup_vector_store};