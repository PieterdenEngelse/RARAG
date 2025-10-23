use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::retriever::Retriever;
use crate::agent_memory::AgentMemory;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStep {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentResponse {
    pub answer: String,
    pub steps: Vec<AgentStep>,
    pub used_chunks: Vec<String>,
}

pub struct Agent<'a> {
    pub agent_id: &'a str,
    pub memory_db_path: &'a str,
    pub retriever: Arc<Mutex<Retriever>>,
}

impl<'a> Agent<'a> {
    pub fn new(agent_id: &'a str, memory_db_path: &'a str, retriever: Arc<Mutex<Retriever>>) -> Self {
        Self { agent_id, memory_db_path, retriever }
    }

    pub fn run(&self, query: &str, top_k: usize) -> AgentResponse {
        let mut steps = Vec::new();

        // Step 1: Recall recent memory
        let recalled: Vec<String> = if let Ok(mem) = AgentMemory::new(self.memory_db_path) {
    mem.recall(self.agent_id)
        .map(|items| items.into_iter().take(5).collect())
        .unwrap_or_default()
} else {
    Vec::new()
};

if !recalled.is_empty() {
    steps.push(AgentStep { 
        kind: "memory".into(), 
        message: format!("Recalled {} memory items", recalled.len()) 
    });
}

        // Step 2: Retrieve relevant chunks
        let mut used_chunks: Vec<String> = Vec::new();
        let retrieval_msg: String;
        {
            if let Ok(mut r) = self.retriever.lock() {
                match r.hybrid_search(query, None) {
                    Ok(mut results) => {
                        if results.len() > top_k { results.truncate(top_k); }
                        used_chunks = results;
                        retrieval_msg = format!("Retrieved {} chunks", used_chunks.len());
                    }
                    Err(e) => {
                        retrieval_msg = format!("Retrieval failed: {}", e);
                    }
                }
            } else {
                retrieval_msg = "Failed to acquire retriever lock".into();
            }
        }
        steps.push(AgentStep { kind: "retrieve".into(), message: retrieval_msg });

        // Step 3: (Optional) Simple planning: if no chunks, fallback
        if used_chunks.is_empty() {
            let answer = "I couldn't find relevant information in the knowledge base.".to_string();
            steps.push(AgentStep { kind: "plan".into(), message: "No chunks found; returning fallback".into() });
            self.store_memory(query, &answer);
            return AgentResponse { answer, steps, used_chunks };
        }

        // Step 4: Summarize (naive){ join key lines }
        let answer = naive_summarize(query, &used_chunks);
        steps.push(AgentStep { kind: "summarize".into(), message: format!("Summarized {} chunks", used_chunks.len()) });

        // Step 5: Store memory
        self.store_memory(query, &answer);
        steps.push(AgentStep { kind: "memory".into(), message: "Stored interaction in memory".into() });

        AgentResponse { answer, steps, used_chunks }
    }

    fn store_memory(&self, query: &str, answer: &str) {
        if let Ok(mem) = AgentMemory::new(self.memory_db_path) {
            let ts = Utc::now().to_rfc3339();
            let _ = mem.store(self.agent_id, &format!("Q: {}", query), &ts);
            let _ = mem.store(self.agent_id, &format!("A: {}", answer), &ts);
        }
    }
}

fn naive_summarize(_query: &str, chunks: &Vec<String>) -> String {
    // Very basic: take up to first 3 non-empty lines
    let mut out = String::new();
    for (i, c) in chunks.iter().enumerate() {
        if i >= 3 { break; }
        out.push_str("- ");
        let line = c.lines().next().unwrap_or("");
        out.push_str(line);
        out.push('\n');
    }
    if out.is_empty() { out.push_str("No relevant content found."); }
    out
}
