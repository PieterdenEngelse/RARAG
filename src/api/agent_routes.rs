// src/api/agent_routes.rs
// Phase 6: Agent Memory Layer API Endpoints

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::memory::{AgentMemoryLayer, Goal, Episode, Reflection, AgentContext};

pub type SharedAgentMemory = Arc<AgentMemoryLayer>;

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct CreateGoalRequest {
    pub goal: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordEpisodeRequest {
    pub query: String,
    pub response: String,
    pub context_chunks_used: usize,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct SimilarQueriesRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

#[derive(Debug, Serialize)]
pub struct GoalResponse {
    pub id: String,
    pub goal: String,
    pub status: String,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct EpisodeResponse {
    pub id: String,
    pub query: String,
    pub response: String,
    pub context_chunks_used: usize,
    pub success: bool,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct ReflectionResponse {
    pub id: String,
    pub reflection_type: String,
    pub insight: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct AgentContextResponse {
    pub agent_id: String,
    pub active_goals: Vec<GoalResponse>,
    pub recent_episodes_count: usize,
    pub recent_reflections_count: usize,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error: String,
}

fn default_top_k() -> usize {
    3
}

// ============ Goal Endpoints ============

/// Create a new goal
pub async fn create_goal(
    agent_memory: web::Data<SharedAgentMemory>,
    req: web::Json<CreateGoalRequest>,
) -> ActixResult<HttpResponse> {
    match agent_memory.set_goal(req.goal.clone()) {
        Ok(goal) => {
            let response = GoalResponse {
                id: goal.id,
                goal: goal.goal,
                status: goal.status.to_string(),
                created_at: goal.created_at,
                completed_at: goal.completed_at,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

/// Get all active goals
pub async fn get_active_goals(
    agent_memory: web::Data<SharedAgentMemory>,
) -> ActixResult<HttpResponse> {
    match agent_memory.get_active_goals() {
        Ok(goals) => {
            let response: Vec<GoalResponse> = goals
                .into_iter()
                .map(|g| GoalResponse {
                    id: g.id,
                    goal: g.goal,
                    status: g.status.to_string(),
                    created_at: g.created_at,
                    completed_at: g.completed_at,
                })
                .collect();
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

/// Complete a goal
pub async fn complete_goal(
    agent_memory: web::Data<SharedAgentMemory>,
    goal_id: web::Path<String>,
) -> ActixResult<HttpResponse> {
    match agent_memory.complete_goal(&goal_id) {
        Ok(_) => Ok(HttpResponse::Ok().json(MessageResponse {
            status: "success".to_string(),
            message: format!("Goal {} completed", goal_id),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

// ============ Episode Endpoints ============

/// Record an episode (query + response)
pub async fn record_episode(
    agent_memory: web::Data<SharedAgentMemory>,
    req: web::Json<RecordEpisodeRequest>,
) -> ActixResult<HttpResponse> {
    match agent_memory
        .record_episode(
            req.query.clone(),
            req.response.clone(),
            req.context_chunks_used,
            req.success,
        )
        .await
    {
        Ok(episode) => {
            let response = EpisodeResponse {
                id: episode.id,
                query: episode.query,
                response: episode.response,
                context_chunks_used: episode.context_chunks_used,
                success: episode.success,
                created_at: episode.created_at,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

/// Get similar past queries
pub async fn get_similar_queries(
    agent_memory: web::Data<SharedAgentMemory>,
    req: web::Json<SimilarQueriesRequest>,
) -> ActixResult<HttpResponse> {
    match agent_memory
        .get_similar_past_queries(&req.query, req.top_k)
        .await
    {
        Ok(episodes) => {
            let response: Vec<EpisodeResponse> = episodes
                .into_iter()
                .map(|e| EpisodeResponse {
                    id: e.id,
                    query: e.query,
                    response: e.response,
                    context_chunks_used: e.context_chunks_used,
                    success: e.success,
                    created_at: e.created_at,
                })
                .collect();
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

// ============ Reflection Endpoints ============

/// Trigger reflection analysis
pub async fn reflect(
    agent_memory: web::Data<SharedAgentMemory>,
) -> ActixResult<HttpResponse> {
    match agent_memory.reflect_on_episodes() {
        Ok(reflection) => {
            let response = ReflectionResponse {
                id: reflection.id,
                reflection_type: reflection.reflection_type.to_string(),
                insight: reflection.insight,
                created_at: reflection.created_at,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

// ============ Context Endpoint ============

/// Get full agent context (for decision making)
pub async fn get_agent_context(
    agent_memory: web::Data<SharedAgentMemory>,
) -> ActixResult<HttpResponse> {
    match agent_memory.get_agent_context() {
        Ok(context) => {
            let response = AgentContextResponse {
                agent_id: context.agent_id,
                active_goals: context
                    .active_goals
                    .into_iter()
                    .map(|g| GoalResponse {
                        id: g.id,
                        goal: g.goal,
                        status: g.status.to_string(),
                        created_at: g.created_at,
                        completed_at: g.completed_at,
                    })
                    .collect(),
                recent_episodes_count: context.recent_episodes.len(),
                recent_reflections_count: context.recent_reflections.len(),
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        })),
    }
}

// ============ Health Check ============

/// Health check for agent memory service
pub async fn agent_health(
    agent_memory: web::Data<SharedAgentMemory>,
) -> ActixResult<HttpResponse> {
    match agent_memory.get_agent_context() {
        Ok(context) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "agent_id": context.agent_id,
            "active_goals": context.active_goals.len(),
            "recent_episodes": context.recent_episodes.len(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))),
        Err(e) => Ok(HttpResponse::ServiceUnavailable().json(ErrorResponse {
            status: "unhealthy".to_string(),
            error: e.to_string(),
        })),
    }
}