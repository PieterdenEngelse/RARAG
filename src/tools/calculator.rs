// src/tools/calculator.rs - PRODUCTION
// Phase 9: Calculator Tool Implementation

use async_trait::async_trait;
use crate::tools::{Tool, ToolType, ToolResult, ToolMetadata};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct CalculatorTool {
    success_count: usize,
    total_count: usize,
}

impl CalculatorTool {
    pub fn new() -> Self {
        Self {
            success_count: 0,
            total_count: 0,
        }
    }

    fn evaluate_expression(&self, expr: &str) -> Result<String, String> {
        let expr = expr.trim();
        
        // Handle standalone numbers first
        if let Ok(num) = expr.parse::<f64>() {
            return Ok(num.to_string());
        }
        
        // Handle simple cases with operators
        if expr.contains("+") {
            let parts: Vec<&str> = expr.split("+").collect();
            if parts.len() == 2 {
                if let (Ok(a), Ok(b)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                    return Ok((a + b).to_string());
                }
            }
        }
        
        if expr.contains("-") && !expr.starts_with("-") {
            let parts: Vec<&str> = expr.split("-").collect();
            if parts.len() == 2 {
                if let (Ok(a), Ok(b)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                    return Ok((a - b).to_string());
                }
            }
        }
        
        if expr.contains("*") {
            let parts: Vec<&str> = expr.split("*").collect();
            if parts.len() == 2 {
                if let (Ok(a), Ok(b)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                    return Ok((a * b).to_string());
                }
            }
        }
        
        if expr.contains("/") {
            let parts: Vec<&str> = expr.split("/").collect();
            if parts.len() == 2 {
                if let (Ok(a), Ok(b)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                    if b != 0.0 {
                        return Ok((a / b).to_string());
                    }
                }
            }
        }
        
        Err("Could not evaluate expression".to_string())
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn tool_type(&self) -> ToolType {
        ToolType::Calculator
    }

    fn description(&self) -> String {
        "Perform mathematical calculations and arithmetic operations".to_string()
    }

    fn success_rate(&self) -> f32 {
        if self.total_count == 0 {
            0.95
        } else {
            self.success_count as f32 / self.total_count as f32
        }
    }

    async fn execute(&self, query: &str) -> Result<ToolResult, String> {
        let start = Instant::now();

        match self.evaluate_expression(query) {
            Ok(result) => {
                Ok(ToolResult {
                    tool: ToolType::Calculator,
                    success: true,
                    result: format!("{} = {}", query, result),
                    metadata: ToolMetadata {
                        execution_time_ms: start.elapsed().as_millis() as u64,
                        confidence: 0.99,
                        source: Some("Calculator".to_string()),
                        cost: Some(0.0),
                    },
                })
            }
            Err(_) => {
                Ok(ToolResult {
                    tool: ToolType::Calculator,
                    success: false,
                    result: format!("Could not evaluate: {}", query),
                    metadata: ToolMetadata {
                        execution_time_ms: start.elapsed().as_millis() as u64,
                        confidence: 0.0,
                        source: Some("Calculator".to_string()),
                        cost: Some(0.0),
                    },
                })
            }
        }
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
    async fn test_calculator_add() {
        let tool = CalculatorTool::new();
        let result = tool.execute("5 + 3").await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.result.contains("8"));
    }

    #[tokio::test]
    async fn test_calculator_multiply() {
        let tool = CalculatorTool::new();
        let result = tool.execute("6 * 7").await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.result.contains("42"));
    }
    
    #[tokio::test]
    async fn test_calculator_standalone_number() {
        let tool = CalculatorTool::new();
        let result = tool.execute("5").await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.success);
        assert!(res.result.contains("5"));
    }
}