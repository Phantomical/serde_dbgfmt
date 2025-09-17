use std::collections::{BTreeMap, BTreeSet};

use assert_matches::assert_matches;
use pretty_assertions::assert_eq;
use serde::Deserialize;
use serde_dbgfmt::{from_str, Error};

#[derive(Debug, Deserialize, PartialEq)]
struct SimpleStruct {
    a: u32,
    b: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct NestedStruct {
    inner: SimpleStruct,
    flag: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
enum TestEnum {
    Unit,
    Tuple(i32, String),
    Struct { x: f64, y: f64 },
}

#[derive(Debug, Deserialize, PartialEq)]
struct CollectionStruct {
    vec: Vec<i32>,
    map: BTreeMap<String, i32>,
    set: BTreeSet<String>,
}

// Test various whitespace patterns in structs
#[test]
fn test_struct_with_various_whitespace() {
    // Normal formatting
    let input = r#"SimpleStruct { a: 42, b: "hello" }"#;
    let result: SimpleStruct = from_str(input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");

    // Extra spaces around punctuation
    let input = r#"SimpleStruct  {  a  :  42  ,  b  :  "hello"  }"#;
    let result: SimpleStruct = from_str(input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");

    // Mixed tabs and spaces
    let input = "SimpleStruct\t{\ta\t:\t42\t,\tb\t:\t\"hello\"\t}";
    let result: SimpleStruct = from_str(input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");

    // Leading/trailing whitespace
    let input = r#"   SimpleStruct { a: 42, b: "hello" }   "#;
    let result: SimpleStruct = from_str(input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");
}

#[test]
fn test_struct_with_newlines() {
    // Newlines between fields
    let input = r#"SimpleStruct {
        a: 42,
        b: "hello"
    }"#;
    let result: SimpleStruct = from_str(input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");

    // Newlines in various places
    let input = r#"SimpleStruct
    {
        a:
            42,
        b:
            "hello"
    }"#;
    let result: SimpleStruct = from_str(input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");
}

#[test]
fn test_struct_with_trailing_comma() {
    // Trailing comma after last field
    let input = r#"SimpleStruct { a: 42, b: "hello", }"#;
    let result: SimpleStruct = from_str(input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");

    // Trailing comma with extra whitespace
    let input = r#"SimpleStruct { a: 42, b: "hello" ,   }"#;
    let result: SimpleStruct = from_str(input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");
}

#[test]
fn test_nested_struct_whitespace() {
    let input = r#"NestedStruct {
        inner: SimpleStruct {
            a: 100,
            b: "nested"
        },
        flag: true
    }"#;
    let result: NestedStruct = from_str(input).unwrap();
    assert_eq!(result.inner.a, 100);
    assert_eq!(result.inner.b, "nested");
    assert_eq!(result.flag, true);

    // Compact nested struct
    let input = r#"NestedStruct{inner:SimpleStruct{a:100,b:"nested"},flag:true}"#;
    let result: NestedStruct = from_str(input).unwrap();
    assert_eq!(result.inner.a, 100);
    assert_eq!(result.inner.b, "nested");
    assert_eq!(result.flag, true);
}

#[test]
fn test_enum_whitespace_patterns() {
    // Unit variant
    let input = "Unit";
    let result: TestEnum = from_str(input).unwrap();
    assert_eq!(result, TestEnum::Unit);

    // Unit variant with whitespace
    let input = "  Unit  ";
    let result: TestEnum = from_str(input).unwrap();
    assert_eq!(result, TestEnum::Unit);

    // Tuple variant with various whitespace
    let input = "Tuple( 42 , \"test\" )";
    let result: TestEnum = from_str(input).unwrap();
    assert_eq!(result, TestEnum::Tuple(42, "test".to_string()));

    // Struct variant with whitespace
    let input = r#"Struct  {  x  :  1.5  ,  y  :  2.5  }"#;
    let result: TestEnum = from_str(input).unwrap();
    assert_eq!(result, TestEnum::Struct { x: 1.5, y: 2.5 });

    // Struct variant with trailing comma
    let input = r#"Struct { x: 1.5, y: 2.5, }"#;
    let result: TestEnum = from_str(input).unwrap();
    assert_eq!(result, TestEnum::Struct { x: 1.5, y: 2.5 });
}

#[test]
fn test_vector_whitespace_patterns() {
    // Normal vector
    let input = "[1, 2, 3, 4]";
    let result: Vec<i32> = from_str(input).unwrap();
    assert_eq!(result, vec![1, 2, 3, 4]);

    // Vector with extra spaces
    let input = "[ 1 , 2 , 3 , 4 ]";
    let result: Vec<i32> = from_str(input).unwrap();
    assert_eq!(result, vec![1, 2, 3, 4]);

    // Vector with trailing comma
    let input = "[1, 2, 3, 4,]";
    let result: Vec<i32> = from_str(input).unwrap();
    assert_eq!(result, vec![1, 2, 3, 4]);

    // Vector with newlines
    let input = r#"[
        1,
        2,
        3,
        4
    ]"#;
    let result: Vec<i32> = from_str(input).unwrap();
    assert_eq!(result, vec![1, 2, 3, 4]);

    // Empty vector
    let input = "[]";
    let result: Vec<i32> = from_str(input).unwrap();
    assert_eq!(result, Vec::<i32>::new());

    // Empty vector with whitespace
    let input = "[  ]";
    let result: Vec<i32> = from_str(input).unwrap();
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn test_btreemap_whitespace_patterns() {
    // Normal BTreeMap formatting
    let input = r#"{"a": 1, "b": 2}"#;
    let result: BTreeMap<String, i32> = from_str(input).unwrap();
    let mut expected = BTreeMap::new();
    expected.insert("a".to_string(), 1);
    expected.insert("b".to_string(), 2);
    assert_eq!(result, expected);

    // BTreeMap with extra spaces
    let input = r#"{  "a"  :  1  ,  "b"  :  2  }"#;
    let result: BTreeMap<String, i32> = from_str(input).unwrap();
    assert_eq!(result, expected);

    // BTreeMap with trailing comma
    let input = r#"{"a": 1, "b": 2,}"#;
    let result: BTreeMap<String, i32> = from_str(input).unwrap();
    assert_eq!(result, expected);

    // BTreeMap with newlines
    let input = r#"{
        "a": 1,
        "b": 2
    }"#;
    let result: BTreeMap<String, i32> = from_str(input).unwrap();
    assert_eq!(result, expected);

    // Empty BTreeMap
    let input = "{}";
    let result: BTreeMap<String, i32> = from_str(input).unwrap();
    assert_eq!(result, BTreeMap::new());

    // Empty BTreeMap with whitespace
    let input = "{  }";
    let result: BTreeMap<String, i32> = from_str(input).unwrap();
    assert_eq!(result, BTreeMap::new());
}

#[test]
fn test_btreeset_whitespace_patterns() {
    // Normal BTreeSet formatting
    let input = r#"{"a", "b", "c"}"#;
    let result: BTreeSet<String> = from_str(input).unwrap();
    let mut expected = BTreeSet::new();
    expected.insert("a".to_string());
    expected.insert("b".to_string());
    expected.insert("c".to_string());
    assert_eq!(result, expected);

    // BTreeSet with extra spaces
    let input = r#"{  "a"  ,  "b"  ,  "c"  }"#;
    let result: BTreeSet<String> = from_str(input).unwrap();
    assert_eq!(result, expected);

    // BTreeSet with trailing comma
    let input = r#"{"a", "b", "c",}"#;
    let result: BTreeSet<String> = from_str(input).unwrap();
    assert_eq!(result, expected);

    // BTreeSet with newlines
    let input = r#"{
        "a",
        "b",
        "c"
    }"#;
    let result: BTreeSet<String> = from_str(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_option_whitespace_patterns() {
    // Some variant with spaces
    let input = "Some( 42 )";
    let result: Option<i32> = from_str(input).unwrap();
    assert_eq!(result, Some(42));

    // None variant with spaces
    let input = "  None  ";
    let result: Option<i32> = from_str(input).unwrap();
    assert_eq!(result, None);

    // Nested Some with complex whitespace
    let input = r#"Some(  SimpleStruct  {  a:  42,  b:  "test"  }  )"#;
    let result: Option<SimpleStruct> = from_str(input).unwrap();
    assert_eq!(
        result,
        Some(SimpleStruct {
            a: 42,
            b: "test".to_string()
        })
    );
}

#[test]
fn test_tuple_whitespace_patterns() {
    // Normal tuple
    let input = "(1, 2, 3)";
    let result: (i32, i32, i32) = from_str(input).unwrap();
    assert_eq!(result, (1, 2, 3));

    // Tuple with extra spaces
    let input = "( 1 , 2 , 3 )";
    let result: (i32, i32, i32) = from_str(input).unwrap();
    assert_eq!(result, (1, 2, 3));

    // Tuple with trailing comma
    let input = "(1, 2, 3,)";
    let result: (i32, i32, i32) = from_str(input).unwrap();
    assert_eq!(result, (1, 2, 3));

    // Single element tuple (requires trailing comma)
    let input = "(42,)";
    let result: (i32,) = from_str(input).unwrap();
    assert_eq!(result, (42,));

    // Empty tuple (unit type)
    let input = "()";
    let result: () = from_str(input).unwrap();
    assert_eq!(result, ());
}

#[test]
fn test_string_with_whitespace_content() {
    // String containing spaces
    let input = r#""hello world""#;
    let result: String = from_str(input).unwrap();
    assert_eq!(result, "hello world");

    // String containing tabs and newlines
    let input = r#""hello\tworld\n""#;
    let result: String = from_str(input).unwrap();
    assert_eq!(result, "hello\tworld\n");

    // String with whitespace around it vs. in it
    let input = r#"  "hello world"  "#;
    let result: String = from_str(input).unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_compact_vs_pretty_formatting() {
    let expected = CollectionStruct {
        vec: vec![1, 2, 3],
        map: {
            let mut m = BTreeMap::new();
            m.insert("key1".to_string(), 1);
            m.insert("key2".to_string(), 2);
            m
        },
        set: {
            let mut s = BTreeSet::new();
            s.insert("item1".to_string());
            s.insert("item2".to_string());
            s
        },
    };

    // Compact format with simple values to avoid parsing issues
    let input = r#"CollectionStruct { vec: [1, 2, 3], map: {"key1": 1, "key2": 2}, set: {"item1", "item2"} }"#;
    let result: CollectionStruct = from_str(input).unwrap();
    assert_eq!(result, expected);

    // Pretty format with extensive whitespace
    let input = r#"CollectionStruct {
        vec: [
            1,
            2,
            3,
        ],
        map: {
            "key1": 1,
            "key2": 2,
        },
        set: {
            "item1",
            "item2",
        },
    }"#;
    let result: CollectionStruct = from_str(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_mixed_whitespace_types() {
    // Mix of spaces, tabs, and newlines
    let input = "SimpleStruct\t{\n\ta:\t42,\n    b: \"hello\"\n}";
    let result: SimpleStruct = from_str(input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");
}

#[test]
fn test_punctuation_whitespace() {
    // Test whitespace around various punctuation marks in different contexts

    // Struct with spaces around colons and commas
    let input = r#"SimpleStruct { a : 42 , b : "hello" }"#;
    let result: SimpleStruct = from_str(input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");

    // Enum tuple variant with spaces around parentheses
    let input = r#"Tuple ( 42 , "test" )"#;
    let result: TestEnum = from_str(input).unwrap();
    assert_eq!(result, TestEnum::Tuple(42, "test".to_string()));
}

// Error case tests - these should fail to parse
#[test]
fn test_missing_comma_errors() {
    // Missing comma between struct fields should fail
    let input = r#"SimpleStruct { a: 42 b: "hello" }"#;
    let result: Result<SimpleStruct, Error> = from_str(input);
    assert_matches!(result, Err(_));

    // Missing comma between vector elements should fail
    let input = "[1 2 3]";
    let result: Result<Vec<i32>, Error> = from_str(input);
    assert_matches!(result, Err(_));

    // Missing comma between map entries should fail
    let input = r#"{"a": 1 "b": 2}"#;
    let result: Result<BTreeMap<String, i32>, Error> = from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_invalid_whitespace_in_numbers() {
    // Space in the middle of a number should fail
    let input = "12 34";
    let result: Result<i32, Error> = from_str(input);
    assert_matches!(result, Err(_));

    // Tab in the middle of a float should fail
    let input = "12.34\t56";
    let result: Result<f64, Error> = from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_invalid_whitespace_in_identifiers() {
    // Space in struct name should fail
    let input = r#"Simple Struct { a: 42, b: "hello" }"#;
    let result: Result<SimpleStruct, Error> = from_str(input);
    assert_matches!(result, Err(_));

    // Space in field name should fail
    let input = r#"SimpleStruct { a b: 42, c: "hello" }"#;
    let result: Result<SimpleStruct, Error> = from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_whitespace_only_input() {
    // Only whitespace should fail
    let input = "   \t\n   ";
    let result: Result<SimpleStruct, Error> = from_str(input);
    assert_matches!(result, Err(_));

    // Empty string should fail
    let input = "";
    let result: Result<SimpleStruct, Error> = from_str(input);
    assert_matches!(result, Err(_));
}

#[test]
fn test_extreme_whitespace_patterns() {
    // Very large amounts of whitespace
    let input = format!(
        "{}SimpleStruct{}{{{}a{}:{}42{},{}b{}:{}\"hello\"{}}}{}",
        " ".repeat(100), // leading
        " ".repeat(50),  // after struct name
        " ".repeat(30),  // after opening brace
        " ".repeat(20),  // after field name
        " ".repeat(15),  // after colon
        " ".repeat(25),  // after value
        " ".repeat(10),  // after comma
        " ".repeat(20),  // after field name
        " ".repeat(15),  // after colon
        " ".repeat(35),  // after value
        " ".repeat(40),  // before closing brace
    );
    let result: SimpleStruct = from_str(&input).unwrap();
    assert_eq!(result.a, 42);
    assert_eq!(result.b, "hello");
}

#[test]
fn test_nested_collections_with_whitespace() {
    // Vector of vectors with various whitespace
    let input = r#"[
        [ 1 , 2 ],
        [3,4],
        [  5  ,  6  ,  ]
    ]"#;
    let result: Vec<Vec<i32>> = from_str(input).unwrap();
    assert_eq!(result, vec![vec![1, 2], vec![3, 4], vec![5, 6]]);

    // Map with vector values
    let input = r#"{
        "first": [ 1, 2, 3 ],
        "second": [4,5,6,]
    }"#;
    let result: BTreeMap<String, Vec<i32>> = from_str(input).unwrap();
    let mut expected = BTreeMap::new();
    expected.insert("first".to_string(), vec![1, 2, 3]);
    expected.insert("second".to_string(), vec![4, 5, 6]);
    assert_eq!(result, expected);
}

#[test]
fn test_struct_with_all_whitespace_variations() {
    // Test a struct that exercises all whitespace patterns in one test
    let input = r#"
        NestedStruct    {
            inner   :   SimpleStruct   {
                a   :   999   ,
                b   :   "complex test"   ,
            }   ,
            flag   :   false   ,
        }
    "#;
    let result: NestedStruct = from_str(input).unwrap();
    assert_eq!(result.inner.a, 999);
    assert_eq!(result.inner.b, "complex test");
    assert_eq!(result.flag, false);
}
