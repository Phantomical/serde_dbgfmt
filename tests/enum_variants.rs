use std::collections::{BTreeMap, BTreeSet};

use pretty_assertions::assert_eq;
use serde::Deserialize;

// Define a macro similar to roundtrip_struct but for enums
macro_rules! roundtrip_enum {
    {
        $(
            $( #[$attr:meta] )*
            $test:ident {
                $( #[$enum_attr:meta] )*
                enum $name:ident {
                    $(
                        $( #[$variant_attr:meta] )*
                        $variant:ident $({
                            $(
                                $( #[$field_attr:meta] )*
                                $field:ident: $ty:ty
                            ),* $(,)?
                        })? $(( $($tuple_ty:ty),* $(,)? ))?
                    ),* $(,)?
                }
                values: [$(
                    $value:expr
                ),* $(,)?]
            }
        )*
    } => {$(
        #[test]
        $( #[$attr] )*
        fn $test() {
            #[derive(Debug, Deserialize, PartialEq)]
            $( #[$enum_attr] )*
            enum $name {
                $(
                    $( #[$variant_attr] )*
                    $variant $({
                        $(
                            $( #[$field_attr] )*
                            $field: $ty,
                        )*
                    })? $(( $($tuple_ty),* ))?
                ),*
            }

            let test_values = vec![$($value),*];

            for src in test_values {
                let text = format!("{src:?}");
                eprintln!("{text}");

                let mut de = serde_dbgfmt::Deserializer::new(&text);
                let dst: $name = serde_path_to_error::deserialize(&mut de)
                    .unwrap_or_else(|e| panic!("Failed to deserialize: {}", e));
                de.end().expect("failed to deserialize");

                assert_eq!(src, dst);
            }
        }
    )*}
}

// Simple enum with all variant types
roundtrip_enum! {
    test_basic_enum_variants {
        enum BasicEnum {
            Unit,
            Tuple(String, u32),
            Struct { name: String, value: i32 }
        }
        values: [
            BasicEnum::Unit,
            BasicEnum::Tuple("hello".to_string(), 42),
            BasicEnum::Struct { name: "test".to_string(), value: -1 }
        ]
    }
}

// Enum with complex payloads including collections
roundtrip_enum! {
    test_complex_enum_payloads {
        enum ComplexEnum {
            Empty,
            WithVec(Vec<String>),
            WithMap(BTreeMap<String, u32>),
            WithSet(BTreeSet<i32>),
            WithOption(Option<String>),
            WithTuple((String, Vec<u32>, BTreeMap<String, bool>)),
            WithStruct {
                vec_field: Vec<f64>,
                map_field: BTreeMap<String, Option<i32>>,
                nested_option: Option<Vec<String>>
            }
        }
        values: [
            ComplexEnum::Empty,
            ComplexEnum::WithVec(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
            ComplexEnum::WithMap({
                let mut map = BTreeMap::new();
                map.insert("key1".to_string(), 10);
                map.insert("key2".to_string(), 20);
                map
            }),
            ComplexEnum::WithSet({
                let mut set = BTreeSet::new();
                set.insert(1);
                set.insert(2);
                set.insert(3);
                set
            }),
            ComplexEnum::WithOption(Some("test".to_string())),
            ComplexEnum::WithOption(None),
            ComplexEnum::WithTuple((
                "nested".to_string(),
                vec![1, 2, 3],
                {
                    let mut map = BTreeMap::new();
                    map.insert("true_key".to_string(), true);
                    map.insert("false_key".to_string(), false);
                    map
                }
            )),
            ComplexEnum::WithStruct {
                vec_field: vec![1.1, 2.2, 3.3],
                map_field: {
                    let mut map = BTreeMap::new();
                    map.insert("some".to_string(), Some(42));
                    map.insert("none".to_string(), None);
                    map
                },
                nested_option: Some(vec!["nested".to_string(), "vector".to_string()])
            }
        ]
    }
}

// Nested enum structures
roundtrip_enum! {
    test_nested_enums {
        enum InnerEnum {
            A,
            B(u32),
            C { value: String }
        }
        values: [
            InnerEnum::A,
            InnerEnum::B(999),
            InnerEnum::C { value: "inner".to_string() }
        ]
    }
}

#[derive(Debug, Deserialize, PartialEq)]
enum InnerEnum {
    A,
    B(u32),
    C { value: String }
}

roundtrip_enum! {
    test_enum_containing_enums {
        enum OuterEnum {
            Simple,
            WithInner(InnerEnum),
            WithMultipleInner(InnerEnum, InnerEnum),
            WithInnerStruct {
                first: InnerEnum,
                second: Option<InnerEnum>,
                third: Vec<InnerEnum>
            }
        }
        values: [
            OuterEnum::Simple,
            OuterEnum::WithInner(InnerEnum::A),
            OuterEnum::WithInner(InnerEnum::B(123)),
            OuterEnum::WithInner(InnerEnum::C { value: "nested_enum".to_string() }),
            OuterEnum::WithMultipleInner(InnerEnum::A, InnerEnum::B(456)),
            OuterEnum::WithInnerStruct {
                first: InnerEnum::C { value: "first".to_string() },
                second: Some(InnerEnum::B(789)),
                third: vec![InnerEnum::A, InnerEnum::B(999), InnerEnum::C { value: "in_vec".to_string() }]
            }
        ]
    }
}

// Enum with different naming patterns
roundtrip_enum! {
    test_enum_different_naming {
        #[allow(non_camel_case_types, non_snake_case)]
        enum NamingEnum {
            CamelCase,
            snake_case_variant,
            SCREAMING_CASE,
            PascalCase { CamelField: String },
            snake_variant { snake_field: u32 }
        }
        values: [
            NamingEnum::CamelCase,
            NamingEnum::snake_case_variant,
            NamingEnum::SCREAMING_CASE,
            NamingEnum::PascalCase { CamelField: "test".to_string() },
            NamingEnum::snake_variant { snake_field: 42 }
        ]
    }
}

// Enum with many variants to test parser scalability
roundtrip_enum! {
    test_enum_many_variants {
        enum ManyVariants {
            V1, V2, V3, V4, V5, V6, V7, V8, V9, V10,
            V11(String), V12(u32), V13(bool), V14(f64), V15(char),
            V16 { a: i32 }, V17 { b: String }, V18 { c: bool }, V19 { d: f32 }, V20 { e: u64 }
        }
        values: [
            ManyVariants::V1, ManyVariants::V5, ManyVariants::V10,
            ManyVariants::V11("test".to_string()),
            ManyVariants::V12(12345),
            ManyVariants::V13(true),
            ManyVariants::V14(3.14159),
            ManyVariants::V15('Z'),
            ManyVariants::V16 { a: -1 },
            ManyVariants::V17 { b: "variant17".to_string() },
            ManyVariants::V18 { c: false },
            ManyVariants::V19 { d: 2.718 },
            ManyVariants::V20 { e: u64::MAX }
        ]
    }
}

// Option-like and Result-like enums (concrete types since we don't support generics in the macro)
roundtrip_enum! {
    test_option_like_enum {
        enum StringOptionLike {
            Some(String),
            None
        }
        values: [
            StringOptionLike::Some("test".to_string()),
            StringOptionLike::None
        ]
    }
}

roundtrip_enum! {
    test_result_like_enum {
        enum StringResultLike {
            Ok(String),
            Err(String)
        }
        values: [
            StringResultLike::Ok("success".to_string()),
            StringResultLike::Err("error".to_string())
        ]
    }
}

// Enum with tuple variants of different arities
roundtrip_enum! {
    test_tuple_variants_different_arities {
        enum TupleVariants {
            Nullary,
            Unary(String),
            Binary(u32, String),
            Ternary(bool, f64, char),
            Quaternary(i8, i16, i32, i64),
            Quinary(String, u32, bool, f64, char)
        }
        values: [
            TupleVariants::Nullary,
            TupleVariants::Unary("single".to_string()),
            TupleVariants::Binary(42, "two".to_string()),
            TupleVariants::Ternary(true, 3.14, 'X'),
            TupleVariants::Quaternary(1, 2, 3, 4),
            TupleVariants::Quinary("five".to_string(), 5, false, 5.5, '5')
        ]
    }
}

// Enum containing complex data structures
roundtrip_enum! {
    test_enum_complex_data_structures {
        enum ComplexDataEnum {
            Unit,
            NestedTuple((Vec<String>, BTreeMap<u32, String>)),
            NestedStruct {
                tuple_field: (String, Vec<u32>),
                map_field: BTreeMap<String, Vec<bool>>,
                optional_complex: Option<(BTreeMap<String, u32>, Vec<String>)>
            },
            DeepNesting(Vec<BTreeMap<String, Vec<Option<String>>>>)
        }
        values: [
            ComplexDataEnum::Unit,
            ComplexDataEnum::NestedTuple((
                vec!["a".to_string(), "b".to_string()],
                {
                    let mut map = BTreeMap::new();
                    map.insert(1, "one".to_string());
                    map.insert(2, "two".to_string());
                    map
                }
            )),
            ComplexDataEnum::NestedStruct {
                tuple_field: ("tuple".to_string(), vec![1, 2, 3]),
                map_field: {
                    let mut map = BTreeMap::new();
                    map.insert("bools".to_string(), vec![true, false, true]);
                    map
                },
                optional_complex: Some(({
                    let mut map = BTreeMap::new();
                    map.insert("key".to_string(), 42);
                    map
                }, vec!["opt".to_string()]))
            },
            ComplexDataEnum::DeepNesting(vec![{
                let mut outer_map = BTreeMap::new();
                outer_map.insert("deep".to_string(), vec![
                    Some("very".to_string()),
                    None,
                    Some("nested".to_string())
                ]);
                outer_map
            }])
        ]
    }
}

// Test enum with numeric types
roundtrip_enum! {
    test_enum_numeric_types {
        enum NumericEnum {
            UnsignedTypes(u8, u16, u32, u64, u128),
            SignedTypes(i8, i16, i32, i64, i128),
            FloatTypes(f32, f64),
            NumericStruct {
                small: u8,
                big: u128,
                negative: i64,
                pi: f64
            }
        }
        values: [
            NumericEnum::UnsignedTypes(u8::MAX, u16::MAX, u32::MAX, u64::MAX, u128::MAX),
            NumericEnum::SignedTypes(i8::MIN, i16::MIN, i32::MIN, i64::MIN, i128::MIN),
            NumericEnum::SignedTypes(i8::MAX, i16::MAX, i32::MAX, i64::MAX, i128::MAX),
            NumericEnum::FloatTypes(3.14f32, 2.718281828f64),
            NumericEnum::FloatTypes(-1.0f32, -1.0f64),
            NumericEnum::NumericStruct {
                small: 42,
                big: u128::MAX,
                negative: -1,
                pi: 3.141592653589793
            }
        ]
    }
}

// Test edge cases with boolean and character types
roundtrip_enum! {
    test_enum_bool_char_types {
        enum BoolCharEnum {
            BoolVariant(bool),
            CharVariant(char),
            BoolStruct { flag: bool },
            CharStruct { character: char },
            Mixed(bool, char, String)
        }
        values: [
            BoolCharEnum::BoolVariant(true),
            BoolCharEnum::BoolVariant(false),
            BoolCharEnum::CharVariant('A'),
            BoolCharEnum::CharVariant('🦀'),
            BoolCharEnum::CharVariant('\n'),
            BoolCharEnum::CharVariant('\0'),
            BoolCharEnum::BoolStruct { flag: true },
            BoolCharEnum::CharStruct { character: '💯' },
            BoolCharEnum::Mixed(false, 'Z', "mixed".to_string())
        ]
    }
}

// Test enum with empty collections
roundtrip_enum! {
    test_enum_empty_collections {
        enum EmptyCollectionEnum {
            EmptyVec(Vec<String>),
            EmptyMap(BTreeMap<String, u32>),
            EmptySet(BTreeSet<i32>),
            EmptyStruct {
                empty_vec: Vec<bool>,
                empty_map: BTreeMap<u32, String>
            }
        }
        values: [
            EmptyCollectionEnum::EmptyVec(vec![]),
            EmptyCollectionEnum::EmptyMap(BTreeMap::new()),
            EmptyCollectionEnum::EmptySet(BTreeSet::new()),
            EmptyCollectionEnum::EmptyStruct {
                empty_vec: vec![],
                empty_map: BTreeMap::new()
            }
        ]
    }
}