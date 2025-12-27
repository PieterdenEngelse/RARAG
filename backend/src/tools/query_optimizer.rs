// src/tools/query_optimizer.rs
use std::collections::HashSet;

pub struct QueryOptimizer;

impl QueryOptimizer {
    /// Normalize query for caching
    pub fn normalize(query: &str) -> String {
        query
            .trim()
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Calculate string similarity (0.0 to 1.0)
    pub fn similarity(a: &str, b: &str) -> f32 {
        let a_norm = Self::normalize(a);
        let b_norm = Self::normalize(b);

        if a_norm == b_norm {
            return 1.0;
        }

        let a_words: HashSet<_> = a_norm.split_whitespace().collect();
        let b_words: HashSet<_> = b_norm.split_whitespace().collect();

        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();

        intersection as f32 / union as f32
    }

    /// Check if queries should use same cache
    pub fn are_similar(q1: &str, q2: &str, threshold: f32) -> bool {
        Self::similarity(q1, q2) > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(
            QueryOptimizer::normalize("  FIND   papers  "),
            "find papers"
        );
    }

    #[test]
    fn test_similarity() {
        assert_eq!(
            QueryOptimizer::similarity("find papers", "find papers"),
            1.0
        );
        assert!(QueryOptimizer::similarity("find papers", "find papers and count") >= 0.5);
    }
}
