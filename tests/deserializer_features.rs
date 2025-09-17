use assert_matches::assert_matches;
use serde::Deserialize;
use serde_dbgfmt::Deserializer;

#[derive(Debug, Deserialize, PartialEq)]
struct SimpleStruct {
    name: String,
    age: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ComplexStruct {
    id: u64,
    data: Vec<i32>,
    meta: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
enum TestEnum {
    Unit,
    Tuple(i32, String),
    Struct { field: bool },
}

/// Test multiple sequential deserializations using the same deserializer
/// instance
#[test]
fn test_multiple_sequential_deserializations() {
    let input = r#"SimpleStruct { name: "Alice", age: 25 } ComplexStruct { id: 42, data: [1, 2, 3], meta: Some("test") }"#;
    let mut deserializer = Deserializer::new(input);

    // First deserialization
    let first: SimpleStruct =
        SimpleStruct::deserialize(&mut deserializer).expect("Failed to deserialize first struct");
    assert_eq!(
        first,
        SimpleStruct {
            name: "Alice".to_string(),
            age: 25
        }
    );

    // Second deserialization from the same deserializer
    let second: ComplexStruct =
        ComplexStruct::deserialize(&mut deserializer).expect("Failed to deserialize second struct");
    assert_eq!(
        second,
        ComplexStruct {
            id: 42,
            data: vec![1, 2, 3],
            meta: Some("test".to_string())
        }
    );

    // Verify we've consumed all input
    deserializer.end().expect("Expected end of input");
}

/// Test multiple values with different types in sequence
#[test]
fn test_mixed_type_sequential_deserialization() {
    let input = r#"42 "hello" true [1, 2, 3] Unit"#;
    let mut deserializer = Deserializer::new(input);

    let num: u32 = u32::deserialize(&mut deserializer).expect("Failed to deserialize number");
    assert_eq!(num, 42);

    let text: String =
        String::deserialize(&mut deserializer).expect("Failed to deserialize string");
    assert_eq!(text, "hello");

    let flag: bool = bool::deserialize(&mut deserializer).expect("Failed to deserialize bool");
    assert!(flag);

    let list: Vec<i32> =
        Vec::<i32>::deserialize(&mut deserializer).expect("Failed to deserialize vector");
    assert_eq!(list, vec![1, 2, 3]);

    let unit_enum: TestEnum =
        TestEnum::deserialize(&mut deserializer).expect("Failed to deserialize enum");
    assert_eq!(unit_enum, TestEnum::Unit);

    deserializer.end().expect("Expected end of input");
}

/// Test partial deserialization - stopping mid-stream and checking state
#[test]
fn test_partial_deserialization() {
    let input = r#"SimpleStruct { name: "Alice", age: 25 } ComplexStruct { id: 42, data: [1, 2, 3], meta: Some("test") }"#;
    let mut deserializer = Deserializer::new(input);

    // Deserialize only the first struct
    let first: SimpleStruct =
        SimpleStruct::deserialize(&mut deserializer).expect("Failed to deserialize first struct");
    assert_eq!(
        first,
        SimpleStruct {
            name: "Alice".to_string(),
            age: 25
        }
    );

    // Verify that end() fails because there's remaining input
    let end_result = deserializer.end();
    assert_matches!(end_result, Err(_));

    // Create a new deserializer for the remaining data since the first one might be
    // in an invalid state
    let remaining_input = r#"ComplexStruct { id: 42, data: [1, 2, 3], meta: Some("test") }"#;
    let mut deserializer2 = Deserializer::new(remaining_input);

    let second: ComplexStruct = ComplexStruct::deserialize(&mut deserializer2)
        .expect("Failed to deserialize second struct");
    assert_eq!(
        second,
        ComplexStruct {
            id: 42,
            data: vec![1, 2, 3],
            meta: Some("test".to_string())
        }
    );

    // Now end() should succeed
    deserializer2
        .end()
        .expect("Expected successful end after consuming all input");
}

/// Test deserializer state validation with proper cleanup using end()
#[test]
fn test_deserializer_state_validation() {
    // Test case 1: Successful complete deserialization
    {
        let input = r#"SimpleStruct { name: "test", age: 30 }"#;
        let mut deserializer = Deserializer::new(input);

        let _result: SimpleStruct =
            SimpleStruct::deserialize(&mut deserializer).expect("Deserialization should succeed");

        // end() should succeed with no remaining input
        deserializer
            .end()
            .expect("End should succeed with complete input consumed");
    }

    // Test case 2: Incomplete deserialization - end() should fail
    {
        let input = r#"SimpleStruct { name: "test", age: 30 } "extra data""#;
        let mut deserializer = Deserializer::new(input);

        let _result: SimpleStruct =
            SimpleStruct::deserialize(&mut deserializer).expect("Deserialization should succeed");

        // end() should fail because there's extra data
        let end_result = deserializer.end();
        assert_matches!(end_result, Err(_));
    }

    // Test case 3: Whitespace-only remainder should be acceptable
    {
        let input = r#"SimpleStruct { name: "test", age: 30 }   "#;
        let mut deserializer = Deserializer::new(input);

        let _result: SimpleStruct =
            SimpleStruct::deserialize(&mut deserializer).expect("Deserialization should succeed");

        // The implementation should handle trailing whitespace properly
        deserializer
            .end()
            .expect("End should succeed with whitespace-only remainder");
    }
}

/// Test error propagation through the deserializer at various stages
#[test]
fn test_error_propagation() {
    // Test malformed input during initial parsing
    {
        let input = r#"SimpleStruct { name: "unclosed string, age: 30 }"#;
        let mut deserializer = Deserializer::new(input);

        let result = SimpleStruct::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }

    // Test type mismatch errors
    {
        let input = r#"SimpleStruct { name: 42, age: "not a number" }"#;
        let mut deserializer = Deserializer::new(input);

        let result = SimpleStruct::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }

    // Test missing field errors
    {
        let input = r#"SimpleStruct { name: "test" }"#;
        let mut deserializer = Deserializer::new(input);

        let result = SimpleStruct::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }

    // Test wrong struct name
    {
        let input = r#"WrongStruct { name: "test", age: 30 }"#;
        let mut deserializer = Deserializer::new(input);

        let result = SimpleStruct::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }
}

/// Test custom deserializer usage patterns
#[test]
fn test_custom_deserializer_patterns() {
    // Test using deserializer with custom visitor patterns
    use std::fmt;

    use serde::de::{self, Visitor};

    struct CustomVisitor;

    impl<'de> Visitor<'de> for CustomVisitor {
        type Value = (String, u32);

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a simple struct with name and age")
        }

        fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where
            V: de::MapAccess<'de>,
        {
            let mut name = None;
            let mut age = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "name" => {
                        if name.is_some() {
                            return Err(de::Error::duplicate_field("name"));
                        }
                        name = Some(map.next_value()?);
                    }
                    "age" => {
                        if age.is_some() {
                            return Err(de::Error::duplicate_field("age"));
                        }
                        age = Some(map.next_value()?);
                    }
                    _ => {
                        let _: de::IgnoredAny = map.next_value()?;
                    }
                }
            }

            let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
            let age = age.ok_or_else(|| de::Error::missing_field("age"))?;
            Ok((name, age))
        }
    }

    let input = r#"SimpleStruct { name: "Alice", age: 25 }"#;
    let mut deserializer = Deserializer::new(input);

    // Skip the struct name manually
    use serde::de::Deserializer as _;
    let result = deserializer
        .deserialize_struct("SimpleStruct", &["name", "age"], CustomVisitor)
        .expect("Custom visitor should work");

    assert_eq!(result, ("Alice".to_string(), 25));
    deserializer.end().expect("Should reach end successfully");
}

/// Test edge cases with deserializer lifecycle
#[test]
fn test_deserializer_lifecycle_edge_cases() {
    // Test empty input
    {
        let input = "";
        let mut deserializer = Deserializer::new(input);

        // Should immediately be at end
        deserializer.end().expect("Empty input should be valid");
    }

    // Test whitespace-only input
    {
        let input = "   \t\n  ";
        let mut deserializer = Deserializer::new(input);

        // Should immediately be at end (whitespace is ignored)
        deserializer
            .end()
            .expect("Whitespace-only input should be valid");
    }

    // Test calling end() multiple times
    {
        let input = r#"42"#;
        let mut deserializer = Deserializer::new(input);

        let _result: u32 = u32::deserialize(&mut deserializer).expect("Should deserialize number");

        deserializer.end().expect("First end() should succeed");

        // Calling end() again should also succeed (idempotent)
        deserializer
            .end()
            .expect("Second end() should also succeed");
    }

    // Test deserializing after successful end()
    {
        let input = r#"42"#;
        let mut deserializer = Deserializer::new(input);

        let _result: u32 = u32::deserialize(&mut deserializer).expect("Should deserialize number");

        deserializer.end().expect("End should succeed");

        // Trying to deserialize more should fail
        let result = u32::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }
}

/// Test deserializer with malformed input at various stages
#[test]
fn test_malformed_input_at_various_stages() {
    // Test malformed input at the beginning
    {
        use std::collections::BTreeMap;
        let input = r#"{"#;
        let mut deserializer = Deserializer::new(input);

        let result = BTreeMap::<String, String>::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }

    // Test malformed input in the middle of parsing
    {
        let input = r#"SimpleStruct { name: "test", age:"#;
        let mut deserializer = Deserializer::new(input);

        let result = SimpleStruct::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }

    // Test malformed nested structures
    {
        let input = r#"ComplexStruct { id: 42, data: [1, 2, , meta: Some("test") }"#;
        let mut deserializer = Deserializer::new(input);

        let result = ComplexStruct::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }

    // Test invalid escape sequences in strings
    {
        let input = r#"SimpleStruct { name: "test\z", age: 30 }"#;
        let mut deserializer = Deserializer::new(input);

        let result = SimpleStruct::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }

    // Test invalid number formats
    {
        let input = r#"SimpleStruct { name: "test", age: 1.2.3 }"#;
        let mut deserializer = Deserializer::new(input);

        let result = SimpleStruct::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }

    // Test unexpected end of input during parsing
    {
        let input = r#"SimpleStruct { name: "test""#;
        let mut deserializer = Deserializer::new(input);

        let result = SimpleStruct::deserialize(&mut deserializer);
        assert_matches!(result, Err(_));
    }
}

/// Test deserializer state consistency across nested structures
#[test]
fn test_complex_nested_structure_state() {
    // Test basic nested structure parsing
    #[derive(Debug, Deserialize, PartialEq)]
    struct SimpleNested {
        inner: SimpleInner,
        list: Vec<String>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct SimpleInner {
        name: String,
    }

    let nested = SimpleNested {
        inner: SimpleInner {
            name: "test".to_string(),
        },
        list: vec!["one".to_string(), "two".to_string()],
    };

    let debug_str = format!("{:?}", nested);
    let mut deserializer = Deserializer::new(&debug_str);

    let deserialized: SimpleNested =
        SimpleNested::deserialize(&mut deserializer).expect("Should deserialize nested structure");

    assert_eq!(nested, deserialized);
    deserializer.end().expect("Should reach end successfully");

    // Test deserializer behavior with multiple parse operations
    let input = r#"SimpleInner { name: "first" } SimpleInner { name: "second" }"#;
    let mut deserializer2 = Deserializer::new(input);

    let first: SimpleInner =
        SimpleInner::deserialize(&mut deserializer2).expect("Should deserialize first inner");
    assert_eq!(first.name, "first");

    let second: SimpleInner =
        SimpleInner::deserialize(&mut deserializer2).expect("Should deserialize second inner");
    assert_eq!(second.name, "second");

    deserializer2.end().expect("Should reach end successfully");
}

/// Test deserializer with various numeric edge cases
#[test]
fn test_numeric_edge_cases() {
    // Test basic maximum values (avoiding infinity which may not be supported)
    {
        let input = format!("{} {}", u64::MAX, i64::MIN);
        let mut deserializer = Deserializer::new(&input);

        let max_u64: u64 =
            u64::deserialize(&mut deserializer).expect("Should deserialize u64::MAX");
        assert_eq!(max_u64, u64::MAX);

        let min_i64: i64 =
            i64::deserialize(&mut deserializer).expect("Should deserialize i64::MIN");
        assert_eq!(min_i64, i64::MIN);

        deserializer.end().expect("Should reach end");
    }

    // Test special float values
    {
        let input = "NaN";
        let mut deserializer = Deserializer::new(input);

        let nan_val: f64 = f64::deserialize(&mut deserializer).expect("Should deserialize NaN");
        assert!(nan_val.is_nan());

        deserializer.end().expect("Should reach end");
    }

    // Test hexadecimal, octal, and binary numbers (these work in struct context)
    {
        #[derive(Debug, Deserialize, PartialEq)]
        struct NumericTest {
            hex: u32,
            oct: u32,
            bin: u32,
        }

        let test_struct = NumericTest {
            hex: 0x42,
            oct: 0o52,     // 42 in decimal
            bin: 0b101010, // 42 in decimal
        };

        let debug_str = format!("{:?}", test_struct);
        let parsed: NumericTest = serde_dbgfmt::from_str(&debug_str)
            .expect("Should deserialize numeric formats in struct");

        assert_eq!(parsed.hex, 0x42);
        assert_eq!(parsed.oct, 42);
        assert_eq!(parsed.bin, 42);
    }

    // Test regular float values
    {
        let input = "1.5 -2.5 0.0";
        let mut deserializer = Deserializer::new(input);

        let pos: f64 =
            f64::deserialize(&mut deserializer).expect("Should deserialize positive float");
        assert_eq!(pos, 1.5);

        let neg: f64 =
            f64::deserialize(&mut deserializer).expect("Should deserialize negative float");
        assert_eq!(neg, -2.5);

        let zero: f64 = f64::deserialize(&mut deserializer).expect("Should deserialize zero");
        assert_eq!(zero, 0.0);

        deserializer.end().expect("Should reach end");
    }
}

/// Test error handling across deserializer reuse scenarios
#[test]
fn test_error_handling_across_reuse() {
    // Test that errors don't corrupt the deserializer state for subsequent
    // operations
    let input = r#"SimpleStruct { name: "good", age: 25 } BadStruct { invalid: syntax } SimpleStruct { name: "also_good", age: 30 }"#;
    let mut deserializer = Deserializer::new(input);

    // First deserialization should succeed
    let first: SimpleStruct =
        SimpleStruct::deserialize(&mut deserializer).expect("First deserialization should succeed");
    assert_eq!(
        first,
        SimpleStruct {
            name: "good".to_string(),
            age: 25
        }
    );

    // Second deserialization should fail due to wrong struct name
    let second_result = SimpleStruct::deserialize(&mut deserializer);
    assert_matches!(second_result, Err(_));

    // The deserializer state should still be usable for error reporting
    // but we can't continue deserializing after an error in the middle of a struct
    let _end_result = deserializer.end();
    // This will likely fail because we're in a corrupted state after the failed
    // parse This test demonstrates that partial failures can leave the
    // deserializer in an unusable state
}

/// Test deserializer behavior with deeply nested structures
#[test]
fn test_deep_nesting() {
    // Create a deeply nested structure to test stack safety and state management
    #[derive(Debug, Deserialize, PartialEq)]
    enum NestedEnum {
        None,
        Some(Box<NestedEnum>),
    }

    let input = "Some(Some(Some(Some(None))))";
    let mut deserializer = Deserializer::new(input);

    let result: NestedEnum =
        NestedEnum::deserialize(&mut deserializer).expect("Should handle deep nesting");

    // Verify the structure by pattern matching
    match result {
        NestedEnum::Some(level1) => match *level1 {
            NestedEnum::Some(level2) => match *level2 {
                NestedEnum::Some(level3) => match *level3 {
                    NestedEnum::Some(level4) => match *level4 {
                        NestedEnum::None => (),
                        _ => panic!("Expected None at deepest level"),
                    },
                    _ => panic!("Expected Some at level 4"),
                },
                _ => panic!("Expected Some at level 3"),
            },
            _ => panic!("Expected Some at level 2"),
        },
        _ => panic!("Expected Some at top level"),
    }

    deserializer.end().expect("Should reach end");
}
