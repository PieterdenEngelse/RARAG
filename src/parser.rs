pub fn clean_text(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace("\t", " ")
        .split('\n')
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
