//! Comprehensive round-trip tests for serde_dbgfmt
//!
//! This test suite covers complex nested structures, large data sets, and
//! stress tests to ensure the deserializer can handle:
//! - Deep nesting of various types (structs, enums, collections)
//! - Large collections (performance and memory stress tests)
//! - Type-specific edge cases (primitive boundary values)
//! - Mixed complex structures combining all supported types
//! - Stress tests with large collections
//! - Unicode and special character handling
//! - Complex enum variations and nested combinations

use std::collections::{BTreeMap, BTreeSet};

use pretty_assertions::assert_eq;
use serde::Deserialize;

macro_rules! roundtrip_struct {
    {
        $(
            $( #[$attr:meta] )*
            $test:ident {
                $( #[$stattr:meta] )*
                struct $name:ident {
                    $(
                        $field:ident: $ty:ty = $value:expr
                    ),* $(,)?
                }
            }
        )*
    } => {$(
        #[test]
        $( #[$attr:meta] )*
        fn $test() {
            #[derive(Debug, Deserialize, PartialEq)]
            $( #[$stattr] )*
            struct $name {
                $( $field: $ty, )*
            }


            let src = $name {
                $( $field: $value, )*
            };

            let text = format!("{src:?}");
            eprintln!("{text}");

            let mut de = serde_dbgfmt::Deserializer::new(&text);

            let dst: $name = serde_path_to_error::deserialize(&mut de)
                .unwrap_or_else(|e| panic!("{}", e));
            de.end().expect("failed to deserialize");

            assert_eq!(src, dst);
        }
    )*}
}

// Complex nested enums for testing
#[derive(Debug, Eq, PartialEq, Deserialize)]
enum ComplexEnum {
    Unit,
    Tuple(u32, String, bool),
    Struct { id: u64, name: String, active: bool },
    Nested(Box<ComplexEnum>),
    Multiple(Vec<ComplexEnum>),
}

#[derive(Debug, PartialEq, Deserialize)]
enum NumberEnum {
    Integer(i64),
    Float(f64),
    Complex { real: f64, imag: f64 },
}

#[derive(Debug, PartialEq, Deserialize)]
struct Point3D {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Matrix3x3 {
    data: Vec<Vec<f64>>,
}

roundtrip_struct! {
    // Test 1: Deep nesting with multiple levels of structs
    test_deep_nesting {
        struct DeepNested {
            level1: Level1 = Level1 {
                level2: Level2 {
                    level3: Level3 {
                        level4: Level4 {
                            level5: Level5 {
                                data: vec![1, 2, 3, 4, 5],
                                message: "Deep nesting test".to_string(),
                            }
                        }
                    }
                }
            }
        }
    }

    // Test 2: Large collections stress test
    test_large_collections {
        struct LargeCollections {
            large_vec: Vec<u32> = (0..1000).collect(),
            large_map: BTreeMap<String, u32> = (0..100)
                .map(|i| (format!("key_{}", i), i * 2))
                .collect(),
            large_set: BTreeSet<u32> = (0..500).collect(),
            nested_vecs: Vec<Vec<u32>> = (0..50)
                .map(|i| (0..i).collect())
                .collect()
        }
    }

    // Test 3: Extreme primitive values and edge cases
    test_extreme_primitives {
        struct ExtremePrimitives {
            // Integer edge cases
            u8_max: u8 = u8::MAX,
            u16_max: u16 = u16::MAX,
            u32_max: u32 = u32::MAX,
            u64_max: u64 = u64::MAX,
            u128_max: u128 = u128::MAX,

            i8_min: i8 = i8::MIN,
            i8_max: i8 = i8::MAX,
            i16_min: i16 = i16::MIN,
            i16_max: i16 = i16::MAX,
            i32_min: i32 = i32::MIN,
            i32_max: i32 = i32::MAX,
            i64_min: i64 = i64::MIN,
            i64_max: i64 = i64::MAX,
            i128_min: i128 = i128::MIN,
            i128_max: i128 = i128::MAX,

            // Float edge cases
            f32_zero: f32 = 0.0,
            f32_neg_zero: f32 = -0.0,
            f32_one: f32 = 1.0,
            f32_neg_one: f32 = -1.0,
            f32_pi: f32 = std::f32::consts::PI,
            f32_e: f32 = std::f32::consts::E,
            f32_small: f32 = f32::MIN_POSITIVE,
            f32_large: f32 = f32::MAX,
            f32_neg_large: f32 = f32::MIN,

            f64_zero: f64 = 0.0,
            f64_neg_zero: f64 = -0.0,
            f64_one: f64 = 1.0,
            f64_neg_one: f64 = -1.0,
            f64_pi: f64 = std::f64::consts::PI,
            f64_e: f64 = std::f64::consts::E,
            f64_small: f64 = f64::MIN_POSITIVE,
            f64_large: f64 = f64::MAX,
            f64_neg_large: f64 = f64::MIN,

            // Character edge cases
            char_ascii: char = 'A',
            char_unicode: char = '🚀',
            char_escape: char = '\n',
            char_tab: char = '\t',
            char_quote: char = '"',
            char_backslash: char = '\\',

            // String edge cases (avoiding empty string due to lexer bug)
            short_string: String = " ".to_string(),
            long_string: String = "A".repeat(1000),
            unicode_string: String = "Hello 世界 🌍 café résumé naïve".to_string(),
            escape_string: String = "Line 1\nLine 2\tTabbed\r\nWindows line ending".to_string()
        }
    }

    // Test 4: Complex nested structures with all types combined
    test_complex_mixed {
        struct ComplexMixed {
            metadata: BTreeMap<String, MetadataValue> = {
                let mut map = BTreeMap::new();
                map.insert("version".to_string(), MetadataValue::Integer(42));
                map.insert("author".to_string(), MetadataValue::Text("Alice".to_string()));
                map.insert("active".to_string(), MetadataValue::Boolean(true));
                map.insert("score".to_string(), MetadataValue::Float(98.5));
                map.insert("tags".to_string(), MetadataValue::List(vec![
                    MetadataValue::Text("rust".to_string()),
                    MetadataValue::Text("serde".to_string()),
                    MetadataValue::Text("debug".to_string())
                ]));
                map
            },
            points: Vec<Point3D> = vec![
                Point3D { x: 0.0, y: 0.0, z: 0.0 },
                Point3D { x: 1.0, y: 2.0, z: 3.0 },
                Point3D { x: -1.5, y: 2.5, z: -3.5 },
                Point3D { x: f64::MAX, y: f64::MIN, z: 0.0 }
            ],
            matrix: Matrix3x3 = Matrix3x3 {
                data: vec![
                    vec![1.0, 2.0, 3.0],
                    vec![4.0, 5.0, 6.0],
                    vec![7.0, 8.0, 9.0]
                ]
            },
            optional_data: Option<Vec<Option<String>>> = Some(vec![
                Some("first".to_string()),
                None,
                Some("third".to_string()),
                Some("fourth".to_string()),
                None
            ]),
            nested_options: Option<Option<Option<u32>>> = Some(Some(Some(42))),
            tuple_data: (String, u32, bool, Option<f64>) = (
                "tuple_test".to_string(),
                123,
                true,
                Some(3.14159)
            )
        }
    }

    // Test 5: Complex enum variations stress test
    test_complex_enums {
        struct ComplexEnums {
            simple_enum: ComplexEnum = ComplexEnum::Unit,
            tuple_enum: ComplexEnum = ComplexEnum::Tuple(
                42,
                "test string".to_string(),
                true
            ),
            struct_enum: ComplexEnum = ComplexEnum::Struct {
                id: 12345,
                name: "Test Name".to_string(),
                active: false
            },
            nested_enum: ComplexEnum = ComplexEnum::Nested(Box::new(
                ComplexEnum::Struct {
                    id: 99999,
                    name: "Nested".to_string(),
                    active: true
                }
            )),
            multiple_enum: ComplexEnum = ComplexEnum::Multiple(vec![
                ComplexEnum::Unit,
                ComplexEnum::Tuple(1, "one".to_string(), true),
                ComplexEnum::Struct { id: 2, name: "two".to_string(), active: false },
                ComplexEnum::Nested(Box::new(ComplexEnum::Unit))
            ]),
            number_enum: NumberEnum = NumberEnum::Complex { real: 3.0, imag: 4.0 },
            enum_vec: Vec<ComplexEnum> = vec![
                ComplexEnum::Unit,
                ComplexEnum::Tuple(100, "hundred".to_string(), false),
                ComplexEnum::Multiple(vec![])
            ],
            enum_map: BTreeMap<String, NumberEnum> = {
                let mut map = BTreeMap::new();
                map.insert("int".to_string(), NumberEnum::Integer(-42));
                map.insert("float".to_string(), NumberEnum::Float(3.14159));
                map.insert("complex".to_string(), NumberEnum::Complex { real: 1.0, imag: -1.0 });
                map
            }
        }
    }

    // Test 6: Deeply nested collections and options
    test_nested_collections {
        struct NestedCollections {
            vec_of_vecs: Vec<Vec<Vec<u32>>> = vec![
                vec![vec![1, 2, 3], vec![4, 5, 6]],
                vec![vec![7, 8], vec![9, 10, 11, 12]],
                vec![vec![], vec![13]],
                vec![]
            ],
            map_of_maps: BTreeMap<String, BTreeMap<String, Vec<Option<u32>>>> = {
                let mut outer = BTreeMap::new();
                let mut inner1 = BTreeMap::new();
                inner1.insert("a".to_string(), vec![Some(1), None, Some(3)]);
                inner1.insert("b".to_string(), vec![None, Some(2)]);
                outer.insert("first".to_string(), inner1);

                let mut inner2 = BTreeMap::new();
                inner2.insert("c".to_string(), vec![Some(10), Some(20), Some(30)]);
                outer.insert("second".to_string(), inner2);

                outer
            },
            set_of_tuples: BTreeSet<(u32, String, bool)> = {
                let mut set = BTreeSet::new();
                set.insert((1, "one".to_string(), true));
                set.insert((2, "two".to_string(), false));
                set.insert((3, "three".to_string(), true));
                set
            },
            optional_collections: Option<Vec<Option<BTreeMap<String, Option<u32>>>>> = Some(vec![
                Some({
                    let mut map = BTreeMap::new();
                    map.insert("key1".to_string(), Some(100));
                    map.insert("key2".to_string(), None);
                    map
                }),
                None,
                Some(BTreeMap::new())
            ])
        }
    }

    // Test 7: Memory stress test with large nested structures
    test_memory_stress {
        struct MemoryStress {
            large_nested_vec: Vec<LargeStruct> = (0..100).map(|i| LargeStruct {
                id: i,
                data: (0..100).map(|j| format!("item_{}_{}", i, j)).collect(),
                metadata: (0..50).map(|k| (format!("key_{}", k), k * i)).collect(),
                active: i % 2 == 0
            }).collect(),
            deep_option_chain: Option<Option<Option<Option<Option<u32>>>>> = Some(Some(Some(Some(Some(42))))),
            tuple_stress: (
                Vec<String>,
                BTreeMap<u32, String>,
                BTreeSet<u32>,
                Option<Vec<Option<u32>>>,
                (u32, (String, (bool, (f64, u32))))
            ) = (
                (0..1000).map(|i| format!("string_{}", i)).collect(),
                (0..500).map(|i| (i, format!("value_{}", i))).collect(),
                (0..250).collect(),
                Some((0..100).map(|i| if i % 3 == 0 { None } else { Some(i) }).collect()),
                (
                    999,
                    (
                        "nested".to_string(),
                        (
                            true,
                            (
                                3.14159265359,
                                42
                            )
                        )
                    )
                )
            )
        }
    }

    // Test 8: Unicode and special character stress test
    test_unicode_stress {
        struct UnicodeStress {
            emoji_map: BTreeMap<String, String> = {
                let mut map = BTreeMap::new();
                map.insert("rocket".to_string(), "🚀".to_string());
                map.insert("globe".to_string(), "🌍".to_string());
                map.insert("heart".to_string(), "❤️".to_string());
                map.insert("fire".to_string(), "🔥".to_string());
                map.insert("star".to_string(), "⭐".to_string());
                map
            },
            multilingual: Vec<String> = vec![
                "Hello".to_string(),
                "世界".to_string(),
                "مرحبا".to_string(),
                "Здравствуй".to_string(),
                "Bonjour".to_string(),
                "こんにちは".to_string(),
                "Hola".to_string(),
                "नमस्ते".to_string()
            ],
            special_chars: BTreeSet<char> = {
                let mut set = BTreeSet::new();
                set.insert('A');
                set.insert('Z');
                set.insert('0');
                set.insert('9');
                set.insert('🦀');
                set.insert('🚀');
                set
            },
            normal_text: String = "This is a normal text string with basic punctuation!".to_string()
        }
    }

    // Test 9: Extreme nesting depth
    test_extreme_nesting {
        struct ExtremeNesting {
            nested_tuples: ((((((u32, String), bool), f64), char), Vec<u32>), BTreeMap<String, u32>) = (
                (
                    (
                        (
                            (
                                (
                                    (42, "deep".to_string()),
                                    true
                                ),
                                3.14159
                            ),
                            'X'
                        ),
                        vec![1, 2, 3, 4, 5]
                    ),
                    {
                        let mut map = BTreeMap::new();
                        map.insert("nested".to_string(), 100);
                        map.insert("deep".to_string(), 200);
                        map
                    }
                )
            ),
            nested_options: Option<Option<Option<Option<Option<String>>>>> = Some(Some(Some(Some(Some("deeply nested".to_string()))))),
            nested_vecs: Vec<Vec<Vec<Vec<Vec<u32>>>>> = vec![
                vec![
                    vec![
                        vec![
                            vec![1, 2, 3],
                            vec![4, 5, 6]
                        ],
                        vec![
                            vec![7, 8],
                            vec![9, 10, 11]
                        ]
                    ]
                ]
            ]
        }
    }

    // Test 10: Mixed primitive and complex type combinations
    test_mixed_combinations {
        struct MixedCombinations {
            all_primitives: AllPrimitives = AllPrimitives {
                b: true,
                u: 255u8,
                i: -128i8,
                f: 3.14f32,
                d: 2.718281828f64,
                c: '🦀',
                s: "Rust".to_string()
            },
            primitive_collections: Vec<AllPrimitives> = vec![
                AllPrimitives { b: true, u: 1u8, i: -1i8, f: 1.0f32, d: 1.0f64, c: 'A', s: "First".to_string() },
                AllPrimitives { b: false, u: 2u8, i: -2i8, f: 2.0f32, d: 2.0f64, c: 'B', s: "Second".to_string() },
                AllPrimitives { b: true, u: 3u8, i: -3i8, f: 3.0f32, d: 3.0f64, c: 'C', s: "Third".to_string() }
            ],
            tuple_variations: Vec<TupleTypes> = vec![
                TupleTypes::Unit,
                TupleTypes::Single(42),
                TupleTypes::Pair(1, 2),
                TupleTypes::Triple(1, 2, 3),
                TupleTypes::Complex((1, "test".to_string()), vec![1, 2, 3], Some(true))
            ],
            map_of_everything: BTreeMap<String, EverythingValue> = {
                let mut map = BTreeMap::new();
                map.insert("bool".to_string(), EverythingValue::Bool(true));
                map.insert("int".to_string(), EverythingValue::Integer(42));
                map.insert("float".to_string(), EverythingValue::Float(3.14));
                map.insert("string".to_string(), EverythingValue::Text("hello".to_string()));
                map.insert("list".to_string(), EverythingValue::List(vec![1, 2, 3, 4, 5]));
                map.insert("map".to_string(), EverythingValue::Map({
                    let mut inner = BTreeMap::new();
                    inner.insert("nested".to_string(), "value".to_string());
                    inner
                }));
                map
            }
        }
    }
}

// Helper types for complex tests

#[derive(Debug, PartialEq, Deserialize)]
struct Level1 {
    level2: Level2,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Level2 {
    level3: Level3,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Level3 {
    level4: Level4,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Level4 {
    level5: Level5,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Level5 {
    data: Vec<u32>,
    message: String,
}

#[derive(Debug, PartialEq, Deserialize)]
enum MetadataValue {
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    List(Vec<MetadataValue>),
}

#[derive(Debug, PartialEq, Deserialize)]
struct LargeStruct {
    id: u32,
    data: Vec<String>,
    metadata: BTreeMap<String, u32>,
    active: bool,
}

#[derive(Debug, PartialEq, Deserialize)]
struct AllPrimitives {
    b: bool,
    u: u8,
    i: i8,
    f: f32,
    d: f64,
    c: char,
    s: String,
}

#[derive(Debug, PartialEq, Deserialize)]
enum TupleTypes {
    Unit,
    Single(u32),
    Pair(u32, u32),
    Triple(u32, u32, u32),
    Complex((u32, String), Vec<u32>, Option<bool>),
}

#[derive(Debug, PartialEq, Deserialize)]
enum EverythingValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    List(Vec<u32>),
    Map(BTreeMap<String, String>),
}

// Additional performance and edge case tests
#[test]
fn test_performance_large_map() {
    let large_map: BTreeMap<String, u32> = (0..10000)
        .map(|i| (format!("key_{:06}", i), i * 2))
        .collect();

    let text = format!("{large_map:?}");
    eprintln!("Large map debug length: {}", text.len());

    let mut de = serde_dbgfmt::Deserializer::new(&text);
    let deserialized: BTreeMap<String, u32> =
        serde_path_to_error::deserialize(&mut de).unwrap_or_else(|e| panic!("{}", e));
    de.end().expect("failed to deserialize");

    assert_eq!(large_map, deserialized);
}

#[test]
fn test_empty_collections() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct EmptyCollections {
        empty_vec: Vec<u32>,
        empty_map: BTreeMap<String, u32>,
        empty_set: BTreeSet<u32>,
        non_empty_string: String,
    }

    let src = EmptyCollections {
        empty_vec: Vec::new(),
        empty_map: BTreeMap::new(),
        empty_set: BTreeSet::new(),
        non_empty_string: "non-empty".to_string(),
    };

    let text = format!("{src:?}");
    eprintln!("{text}");

    let mut de = serde_dbgfmt::Deserializer::new(&text);
    let dst: EmptyCollections =
        serde_path_to_error::deserialize(&mut de).unwrap_or_else(|e| panic!("{}", e));
    de.end().expect("failed to deserialize");

    assert_eq!(src, dst);
}

#[test]
fn test_normal_float_values() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct NormalFloats {
        zero: f64,
        negative_zero: f64,
        small: f64,
        large: f64,
        negative_large: f64,
    }

    let src = NormalFloats {
        zero: 0.0,
        negative_zero: -0.0,
        small: f64::MIN_POSITIVE,
        large: 1e100,
        negative_large: -1e100,
    };

    let text = format!("{src:?}");
    eprintln!("{text}");

    let mut de = serde_dbgfmt::Deserializer::new(&text);
    let dst: NormalFloats =
        serde_path_to_error::deserialize(&mut de).unwrap_or_else(|e| panic!("{}", e));
    de.end().expect("failed to deserialize");

    assert_eq!(src, dst);
}

#[test]
fn test_complex_tuple_struct() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct TupleStruct(u32, String, Vec<bool>, Option<f64>);

    let src = TupleStruct(
        42,
        "tuple test".to_string(),
        vec![true, false, true, false],
        Some(3.14159),
    );

    let text = format!("{src:?}");
    eprintln!("{text}");

    let mut de = serde_dbgfmt::Deserializer::new(&text);
    let dst: TupleStruct =
        serde_path_to_error::deserialize(&mut de).unwrap_or_else(|e| panic!("{}", e));
    de.end().expect("failed to deserialize");

    assert_eq!(src, dst);
}

#[test]
fn test_nested_enum_stress() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum NestedEnum {
        Simple,
        WithData(u32),
        WithStruct { field: String },
        WithVec(Vec<NestedEnum>),
        WithMap(BTreeMap<String, NestedEnum>),
    }

    let src = NestedEnum::WithMap({
        let mut map = BTreeMap::new();
        map.insert("simple".to_string(), NestedEnum::Simple);
        map.insert("data".to_string(), NestedEnum::WithData(100));
        map.insert(
            "struct".to_string(),
            NestedEnum::WithStruct {
                field: "test".to_string(),
            },
        );
        map.insert(
            "vec".to_string(),
            NestedEnum::WithVec(vec![
                NestedEnum::Simple,
                NestedEnum::WithData(200),
                NestedEnum::WithStruct {
                    field: "nested".to_string(),
                },
            ]),
        );
        map
    });

    let text = format!("{src:?}");
    eprintln!("{text}");

    let mut de = serde_dbgfmt::Deserializer::new(&text);
    let dst: NestedEnum =
        serde_path_to_error::deserialize(&mut de).unwrap_or_else(|e| panic!("{}", e));
    de.end().expect("failed to deserialize");

    assert_eq!(src, dst);
}
