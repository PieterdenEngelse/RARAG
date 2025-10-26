// src/api/composer_routes.rs - FIXED TIMING
// Phase 10 Optimization: Proper execution time measurement

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use tracing::info;
use tokio::time::{timeout, Duration};

use crate::tools::tool_composer::ToolComposer;
use crate::tools::tool_executor::ToolExecutor;
use crate::tools::result_formatter::ResultFormatter;
use crate::tools::result_cache::ResultCache;

// ============ Global Cache ============
lazy_static::lazy_static! {
    pub static ref RESULT_CACHE: tokio::sync::Mutex<ResultCache> = 
    tokio::sync::Mutex::new(ResultCache::new(3600));
}

const TOOL_TIMEOUT_SECS: u64 = 30;

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct ComposerRequest {
    pub query: String,
    #[serde(default)]
    pub max_steps: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ChainPlanResponse {
    pub query: String,
    pub is_multi_step: bool,
    pub planned_steps: Vec<PlannedStepResponse>,
    pub total_steps: usize,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct PlannedStepResponse {
    pub step: usize,
    pub tool: String,
    pub purpose: String,
    pub expected_confidence: f32,
}

#[derive(Debug, Serialize)]
pub struct ComposerExecutionResponse {
    pub query: String,
    pub is_multi_step: bool,
    pub chain: Vec<ExecutionStepResponse>,
    pub final_answer: String,
    pub total_confidence: f32,
    pub total_execution_time_ms: u64,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutionStepResponse {
    pub step: usize,
    pub tool: String,
    pub query: String,
    pub formatted_query: Option<String>,
    pub result: Option<String>,
    pub confidence: f32,
    pub execution_time_ms: u64,
    pub success: bool,
    pub from_cache: bool,
}

// ============ Route Handlers ============

/// Plan a tool chain without executing
pub async fn plan_chain(
    req: web::Json<ComposerRequest>,
) -> ActixResult<HttpResponse> {
    info!(query = %req.query, "Planning tool chain");

    let plan = ToolComposer::plan_chain(&req.query);

    let planned_steps: Vec<PlannedStepResponse> = plan
        .planned_steps
        .iter()
        .map(|step| PlannedStepResponse {
            step: step.step,
            tool: step.tool.to_string(),
            purpose: step.purpose.clone(),
            expected_confidence: step.expected_confidence,
        })
        .collect();

    let response = ChainPlanResponse {
        query: plan.query,
        is_multi_step: plan.is_multi_step,
        planned_steps,
        total_steps: plan.total_planned_steps,
        timestamp: Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Execute a multi-step tool chain with caching, timeouts, and smart result formatting
pub async fn execute_chain(
    req: web::Json<ComposerRequest>,
) -> ActixResult<HttpResponse> {
    info!(query = %req.query, "Executing tool chain with optimizations");

    // Plan the chain
    let plan = ToolComposer::plan_chain(&req.query);

    // Create chain from plan
    let mut chain = ToolComposer::create_chain_from_plan(&plan);

    // Execute each step
    let mut total_time: u64 = 0;
    let mut confidences = Vec::new();
    let mut previous_result: Option<String> = None;

    for idx in 0..chain.steps.len() {
        // Get intents BEFORE borrowing step
        let current_intent = chain.steps[idx].tool.to_string().to_lowercase();

        let next_intent = if idx + 1 < chain.steps.len() {
            chain.steps[idx + 1].tool.to_string().to_lowercase()
        } else {
            String::new()
        };

        // NOW borrow step
        let step = &mut chain.steps[idx];

        // Build formatted query if we have previous result
        let formatted_query = if let Some(prev_result) = &previous_result {
            let extracted = ResultFormatter::extract_key_data(&prev_result, &current_intent);
            let formatted = ResultFormatter::build_next_query(
                &extracted,
                &step.query,
                &current_intent,
            );
            step.formatted_query = Some(formatted.clone());
            formatted
        } else {
            step.query.clone()
        };

        // Check cache first
        let mut from_cache = false;
        let tool_type_str = step.tool.to_string();
        
        // START TIMING HERE - right before execution
        let step_start = std::time::Instant::now();
        
        if let Some(tool_result) = RESULT_CACHE.lock().await.get(&tool_type_str, &formatted_query).await {
            step.result = Some(tool_result.result.clone());
            step.confidence = tool_result.metadata.confidence;
            step.execution_time_ms = step_start.elapsed().as_millis() as u64;
            total_time += step.execution_time_ms;
            confidences.push(tool_result.metadata.confidence);
            from_cache = true;

            previous_result = Some(ResultFormatter::extract_key_data(
                &tool_result.result,
                &next_intent,
            ));

            info!(step = idx + 1, from_cache = true, time_ms = step.execution_time_ms, "Tool result from cache");
        } else {
            // Execute with timeout
            match timeout(
                Duration::from_secs(TOOL_TIMEOUT_SECS),
                ToolExecutor::execute_tool(
                    &step.tool,
                    &formatted_query,
                    previous_result.as_deref(),
                )
            ).await
            {
                Ok(Ok(tool_result)) => {
                    // STOP TIMING - after execution
                    let execution_time = step_start.elapsed().as_millis() as u64;
                    
                    // Cache the result
                    RESULT_CACHE.lock().await.set(&tool_type_str, formatted_query.clone(), tool_result.clone()).await;

                    step.result = Some(tool_result.result.clone());
                    step.confidence = tool_result.metadata.confidence;
                    step.execution_time_ms = execution_time;
                    total_time += step.execution_time_ms;
                    confidences.push(tool_result.metadata.confidence);

                    previous_result = Some(ResultFormatter::extract_key_data(
                        &tool_result.result,
                        &next_intent,
                    ));

                    info!(step = idx + 1, success = true, execution_ms = step.execution_time_ms, "Tool executed successfully");
                }
                Ok(Err(e)) => {
                    let execution_time = step_start.elapsed().as_millis() as u64;
                    step.result = Some(format!("Error: {}", e));
                    step.confidence = 0.0;
                    step.execution_time_ms = execution_time;
                    total_time += step.execution_time_ms;
                    confidences.push(0.0);
                    info!(error = %e, step = idx + 1, "Tool execution failed");
                }
                Err(_) => {
                    let execution_time = step_start.elapsed().as_millis() as u64;
                    step.result = Some(format!("Timeout: Tool execution exceeded {}s", TOOL_TIMEOUT_SECS));
                    step.confidence = 0.0;
                    step.execution_time_ms = execution_time;
                    total_time += step.execution_time_ms;
                    confidences.push(0.0);
                    info!(step = idx + 1, timeout_ms = execution_time, "Tool execution timed out");
                }
            }
        }

        // Store from_cache flag for response
        chain.steps[idx].metadata_extra = Some(from_cache);
    }

    // Calculate aggregate confidence
    chain.total_confidence = ToolComposer::calculate_aggregate_confidence(&confidences);
    chain.total_execution_time_ms = total_time;

    // Compose final answer
    chain.final_answer = ToolComposer::compose_answer(&chain);

    // Build response
    let chain_responses: Vec<ExecutionStepResponse> = chain
        .steps
        .iter()
        .map(|step| ExecutionStepResponse {
            step: step.step,
            tool: step.tool.to_string(),
            query: step.query.clone(),
            formatted_query: step.formatted_query.clone(),
            result: step.result.clone(),
            confidence: step.confidence,
            execution_time_ms: step.execution_time_ms,
            success: step.result.is_some() && !step.result.as_ref().unwrap().starts_with("Error"),
            from_cache: step.metadata_extra.unwrap_or(false),
        })
        .collect();

    let response = ComposerExecutionResponse {
        query: req.query.clone(),
        is_multi_step: chain.is_multi_step,
        chain: chain_responses,
        final_answer: chain.final_answer,
        total_confidence: chain.total_confidence,
        total_execution_time_ms: chain.total_execution_time_ms,
        timestamp: Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Detect if query is multi-step
pub async fn detect_multi_step(
    req: web::Json<ComposerRequest>,
) -> ActixResult<HttpResponse> {
    info!(query = %req.query, "Detecting multi-step query");

    let is_multi_step = ToolComposer::is_multi_step_query(&req.query);
    let sub_queries = ToolComposer::split_query(&req.query);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "query": req.query,
        "is_multi_step": is_multi_step,
        "sub_queries": sub_queries,
        "sub_query_count": sub_queries.len(),
        "timestamp": Utc::now().to_rfc3339()
    })))
}

/// Clear cache endpoint
pub async fn clear_cache() -> ActixResult<HttpResponse> {
    RESULT_CACHE.lock().await.clear().await;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Cache cleared"
    })))
}

/// Get cache stats endpoint
pub async fn cache_stats() -> ActixResult<HttpResponse> {
    let size = RESULT_CACHE.lock().await.size().await;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "cache_size": size,
        "timestamp": Utc::now().to_rfc3339()
    })))
}

// ============ Route Configuration ============

pub fn configure_composer_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/composer")
            .route("/plan", web::post().to(plan_chain))
            .route("/execute", web::post().to(execute_chain))
            .route("/detect-multi-step", web::post().to(detect_multi_step))
            .route("/cache/clear", web::post().to(clear_cache))
            .route("/cache/stats", web::get().to(cache_stats))
    );
}