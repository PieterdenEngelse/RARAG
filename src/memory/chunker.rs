// src/memory/chunker.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub content: String,
    pub chunk_index: usize,
    pub token_count: usize,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub document_id: String,
    pub source: String,
    pub source_type: SourceType,
    pub created_at: i64,
    pub start_char: usize,
    pub end_char: usize,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    Pdf,
    Text,
    Markdown,
    Html,
    Code,
}

pub const DEFAULT_SEMANTIC_SIMILARITY_THRESHOLD: f32 = 0.78;

#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    pub target_size: usize, // 512 tokens
    pub min_size: usize,    // 256 tokens (allow smaller for semantic boundaries)
    pub max_size: usize,    // 768 tokens (allow larger to avoid splitting mid-concept)
    pub overlap: usize,     // 75 tokens (middle of 50-100 range)
    pub semantic_similarity_threshold: f32,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            target_size: 512,
            min_size: 256,
            max_size: 768,
            overlap: 75,
            semantic_similarity_threshold: DEFAULT_SEMANTIC_SIMILARITY_THRESHOLD,
        }
    }
}

impl ChunkerConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Ok(raw) = env::var("SEMANTIC_SIMILARITY_THRESHOLD") {
            if let Ok(value) = raw.parse::<f32>() {
                // Clamp between 0 and 1 to avoid invalid cosine thresholds
                config.semantic_similarity_threshold = value.clamp(0.0, 1.0);
            }
        }
        config
    }
}

pub struct SemanticChunker {
    config: ChunkerConfig,
}

impl SemanticChunker {
    pub fn new(config: ChunkerConfig) -> Self {
        Self { config }
    }

    pub fn with_default() -> Self {
        Self::new(ChunkerConfig::default())
    }

    /// Main entry point: chunk a document with metadata
    pub fn chunk_document(
        &self,
        content: &str,
        document_id: String,
        source: String,
        source_type: SourceType,
    ) -> Vec<Chunk> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Split into semantic units (paragraphs, sections)
        let units = self.split_into_semantic_units(content, &source_type);

        // Group units into chunks based on token limits
        let grouped_chunks = self.group_into_chunks(&units);

        // Create Chunk objects with metadata
        grouped_chunks
            .into_iter()
            .enumerate()
            .map(|(idx, (text, start_char, end_char))| {
                let token_count = self.estimate_tokens(&text);

                Chunk {
                    id: Uuid::new_v4().to_string(),
                    content: text,
                    chunk_index: idx,
                    token_count,
                    metadata: ChunkMetadata {
                        document_id: document_id.clone(),
                        source: source.clone(),
                        source_type: source_type.clone(),
                        created_at: now,
                        start_char,
                        end_char,
                        extra: HashMap::new(),
                    },
                }
            })
            .collect()
    }

    /// Split text into semantic units based on source type
    fn split_into_semantic_units(
        &self,
        content: &str,
        source_type: &SourceType,
    ) -> Vec<SemanticUnit> {
        match source_type {
            SourceType::Pdf | SourceType::Text => self.split_by_paragraphs(content),
            SourceType::Markdown => self.split_markdown(content),
            SourceType::Html => self.split_html(content),
            SourceType::Code => self.split_code(content),
        }
    }

    /// Split by paragraphs (double newline or period + newline)
    fn split_by_paragraphs(&self, content: &str) -> Vec<SemanticUnit> {
        let mut units = Vec::new();
        let mut current_pos = 0;

        // Split on paragraph boundaries
        for paragraph in content.split("\n\n") {
            let trimmed = paragraph.trim();
            if trimmed.is_empty() {
                current_pos += paragraph.len() + 2;
                continue;
            }

            // Further split long paragraphs by sentences
            if self.estimate_tokens(trimmed) > self.config.target_size {
                units.extend(self.split_by_sentences(trimmed, current_pos));
            } else {
                units.push(SemanticUnit {
                    text: trimmed.to_string(),
                    start_char: current_pos,
                    end_char: current_pos + trimmed.len(),
                    boundary_strength: BoundaryStrength::Strong,
                });
            }

            current_pos += paragraph.len() + 2;
        }

        units
    }

    /// Split by sentences for long paragraphs
    fn split_by_sentences(&self, text: &str, base_offset: usize) -> Vec<SemanticUnit> {
        let mut units = Vec::new();
        let _current_pos = 0; // or just remove it if unused

        // Simple sentence splitting on .!? followed by space and capital
        let sentence_regex = regex::Regex::new(r"([.!?]+)\s+(?=[A-Z])").unwrap();

        let mut last_end = 0;
        for mat in sentence_regex.find_iter(text) {
            let sentence = &text[last_end..mat.end()].trim();
            if !sentence.is_empty() {
                units.push(SemanticUnit {
                    text: sentence.to_string(),
                    start_char: base_offset + last_end,
                    end_char: base_offset + mat.end(),
                    boundary_strength: BoundaryStrength::Medium,
                });
            }
            last_end = mat.end();
        }

        // Add remaining text
        if last_end < text.len() {
            let sentence = text[last_end..].trim();
            if !sentence.is_empty() {
                units.push(SemanticUnit {
                    text: sentence.to_string(),
                    start_char: base_offset + last_end,
                    end_char: base_offset + text.len(),
                    boundary_strength: BoundaryStrength::Medium,
                });
            }
        }

        units
    }

    /// Split markdown by headers and paragraphs
    fn split_markdown(&self, content: &str) -> Vec<SemanticUnit> {
        let mut units = Vec::new();
        let mut current_pos = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Headers are strong boundaries
            if trimmed.starts_with('#') {
                if !trimmed.is_empty() {
                    units.push(SemanticUnit {
                        text: trimmed.to_string(),
                        start_char: current_pos,
                        end_char: current_pos + line.len(),
                        boundary_strength: BoundaryStrength::Strong,
                    });
                }
            } else if !trimmed.is_empty() {
                units.push(SemanticUnit {
                    text: trimmed.to_string(),
                    start_char: current_pos,
                    end_char: current_pos + line.len(),
                    boundary_strength: BoundaryStrength::Weak,
                });
            }

            current_pos += line.len() + 1;
        }

        units
    }

    /// Split HTML by tags (simplified - would need proper HTML parser for production)
    fn split_html(&self, content: &str) -> Vec<SemanticUnit> {
        // For now, strip tags and split by paragraphs
        let text = content.replace("<br>", "\n").replace("</p>", "\n\n");
        let cleaned = regex::Regex::new(r"<[^>]+>")
            .unwrap()
            .replace_all(&text, "");
        self.split_by_paragraphs(&cleaned)
    }

    /// Split code by functions/classes
    fn split_code(&self, content: &str) -> Vec<SemanticUnit> {
        let mut units = Vec::new();
        let mut current_pos = 0;
        let mut current_block = String::new();
        let mut block_start = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Detect function/class boundaries
            let is_boundary = trimmed.starts_with("fn ")
                || trimmed.starts_with("func ")
                || trimmed.starts_with("def ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("impl ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("async fn ");

            if is_boundary && !current_block.is_empty() {
                units.push(SemanticUnit {
                    text: current_block.trim().to_string(),
                    start_char: block_start,
                    end_char: current_pos,
                    boundary_strength: BoundaryStrength::Strong,
                });
                current_block.clear();
                block_start = current_pos;
            }

            current_block.push_str(line);
            current_block.push('\n');
            current_pos += line.len() + 1;
        }

        // Add remaining block
        if !current_block.trim().is_empty() {
            units.push(SemanticUnit {
                text: current_block.trim().to_string(),
                start_char: block_start,
                end_char: current_pos,
                boundary_strength: BoundaryStrength::Strong,
            });
        }

        units
    }

    /// Group semantic units into chunks respecting token limits
    fn group_into_chunks(&self, units: &[SemanticUnit]) -> Vec<(String, usize, usize)> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_tokens = 0;
        let mut chunk_start = 0;
        let mut chunk_end = 0;

        for unit in units {
            let unit_tokens = self.estimate_tokens(&unit.text);

            // Check if adding this unit would exceed max_size
            if current_tokens + unit_tokens > self.config.max_size && !current_chunk.is_empty() {
                // Save current chunk
                chunks.push((current_chunk.trim().to_string(), chunk_start, chunk_end));

                // Start new chunk with overlap
                let overlap_text = self.get_overlap_text(&current_chunk);
                current_chunk = overlap_text;
                current_tokens = self.estimate_tokens(&current_chunk);
                chunk_start = unit.start_char;
            }

            // Add unit to current chunk
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(&unit.text);
            current_tokens += unit_tokens;
            chunk_end = unit.end_char;

            // If we've reached target size at a strong boundary, save chunk
            if current_tokens >= self.config.target_size
                && matches!(unit.boundary_strength, BoundaryStrength::Strong)
            {
                chunks.push((current_chunk.trim().to_string(), chunk_start, chunk_end));
                current_chunk.clear();
                current_tokens = 0;
                chunk_start = chunk_end;
            }
        }

        // Add final chunk
        if !current_chunk.trim().is_empty() {
            chunks.push((current_chunk.trim().to_string(), chunk_start, chunk_end));
        }

        chunks
    }

    /// Get last N tokens for overlap
    fn get_overlap_text(&self, text: &str) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        let overlap_words = (self.config.overlap * 3 / 4).min(words.len()); // ~0.75 tokens per word

        words[words.len().saturating_sub(overlap_words)..].join(" ")
    }

    /// Estimate token count (rough approximation: 1 token ≈ 4 chars or 0.75 words)
    fn estimate_tokens(&self, text: &str) -> usize {
        let char_estimate = text.len() / 4;
        let word_estimate = text.split_whitespace().count() * 4 / 3;
        (char_estimate + word_estimate) / 2
    }
}

#[derive(Debug, Clone)]
struct SemanticUnit {
    text: String,
    start_char: usize,
    end_char: usize,
    boundary_strength: BoundaryStrength,
}

#[derive(Debug, Clone)]
enum BoundaryStrength {
    Strong, // Paragraph, header, function
    Medium, // Sentence
    Weak,   // Line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_chunking() {
        let chunker = SemanticChunker::with_default();
        let content = "This is paragraph one. It has multiple sentences.\n\nThis is paragraph two. It's a bit longer and has more content to test the chunking logic.";

        let chunks = chunker.chunk_document(
            content,
            "doc1".to_string(),
            "test.txt".to_string(),
            SourceType::Text,
        );

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].chunk_index, 0);
        assert!(chunks[0].token_count > 0);
    }

    #[test]
    fn test_markdown_chunking() {
        let chunker = SemanticChunker::with_default();
        let content = "# Header 1\n\nSome content here.\n\n## Header 2\n\nMore content.";

        let chunks = chunker.chunk_document(
            content,
            "doc2".to_string(),
            "test.md".to_string(),
            SourceType::Markdown,
        );

        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_token_estimation() {
        let chunker = SemanticChunker::with_default();
        let text = "The quick brown fox jumps over the lazy dog";
        let tokens = chunker.estimate_tokens(text);

        // Should be roughly 9-12 tokens for this sentence
        assert!(tokens >= 8 && tokens <= 15);
    }
}
