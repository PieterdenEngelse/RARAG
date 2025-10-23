/// Splits cleaned text into ~500–800 character chunks for small local models.
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
