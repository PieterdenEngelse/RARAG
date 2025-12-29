// src/memory/decision_engine.rs
// Phase 7: Agent Decision Engine
// Multi-step reasoning, tool selection, autonomous execution

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

use crate::memory::{AgentMemoryLayer, Episode, RagQueryPipeline, RagQueryRequest};

/// Available tools the agent can use
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Tool {
    SemanticSearch,   // Search vector store for relevant chunks
    ReflectOnHistory, // Analyze past episodes for patterns
    RefinedSearch,    // Search with adjusted parameters based on reflection
    DirectAnswer,     // Provide answer from retrieved context
}

/// Decision made by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub tool: Tool,
    pub reasoning: String,
    pub confidence: f32,
}

/// Step in agent's plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub step_number: usize,
    pub action: String,
    pub expected_outcome: String,
}

/// Agent's execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub estimated_effort: usize, // number of steps
}

/// Result of agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub plan: ExecutionPlan,
    pub answer: String,
    pub steps_executed: usize,
    pub success: bool,
    pub reasoning_trace: Vec<String>,
}

/// Agent Decision Engine
pub struct DecisionEngine {
    rag_pipeline: Arc<RagQueryPipeline>,
    agent_memory: Arc<AgentMemoryLayer>,
}

impl DecisionEngine {
    /// Create new decision engine
    pub fn new(rag_pipeline: Arc<RagQueryPipeline>, agent_memory: Arc<AgentMemoryLayer>) -> Self {
        info!("Initializing Agent Decision Engine");
        Self {
            rag_pipeline,
            agent_memory,
        }
    }

    /// Execute a query with multi-step reasoning
    pub async fn execute_query(
        &self,
        query: &str,
        goal_id: Option<String>,
    ) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
        info!(query = %query, "Starting agent execution");

        let mut reasoning_trace = Vec::new();
        let mut steps_executed = 0;

        // Step 1: Assess situation
        reasoning_trace.push("Step 1: Assessing query and goal context".to_string());
        debug!("Assessing query: {}", query);

        let context = self.agent_memory.get_agent_context()?;
        reasoning_trace.push(format!(
            "Found {} active goals, {} recent episodes",
            context.active_goals.len(),
            context.recent_episodes.len()
        ));

        // Step 2: Check for similar past queries
        reasoning_trace.push("Step 2: Checking for similar past queries".to_string());
        let similar_queries = self.agent_memory.recall_similar_episodes(query, 3).await?;
        steps_executed += 1;

        if !similar_queries.is_empty() {
            let success_rate = self.calculate_success_rate(&similar_queries);
            reasoning_trace.push(format!(
                "Found {} similar queries with {:.1}% success rate",
                similar_queries.len(),
                success_rate * 100.0
            ));

            // If past similar queries failed, refine search strategy
            if success_rate < 0.5 {
                reasoning_trace.push(
                    "Past similar queries had low success - will refine search parameters"
                        .to_string(),
                );
            }
        }

        // Step 3: Decide on tool/strategy
        reasoning_trace.push("Step 3: Deciding on search strategy".to_string());
        let decision = self.make_decision(query, &similar_queries).await?;
        reasoning_trace.push(format!(
            "Decision: {:?} (confidence: {:.1}%)",
            decision.tool,
            decision.confidence * 100.0
        ));
        reasoning_trace.push(format!("Reasoning: {}", decision.reasoning));

        // Step 4: Execute RAG pipeline
        reasoning_trace.push("Step 4: Executing RAG query pipeline".to_string());
        let rag_request = RagQueryRequest {
            query: query.to_string(),
            top_k: self.determine_top_k(&decision),
            include_sources: true,
        };

        let rag_response = self.rag_pipeline.query(&rag_request).await?;
        steps_executed += 1;
        reasoning_trace.push(format!(
            "Retrieved {} context chunks",
            rag_response.total_chunks_used
        ));

        // Step 5: Record episode in memory
        reasoning_trace.push("Step 5: Recording episode in memory".to_string());
        let success = !rag_response.answer.is_empty();
        let _episode = self
            .agent_memory
            .record_episode(
                query.to_string(),
                rag_response.answer.clone(),
                rag_response.total_chunks_used,
                success,
            )
            .await?;
        steps_executed += 1;

        // Step 6: Update goal if provided
        if let Some(goal_id) = goal_id {
            reasoning_trace.push(format!("Step 6: Updating goal {}", goal_id));
            self.agent_memory.complete_goal(&goal_id)?;
            steps_executed += 1;
        }

        // Step 7: Reflect on execution
        reasoning_trace.push("Step 7: Reflecting on execution".to_string());
        let _reflection = self.agent_memory.reflect_on_episodes()?;
        reasoning_trace.push("Reflection stored for future learning".to_string());
        steps_executed += 1;

        let plan = ExecutionPlan {
            goal: query.to_string(),
            steps: vec![
                PlanStep {
                    step_number: 1,
                    action: "Assess context".to_string(),
                    expected_outcome: "Understand goal and history".to_string(),
                },
                PlanStep {
                    step_number: 2,
                    action: "Search similar queries".to_string(),
                    expected_outcome: "Learn from past".to_string(),
                },
                PlanStep {
                    step_number: 3,
                    action: "Decide strategy".to_string(),
                    expected_outcome: "Choose best tool".to_string(),
                },
                PlanStep {
                    step_number: 4,
                    action: "Execute RAG pipeline".to_string(),
                    expected_outcome: "Generate answer".to_string(),
                },
                PlanStep {
                    step_number: 5,
                    action: "Record learning".to_string(),
                    expected_outcome: "Improve future decisions".to_string(),
                },
            ],
            estimated_effort: 5,
        };

        info!(
            query = %query,
            success = success,
            steps = steps_executed,
            "Agent execution completed"
        );

        Ok(ExecutionResult {
            plan,
            answer: rag_response.answer,
            steps_executed,
            success,
            reasoning_trace,
        })
    }

    /// Make decision about which tool to use
    async fn make_decision(
        &self,
        _query: &str,
        similar_queries: &[Episode],
    ) -> Result<Decision, Box<dyn std::error::Error>> {
        // Simple heuristic-based decision making
        // In Phase 8+, could use ML model for this

        if similar_queries.is_empty() {
            return Ok(Decision {
                tool: Tool::SemanticSearch,
                reasoning: "No similar past queries found, starting fresh search".to_string(),
                confidence: 0.6,
            });
        }

        let success_rate = self.calculate_success_rate(similar_queries);

        if success_rate > 0.8 {
            Ok(Decision {
                tool: Tool::DirectAnswer,
                reasoning: "Similar past queries succeeded - using similar strategy".to_string(),
                confidence: 0.9,
            })
        } else if success_rate > 0.5 {
            Ok(Decision {
                tool: Tool::RefinedSearch,
                reasoning: "Mixed results from similar queries - refining approach".to_string(),
                confidence: 0.7,
            })
        } else {
            Ok(Decision {
                tool: Tool::SemanticSearch,
                reasoning: "Similar queries had low success - trying new approach".to_string(),
                confidence: 0.5,
            })
        }
    }

    /// Calculate success rate from episodes
    fn calculate_success_rate(&self, episodes: &[Episode]) -> f32 {
        if episodes.is_empty() {
            return 0.0;
        }
        let successes = episodes.iter().filter(|e| e.success).count();
        successes as f32 / episodes.len() as f32
    }

    /// Determine top_k based on decision confidence
    fn determine_top_k(&self, decision: &Decision) -> usize {
        match decision.tool {
            Tool::DirectAnswer => 3,
            Tool::RefinedSearch => 5,
            Tool::SemanticSearch => 7,
            Tool::ReflectOnHistory => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_success_rate() {
        let engine = DecisionEngine::new(
            Arc::new(RagQueryPipeline::new(
                std::sync::Arc::new(crate::embedder::EmbeddingService::new(
                    crate::embedder::EmbeddingConfig::default(),
                )),
                std::sync::Arc::new(tokio::sync::RwLock::new(
                    crate::memory::VectorStore::with_defaults().unwrap(),
                )),
                std::sync::Arc::new(MockLLM),
                Default::default(),
            )),
            Arc::new(
                AgentMemoryLayer::new(
                    "test".to_string(),
                    "test".to_string(),
                    tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
                    std::sync::Arc::new(tokio::sync::RwLock::new(
                        crate::memory::VectorStore::with_defaults().unwrap(),
                    )),
                    std::sync::Arc::new(crate::embedder::EmbeddingService::new(
                        crate::embedder::EmbeddingConfig::default(),
                    )),
                )
                .unwrap(),
            ),
        );

        let episodes = vec![
            Episode {
                id: "1".to_string(),
                agent_id: "test".to_string(),
                query: "test".to_string(),
                response: "answer".to_string(),
                context_chunks_used: 3,
                success: true,
                created_at: 0,
            },
            Episode {
                id: "2".to_string(),
                agent_id: "test".to_string(),
                query: "test".to_string(),
                response: "answer".to_string(),
                context_chunks_used: 3,
                success: false,
                created_at: 0,
            },
        ];

        let rate = engine.calculate_success_rate(&episodes);
        assert!((rate - 0.5).abs() < 0.01);
    }

    struct MockLLM;

    #[async_trait::async_trait]
    impl crate::memory::LLMProvider for MockLLM {
        async fn generate(&self, _prompt: &str) -> Result<String, crate::memory::LLMError> {
            Ok("test".to_string())
        }
        async fn generate_with_config(
            &self,
            _prompt: &str,
            _config: &crate::db::llm_settings::LlmConfig,
        ) -> Result<String, crate::memory::LLMError> {
            Ok("test".to_string())
        }
        fn model_name(&self) -> &str {
            "mock"
        }
    }
}
