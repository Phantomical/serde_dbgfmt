use assert_matches::assert_matches;
use serde::Deserialize;
use serde_dbgfmt::{Deserializer, Error};

#[derive(Debug, Deserialize, PartialEq)]
struct TestStruct {
    field: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ComplexStruct {
    name: String,
    value: Option<i32>,
    flag: bool,
}

#[test]
fn test_malformed_struct_missing_brace() {
    let input = "TestStruct { field: 42";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_malformed_struct_extra_brace() {
    let input = "TestStruct { field: 42 }}";
    let mut de = Deserializer::new(input);
    let result: Result<TestStruct, Error> = serde::Deserialize::deserialize(&mut de);
    assert_matches!(result, Ok(_));
    let end_result = de.end();
    assert_matches!(end_result, Err(_));
}

#[test]
fn test_unmatched_brackets() {
    let input = "[1, 2, 3";
    let result: Result<Vec<u32>, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_unmatched_brackets_extra() {
    let input = "[1, 2, 3]]";
    let mut de = Deserializer::new(input);
    let result: Result<Vec<u32>, Error> = serde::Deserialize::deserialize(&mut de);
    assert_matches!(result, Ok(_));
    let end_result = de.end();
    assert_matches!(end_result, Err(_));
}

#[test]
fn test_unterminated_string() {
    let input = r#"ComplexStruct { name: "unterminated, value: Some(42), flag: true }"#;
    let result: Result<ComplexStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_unterminated_char() {
    let input = "TestStruct { field: 'a }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_invalid_escape_sequence() {
    let input = r#"ComplexStruct { name: "\q", value: Some(42), flag: true }"#;
    let result: Result<ComplexStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_incomplete_unicode_escape() {
    let input = r#"ComplexStruct { name: "\u{123", value: Some(42), flag: true }"#;
    let result: Result<ComplexStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_invalid_unicode_codepoint() {
    let input = r#"ComplexStruct { name: "\u{999999}", value: Some(42), flag: true }"#;
    let result: Result<ComplexStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_integer_overflow_u8() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct TestU8 {
        value: u8,
    }

    let input = "TestU8 { field: 256 }";
    let mut de = Deserializer::new(input);
    let result: Result<TestU8, Error> = serde::Deserialize::deserialize(&mut de);
    assert_matches!(result, Err(_));
}

#[test]
fn test_integer_underflow_i8() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct TestI8 {
        value: i8,
    }

    let input = "TestI8 { value: -129 }";
    let result: Result<TestI8, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_negative_unsigned_integer() {
    let input = "TestStruct { field: -1 }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_empty_input() {
    let input = "";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_whitespace_only_input() {
    let input = "   \t\n  ";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_unexpected_token() {
    let input = "TestStruct @ field: 42 }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_incomplete_struct() {
    let input = "TestStruct {";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_missing_field_value() {
    let input = "TestStruct { field: }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_missing_colon() {
    let input = "TestStruct { field 42 }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_invalid_bool() {
    let input = "ComplexStruct { name: \"test\", value: Some(42), flag: maybe }";
    let result: Result<ComplexStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_invalid_option_variant() {
    let input = "ComplexStruct { name: \"test\", value: Maybe(42), flag: true }";
    let result: Result<ComplexStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_float_edge_cases_valid() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct FloatTest {
        value: f64,
    }

    let input = "FloatTest { value: 1.7976931348623157e308 }";
    let result: Result<FloatTest, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));

    let input = "FloatTest { value: 2.2250738585072014e-308 }";
    let result: Result<FloatTest, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
}

#[test]
fn test_malformed_number() {
    let input = "TestStruct { field: 12.34.56 }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_invalid_hex_digit() {
    let input = "TestStruct { field: 0xGHI }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_incomplete_float() {
    let input = "TestStruct { field: 42. }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_incomplete_scientific_notation() {
    let input = "TestStruct { field: 42e }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_unmatched_parentheses() {
    let input = "Some(42";
    let result: Result<Option<u32>, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_extra_comma() {
    let input = "[1, 2, 3,]";
    let result: Result<Vec<u32>, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
}

#[test]
fn test_valid_edge_cases() {
    let input = "TestStruct { field: 0 }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    assert_eq!(result.unwrap().field, 0);

    #[derive(Debug, Deserialize, PartialEq)]
    struct MaxTest {
        value: u32,
    }

    let input = "MaxTest { value: 4294967295 }";
    let result: Result<MaxTest, Error> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    assert_eq!(result.unwrap().value, u32::MAX);
}

#[test]
fn test_complex_error_recovery() {
    let inputs = [
        "TestStruct { field: }",
        "TestStruct field: 42 }",
        "TestStruct { field 42 }",
        "TestStruct { field: 42",
        "{ field: 42 }",
        "TestStruct { field: abc }",
    ];

    for input in inputs {
        let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
        assert_matches!(result, Err(_), "Expected error for input: {}", input);
    }
}

#[test]
fn test_error_message_quality() {
    let input = "TestStruct { field: abc }";
    let result: Result<TestStruct, Error> = serde_dbgfmt::from_str(input);
    match result {
        Err(error) => {
            let error_string = error.to_string();
            assert!(!error_string.is_empty());
        }
        Ok(_) => panic!("Expected error for invalid input"),
    }
}
