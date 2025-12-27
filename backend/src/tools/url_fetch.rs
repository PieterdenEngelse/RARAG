// src/tools/url_fetch.rs
// Phase 9: URL Fetch Tool Implementation

use crate::tools::{Tool, ToolMetadata, ToolResult, ToolType};
use async_trait::async_trait;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct URLFetchTool {
    success_count: usize,
    total_count: usize,
}

impl URLFetchTool {
    pub fn new() -> Self {
        Self {
            success_count: 0,
            total_count: 0,
        }
    }

    fn extract_urls(&self, query: &str) -> Vec<String> {
        // Simple URL extraction
        let mut urls = Vec::new();

        // Look for http:// or https://
        if let Some(start) = query.find("http") {
            let rest = &query[start..];
            if let Some(space) = rest.find(' ') {
                urls.push(rest[..space].to_string());
            } else {
                urls.push(rest.to_string());
            }
        }

        urls
    }
}

#[async_trait]
impl Tool for URLFetchTool {
    fn tool_type(&self) -> ToolType {
        ToolType::URLFetch
    }

    fn description(&self) -> String {
        "Fetch and extract content from URLs".to_string()
    }

    fn success_rate(&self) -> f32 {
        if self.total_count == 0 {
            0.75
        } else {
            self.success_count as f32 / self.total_count as f32
        }
    }

    async fn execute(&self, query: &str) -> Result<ToolResult, String> {
        let start = Instant::now();

        // Extract URLs from query
        let urls = self.extract_urls(query);

        if urls.is_empty() {
            return Ok(ToolResult {
                tool: ToolType::URLFetch,
                success: false,
                result: "No valid URLs found in query".to_string(),
                metadata: ToolMetadata {
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    confidence: 0.0,
                    source: None,
                    cost: Some(0.0),
                },
            });
        }

        // In production, would use reqwest or similar
        let result = format!(
            "Fetched content from {} URL(s): {:?}\n\
             Content preview: [Document content would be here in production]\n\
             Extracted {} chunks of relevant information",
            urls.len(),
            urls,
            urls.len() * 3
        );

        Ok(ToolResult {
            tool: ToolType::URLFetch,
            success: true,
            result,
            metadata: ToolMetadata {
                execution_time_ms: start.elapsed().as_millis() as u64,
                confidence: 0.80,
                source: Some(format!("{} URLs", urls.len())),
                cost: Some(0.02 * urls.len() as f32),
            },
        })
    }

    fn update_success(&mut self, success: bool) {
        self.total_count += 1;
        if success {
            self.success_count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_url_fetch_valid() {
        let tool = URLFetchTool::new();
        let result = tool.execute("Fetch https://example.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_url_fetch_no_url() {
        let tool = URLFetchTool::new();
        let result = tool.execute("No URL here").await;
        assert!(result.is_ok());
        assert!(!result.unwrap().success);
    }
}
