use std::env;

/// Default max characters per chunk (optimized for phi/small models)
/// Can be overridden with CHUNK_MAX_CHARS environment variable
pub const DEFAULT_MAX_CHARS: usize = 1500; // ~375 tokens at 4 chars/token

/// Get max chars from environment or use default
pub fn get_max_chars() -> usize {
    env::var("CHUNK_MAX_CHARS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_CHARS)
}

/// Splits cleaned text into chunks for small local models.
/// Default: ~1500 characters (~375 tokens) optimized for phi model.
/// Configure with CHUNK_MAX_CHARS environment variable.
pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for paragraph in text.split("\n\n") {
        let para = paragraph.trim();
        if para.is_empty() {
            continue;
        }

        if current.len() + para.len() + 2 <= max_chars {
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(para);
        } else {
            if !current.is_empty() {
                chunks.push(current.clone());
                current.clear();
            }

            if para.len() > max_chars {
                // Split long paragraph into sentences
                for sentence in para.split('.').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                    if current.len() + sentence.len() + 2 <= max_chars {
                        if !current.is_empty() {
                            current.push_str(". ");
                        }
                        current.push_str(sentence);
                    } else {
                        if !current.is_empty() {
                            current.push('.');
                            chunks.push(current.clone());
                            current.clear();
                        }
                        current.push_str(sentence);
                    }
                }
                if !current.is_empty() {
                    current.push('.');
                    chunks.push(current.clone());
                    current.clear();
                }
            } else {
                chunks.push(para.to_string());
            }
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}
