use tantivy::schema::Document;

pub fn test_func<T: Document>(_doc: &T) {
    // This compiles correctly with Tantivy ≥0.24
}