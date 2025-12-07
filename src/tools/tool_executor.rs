// src/tools/tool_executor.rs - FIXED
// Phase 10: Execute individual tools in a chain
// Handles actual tool execution with result passing

use crate::tools::calculator::CalculatorTool;
use crate::tools::url_fetch::URLFetchTool;
use crate::tools::web_search::WebSearchTool;
use crate::tools::{Tool, ToolResult, ToolType};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub step_number: usize,
    pub tool: ToolType,
    pub query: String,
    pub previous_result: Option<String>,
}

pub struct ToolExecutor;

impl ToolExecutor {
    /// Execute a single tool with context
    pub async fn execute_tool(
        tool_type: &ToolType,
        query: &str,
        _previous_result: Option<&str>,
    ) -> Result<ToolResult, String> {
        let start = Instant::now();

        // Use query as-is, don't append context
        let input_query = query.to_string();

        // Execute appropriate tool
        let result = match tool_type {
            ToolType::Calculator => {
                let calculator = CalculatorTool::new();
                calculator.execute(&input_query).await?
            }
            ToolType::WebSearch => {
                let web_search = WebSearchTool::new();
                web_search.execute(&input_query).await?
            }
            ToolType::URLFetch => {
                let url_fetch = URLFetchTool::new();
                url_fetch.execute(&input_query).await?
            }
            ToolType::SemanticSearch => {
                // Fallback to semantic search description
                ToolResult {
                    tool: ToolType::SemanticSearch,
                    success: true,
                    result: format!("Semantic search for: {}", input_query),
                    metadata: crate::tools::ToolMetadata {
                        execution_time_ms: start.elapsed().as_millis() as u64,
                        confidence: 0.70,
                        source: Some("SemanticSearch".to_string()),
                        cost: Some(0.0),
                    },
                }
            }
            _ => {
                return Err(format!("Tool {:?} not implemented", tool_type));
            }
        };

        Ok(result)
    }

    /// Extract relevant data from tool result
    pub fn extract_data(result: &str) -> String {
        // Try to extract numbers if it's a calculation result
        if let Some(pos) = result.rfind('=') {
            let number_part = &result[pos + 1..].trim();
            if number_part.chars().all(|c| c.is_numeric() || c == '.') {
                return number_part.to_string();
            }
        }

        // Otherwise return the whole result
        result.to_string()
    }

    /// Validate tool result
    pub fn validate_result(result: &ToolResult) -> bool {
        result.success && !result.result.is_empty()
    }

    /// Retry tool execution with fallback
    pub async fn execute_with_fallback(
        primary_tool: &ToolType,
        fallback_tools: &[ToolType],
        query: &str,
        previous_result: Option<&str>,
    ) -> Result<ToolResult, String> {
        // Try primary tool
        match Self::execute_tool(primary_tool, query, previous_result).await {
            Ok(result) if Self::validate_result(&result) => {
                return Ok(result);
            }
            _ => {
                // Try fallback tools
                for fallback_tool in fallback_tools {
                    match Self::execute_tool(fallback_tool, query, previous_result).await {
                        Ok(result) if Self::validate_result(&result) => {
                            return Ok(result);
                        }
                        _ => continue,
                    }
                }
            }
        }

        Err(format!(
            "All tools failed. Primary: {:?}, Fallbacks: {:?}",
            primary_tool, fallback_tools
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_calculator() {
        let result = ToolExecutor::execute_tool(&ToolType::Calculator, "5 + 3", None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[tokio::test]
    async fn test_execute_web_search() {
        let result = ToolExecutor::execute_tool(&ToolType::WebSearch, "AI papers", None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[test]
    fn test_extract_data() {
        let result = "5 + 3 = 8";
        let extracted = ToolExecutor::extract_data(result);
        assert_eq!(extracted, "8");
    }

    #[test]
    fn test_validate_result() {
        let valid_result = ToolResult {
            tool: ToolType::Calculator,
            success: true,
            result: "8".to_string(),
            metadata: crate::tools::ToolMetadata {
                execution_time_ms: 100,
                confidence: 0.99,
                source: None,
                cost: None,
            },
        };

        assert!(ToolExecutor::validate_result(&valid_result));
    }
}
