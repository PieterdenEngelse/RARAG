// src/tools/result_compressor.rs

pub struct ResultCompressor;

impl ResultCompressor {
    const MAX_RESULT_LENGTH: usize = 500;

    /// Compress result if too large
    pub fn compress(result: &str) -> String {
        if result.len() > Self::MAX_RESULT_LENGTH {
            format!("{}...", &result[..Self::MAX_RESULT_LENGTH])
        } else {
            result.to_string()
        }
    }

    /// Get compression ratio
    pub fn ratio(original: &str, compressed: &str) -> f32 {
        compressed.len() as f32 / original.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress() {
        let long = "a".repeat(1000);
        let compressed = ResultCompressor::compress(&long);
        assert!(compressed.len() < long.len());
    }
}