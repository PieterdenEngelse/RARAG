#[test]
fn test_parse_buckets_env_valid() {
    std::env::set_var("SEARCH_HISTO_BUCKETS", "5, 10, 1, 20");
    let parsed = ag::monitoring::metrics::__test_parse_buckets_env("SEARCH_HISTO_BUCKETS");
    assert_eq!(parsed, Some(vec![1.0, 5.0, 10.0, 20.0]));
    std::env::remove_var("SEARCH_HISTO_BUCKETS");
}

#[test]
fn test_parse_buckets_env_invalid_token_falls_back() {
    std::env::set_var("REINDEX_HISTO_BUCKETS", "50, abc, 100");
    let parsed = ag::monitoring::metrics::__test_parse_buckets_env("REINDEX_HISTO_BUCKETS");
    // invalid token causes None (fallback to defaults at call site)
    assert_eq!(parsed, None);
    std::env::remove_var("REINDEX_HISTO_BUCKETS");
}

#[test]
fn test_parse_buckets_env_empty_falls_back() {
    std::env::set_var("SEARCH_HISTO_BUCKETS", "   ");
    let parsed = ag::monitoring::metrics::__test_parse_buckets_env("SEARCH_HISTO_BUCKETS");
    assert_eq!(parsed, None);
    std::env::remove_var("SEARCH_HISTO_BUCKETS");
}
