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

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
enum ComplexEnum {
    Unit,
    Tuple(u32, String),
    Struct { id: u32, name: String },
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
struct NestedStruct {
    id: u32,
    data: Option<String>,
}

roundtrip_struct! {
    // Empty collections
    test_empty_vec {
        struct EmptyVec {
            empty_vec: Vec<u32> = vec![]
        }
    }

    test_empty_btreemap {
        struct EmptyMap {
            empty_map: BTreeMap<String, u32> = BTreeMap::new()
        }
    }

    test_empty_btreeset {
        struct EmptySet {
            empty_set: BTreeSet<String> = BTreeSet::new()
        }
    }

    // Single-element collections
    test_single_element_vec {
        struct SingleVec {
            single_vec: Vec<String> = vec!["single".to_string()]
        }
    }

    test_single_element_btreemap {
        struct SingleMap {
            single_map: BTreeMap<String, u32> = {
                let mut map = BTreeMap::new();
                map.insert("key".to_string(), 42);
                map
            }
        }
    }

    test_single_element_btreeset {
        struct SingleSet {
            single_set: BTreeSet<u32> = {
                let mut set = BTreeSet::new();
                set.insert(100);
                set
            }
        }
    }

    // Large collections (performance test)
    test_large_vec {
        struct LargeVec {
            large_vec: Vec<u32> = (0..100).collect()
        }
    }

    test_large_btreemap {
        struct LargeMap {
            large_map: BTreeMap<String, u32> = {
                (0..50).map(|i| (format!("key_{}", i), i)).collect()
            }
        }
    }

    test_large_btreeset {
        struct LargeSet {
            large_set: BTreeSet<u32> = (0..100).collect()
        }
    }

    // Nested collections
    test_nested_vec_vec {
        struct NestedVecVec {
            nested: Vec<Vec<u32>> = vec![
                vec![1, 2, 3],
                vec![],
                vec![4, 5],
                vec![6]
            ]
        }
    }

    test_nested_vec_map {
        struct NestedVecMap {
            nested: Vec<BTreeMap<String, u32>> = vec![
                {
                    let mut map = BTreeMap::new();
                    map.insert("a".to_string(), 1);
                    map.insert("b".to_string(), 2);
                    map
                },
                BTreeMap::new(),
                {
                    let mut map = BTreeMap::new();
                    map.insert("c".to_string(), 3);
                    map
                }
            ]
        }
    }

    test_map_with_vec_values {
        struct MapWithVecValues {
            map_vec: BTreeMap<String, Vec<u32>> = {
                let mut map = BTreeMap::new();
                map.insert("first".to_string(), vec![1, 2, 3]);
                map.insert("second".to_string(), vec![]);
                map.insert("third".to_string(), vec![4]);
                map
            }
        }
    }

    test_map_with_set_values {
        struct MapWithSetValues {
            map_set: BTreeMap<String, BTreeSet<u32>> = {
                let mut map = BTreeMap::new();
                map.insert("odds".to_string(), [1, 3, 5, 7].into_iter().collect());
                map.insert("evens".to_string(), [2, 4, 6, 8].into_iter().collect());
                map.insert("empty".to_string(), BTreeSet::new());
                map
            }
        }
    }

    // Complex key/value types
    test_map_with_tuple_keys {
        struct MapWithTupleKeys {
            tuple_key_map: BTreeMap<(u32, String), String> = {
                let mut map = BTreeMap::new();
                map.insert((1, "first".to_string()), "value1".to_string());
                map.insert((2, "second".to_string()), "value2".to_string());
                map
            }
        }
    }

    test_map_with_tuple_values {
        struct MapWithTupleValues {
            tuple_value_map: BTreeMap<String, (u32, bool, String)> = {
                let mut map = BTreeMap::new();
                map.insert("key1".to_string(), (42, true, "test".to_string()));
                map.insert("key2".to_string(), (0, false, "value".to_string()));
                map
            }
        }
    }

    // Collections with enums
    test_vec_with_enums {
        struct VecWithEnums {
            enum_vec: Vec<ComplexEnum> = vec![
                ComplexEnum::Unit,
                ComplexEnum::Tuple(42, "test".to_string()),
                ComplexEnum::Struct { id: 1, name: "example".to_string() }
            ]
        }
    }

    test_map_with_enum_keys {
        struct MapWithEnumKeys {
            enum_key_map: BTreeMap<ComplexEnum, String> = {
                let mut map = BTreeMap::new();
                map.insert(ComplexEnum::Unit, "unit".to_string());
                map.insert(ComplexEnum::Tuple(1, "key".to_string()), "tuple".to_string());
                map
            }
        }
    }

    test_map_with_enum_values {
        struct MapWithEnumValues {
            enum_value_map: BTreeMap<String, ComplexEnum> = {
                let mut map = BTreeMap::new();
                map.insert("first".to_string(), ComplexEnum::Unit);
                map.insert("second".to_string(), ComplexEnum::Tuple(99, "data".to_string()));
                map.insert("third".to_string(), ComplexEnum::Struct { id: 42, name: "test".to_string() });
                map
            }
        }
    }

    // Collections with structs
    test_vec_with_structs {
        struct VecWithStructs {
            struct_vec: Vec<NestedStruct> = vec![
                NestedStruct { id: 1, data: Some("first".to_string()) },
                NestedStruct { id: 2, data: None },
                NestedStruct { id: 3, data: Some("third".to_string()) }
            ]
        }
    }

    test_map_with_struct_values {
        struct MapWithStructValues {
            struct_value_map: BTreeMap<String, NestedStruct> = {
                let mut map = BTreeMap::new();
                map.insert("a".to_string(), NestedStruct { id: 1, data: Some("test".to_string()) });
                map.insert("b".to_string(), NestedStruct { id: 2, data: None });
                map
            }
        }
    }

    // Collections with Options
    test_vec_with_options {
        struct VecWithOptions {
            option_vec: Vec<Option<u32>> = vec![
                Some(1),
                None,
                Some(42),
                None,
                Some(0)
            ]
        }
    }

    test_map_with_option_values {
        struct MapWithOptionValues {
            option_value_map: BTreeMap<String, Option<String>> = {
                let mut map = BTreeMap::new();
                map.insert("some".to_string(), Some("value".to_string()));
                map.insert("none".to_string(), None);
                map.insert("another".to_string(), Some("test".to_string()));
                map
            }
        }
    }

    test_option_of_collections {
        struct OptionOfCollections {
            opt_vec: Option<Vec<u32>> = Some(vec![1, 2, 3]),
            opt_empty_vec: Option<Vec<u32>> = Some(vec![]),
            opt_none_vec: Option<Vec<u32>> = None,
            opt_map: Option<BTreeMap<String, u32>> = {
                let mut map = BTreeMap::new();
                map.insert("key".to_string(), 42);
                Some(map)
            },
            opt_none_map: Option<BTreeMap<String, u32>> = None
        }
    }

    // Collections with tuples
    test_vec_with_tuples {
        struct VecWithTuples {
            tuple_vec: Vec<(u32, String, bool)> = vec![
                (1, "first".to_string(), true),
                (2, "second".to_string(), false),
                (0, "third".to_string(), true)
            ]
        }
    }

    test_set_with_tuples {
        struct SetWithTuples {
            tuple_set: BTreeSet<(u32, String)> = {
                let mut set = BTreeSet::new();
                set.insert((1, "a".to_string()));
                set.insert((2, "b".to_string()));
                set.insert((3, "c".to_string()));
                set
            }
        }
    }

    // Mixed heterogeneous structures
    test_complex_mixed_structure {
        struct ComplexMixed {
            mixed_data: BTreeMap<String, Vec<Option<(u32, BTreeSet<String>)>>> = {
                let mut map = BTreeMap::new();

                let mut set1 = BTreeSet::new();
                set1.insert("tag1".to_string());
                set1.insert("tag2".to_string());

                let mut set2 = BTreeSet::new();
                set2.insert("tag3".to_string());

                map.insert("group1".to_string(), vec![
                    Some((1, set1)),
                    None,
                    Some((2, BTreeSet::new()))
                ]);
                map.insert("group2".to_string(), vec![
                    Some((3, set2))
                ]);
                map.insert("empty_group".to_string(), vec![]);

                map
            }
        }
    }

    // Edge cases with safe string values
    test_collections_with_safe_strings {
        struct SafeStrings {
            safe_vec: Vec<String> = vec![
                "space".to_string(),
                "double_space".to_string(),
                "null".to_string(),
                "true".to_string(),
                "false".to_string(),
                "zero".to_string(),
                "empty_braces".to_string(),
                "empty_brackets".to_string(),
                "empty_parens".to_string()
            ]
        }
    }

    test_map_with_safe_string_keys {
        struct SafeStringKeys {
            safe_key_map: BTreeMap<String, u32> = {
                let mut map = BTreeMap::new();
                map.insert("key_a".to_string(), 0);
                map.insert("key_b".to_string(), 1);
                map.insert("key with spaces".to_string(), 4);
                map.insert("regular_key".to_string(), 5);
                map
            }
        }
    }

    // Extreme nesting
    test_deeply_nested_collections {
        struct DeeplyNested {
            deep: Vec<Vec<Vec<u32>>> = vec![
                vec![
                    vec![1, 2],
                    vec![3, 4, 5]
                ],
                vec![
                    vec![],
                    vec![6],
                    vec![7, 8, 9, 10]
                ],
                vec![]
            ]
        }
    }

    // Collections with numeric edge cases
    test_collections_with_numeric_extremes {
        struct NumericExtremes {
            extreme_vec: Vec<i64> = vec![
                i64::MIN,
                -1,
                0,
                1,
                i64::MAX
            ],
            extreme_map: BTreeMap<i32, u64> = {
                let mut map = BTreeMap::new();
                map.insert(i32::MIN, 0);
                map.insert(-1, u64::MAX);
                map.insert(0, 42);
                map.insert(1, 1);
                map.insert(i32::MAX, u64::MAX);
                map
            }
        }
    }

    // Collections with boolean values
    test_collections_with_booleans {
        struct BooleanCollections {
            bool_vec: Vec<bool> = vec![true, false, true, true, false],
            bool_map: BTreeMap<String, bool> = {
                let mut map = BTreeMap::new();
                map.insert("yes".to_string(), true);
                map.insert("no".to_string(), false);
                map.insert("maybe".to_string(), true);
                map
            },
            bool_set: BTreeSet<bool> = {
                let mut set = BTreeSet::new();
                set.insert(true);
                set.insert(false);
                set
            }
        }
    }
}

// Additional tests for specific edge cases that don't fit the macro pattern

#[test]
fn test_roundtrip_multiple_empty_collections() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct MultipleEmpty {
        vec1: Vec<u32>,
        vec2: Vec<String>,
        map1: BTreeMap<String, u32>,
        map2: BTreeMap<u32, String>,
        set1: BTreeSet<u32>,
        set2: BTreeSet<String>,
    }

    let src = MultipleEmpty {
        vec1: vec![],
        vec2: vec![],
        map1: BTreeMap::new(),
        map2: BTreeMap::new(),
        set1: BTreeSet::new(),
        set2: BTreeSet::new(),
    };

    let text = format!("{src:?}");
    eprintln!("{text}");

    let mut de = serde_dbgfmt::Deserializer::new(&text);
    let dst: MultipleEmpty =
        serde_path_to_error::deserialize(&mut de).unwrap_or_else(|e| panic!("{}", e));
    de.end().expect("failed to deserialize");

    assert_eq!(src, dst);
}

#[test]
fn test_roundtrip_collections_in_tuples() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct CollectionsInTuples {
        tuple_with_collections: (Vec<u32>, BTreeMap<String, u32>, BTreeSet<String>),
    }

    let src = CollectionsInTuples {
        tuple_with_collections: (
            vec![1, 2, 3],
            {
                let mut map = BTreeMap::new();
                map.insert("key".to_string(), 42);
                map
            },
            {
                let mut set = BTreeSet::new();
                set.insert("item".to_string());
                set
            },
        ),
    };

    let text = format!("{src:?}");
    eprintln!("{text}");

    let mut de = serde_dbgfmt::Deserializer::new(&text);
    let dst: CollectionsInTuples =
        serde_path_to_error::deserialize(&mut de).unwrap_or_else(|e| panic!("{}", e));
    de.end().expect("failed to deserialize");

    assert_eq!(src, dst);
}
