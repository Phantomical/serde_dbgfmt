use assert_matches::assert_matches;

#[test]
fn test_integer_overflow() {
    let result: Result<u8, _> = serde_dbgfmt::from_str("256");
    assert_matches!(result, Err(_));
    let error = result.unwrap_err();
    assert!(error.to_string().contains("invalid integer literal"));
}

#[test]
fn test_missing_braces() {
    let result: Result<serde::de::IgnoredAny, _> = serde_dbgfmt::from_str("Test { a: 1");
    assert_matches!(result, Err(_));
}

#[test]
fn test_unterminated_string() {
    let result: Result<serde::de::IgnoredAny, _> = serde_dbgfmt::from_str("\"hello");
    assert_matches!(result, Err(_));
}

#[test]
fn test_invalid_escape() {
    let result: Result<String, _> = serde_dbgfmt::from_str("\"\\z\"");
    assert_matches!(result, Err(_));
    let error = result.unwrap_err();
    assert!(error.to_string().contains("invalid escape"));
}

#[test]
fn test_simple_parse_works() {
    let result: Result<i32, _> = serde_dbgfmt::from_str("42");
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_simple_string_works() {
    let result: Result<String, _> = serde_dbgfmt::from_str("\"hello\"");
    assert_eq!(result.unwrap(), "hello");
}
