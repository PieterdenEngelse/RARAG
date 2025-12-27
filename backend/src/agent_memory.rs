use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

pub struct AgentMemory {
    conn: Connection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryItem {
    pub id: i64,
    pub agent_id: String,
    pub memory_type: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemorySearchResult {
    pub item: MemoryItem,
    pub score: f32,
}

impl AgentMemory {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        // Legacy table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_memory (
                id INTEGER PRIMARY KEY,
                agent_id TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;
        // RAG memory table with vector stored as JSON text
        conn.execute(
            "CREATE TABLE IF NOT EXISTS rag_memory (
                id INTEGER PRIMARY KEY,
                agent_id TEXT NOT NULL,
                memory_type TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                vector TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    // Simple append-only legacy store
    pub fn store(&self, agent_id: &str, content: &str, timestamp: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO agent_memory (agent_id, content, timestamp) VALUES (?1, ?2, ?3)",
            (agent_id, content, timestamp),
        )?;
        Ok(())
    }

    pub fn recall(&self, agent_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT content FROM agent_memory WHERE agent_id = ?1 ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map([agent_id], |row| row.get(0))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    // RAG memory: store with embedding
    pub fn store_rag(
        &self,
        agent_id: &str,
        memory_type: &str,
        content: &str,
        timestamp: &str,
    ) -> Result<()> {
        let vec = crate::embedder::embed(content);
        let vector_json = serde_json::to_string(&vec).unwrap_or("[]".to_string());
        self.conn.execute(
            "INSERT INTO rag_memory (agent_id, memory_type, content, timestamp, vector) VALUES (?1, ?2, ?3, ?4, ?5)",
            (agent_id, memory_type, content, timestamp, &vector_json),
        )?;
        Ok(())
    }

    pub fn recall_rag(&self, agent_id: &str, limit: usize) -> Result<Vec<MemoryItem>> {
        let sql = format!(
            "SELECT id, agent_id, memory_type, content, timestamp FROM rag_memory WHERE agent_id = ?1 ORDER BY timestamp DESC LIMIT {}",
            limit
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([agent_id], |row| {
            Ok(MemoryItem {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                memory_type: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn search_rag(
        &self,
        agent_id: &str,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<MemorySearchResult>> {
        // Load all memory vectors for this agent (keep it simple for now)
        let mut stmt = self.conn.prepare(
            "SELECT id, agent_id, memory_type, content, timestamp, vector FROM rag_memory WHERE agent_id = ?1",
        )?;
        let rows = stmt.query_map([agent_id], |row| {
            let id: i64 = row.get(0)?;
            let agent_id: String = row.get(1)?;
            let memory_type: String = row.get(2)?;
            let content: String = row.get(3)?;
            let timestamp: String = row.get(4)?;
            let vector_json: String = row.get(5)?;
            let vector: Vec<f32> = serde_json::from_str(&vector_json).unwrap_or_default();
            Ok((
                MemoryItem {
                    id,
                    agent_id,
                    memory_type,
                    content,
                    timestamp,
                },
                vector,
            ))
        })?;

        let items: Vec<(MemoryItem, Vec<f32>)> = rows.filter_map(Result::ok).collect();
        let q_vec = crate::embedder::embed(query);
        let mut scored: Vec<MemorySearchResult> = items
            .into_iter()
            .map(|(item, vec)| {
                let score = cosine_similarity(&q_vec, &vec);
                MemorySearchResult { item, score }
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top_k);
        Ok(scored)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let ma: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if ma == 0.0 || mb == 0.0 {
        0.0
    } else {
        dot / (ma * mb)
    }
}
