//! Comprehensive test suite for struct variations in serde_dbgfmt
//!
//! This test file covers various structural patterns that can be deserialized
//! from Rust's Debug format output, including:
//!
//! - Nested structs (structs containing other structs)
//! - Tuple structs (various arities: single, pair, triple, complex types)
//! - Unit structs (with and without attributes like #[non_exhaustive])
//! - Mixed field types (combining primitives, collections, enums, structs)
//! - Deeply nested struct hierarchies
//! - Structs with optional fields (Option<T>)
//! - Newtype structs (single-field tuple structs)
//! - Complex non-exhaustive struct cases
//! - Edge cases with empty collections and numeric extremes
//!
//! Note: Tests for structs with renamed fields (#[serde(rename = "...")]) are
//! not included due to a fundamental limitation: the Debug format uses the
//! original field names while serde expects the renamed ones.

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
        $( #[$attr] )*
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

macro_rules! roundtrip_tuple_struct {
    {
        $(
            $( #[$attr:meta] )*
            $test:ident {
                $( #[$stattr:meta] )*
                struct $name:ident ( $( $ty:ty ),* $(,)? ) = $value:expr
            }
        )*
    } => {$(
        #[test]
        $( #[$attr] )*
        fn $test() {
            #[derive(Debug, Deserialize, PartialEq)]
            $( #[$stattr] )*
            struct $name ( $( $ty, )* );

            let src = $value;
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

macro_rules! roundtrip_unit_struct {
    {
        $(
            $( #[$attr:meta] )*
            $test:ident {
                $( #[$stattr:meta] )*
                struct $name:ident;
            }
        )*
    } => {$(
        #[test]
        $( #[$attr] )*
        fn $test() {
            #[derive(Debug, Deserialize, PartialEq)]
            $( #[$stattr] )*
            struct $name;

            let src = $name;
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

// Helper types for nested struct tests
#[derive(Debug, Deserialize, PartialEq)]
struct Address {
    street: String,
    city: String,
    zip: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Person {
    name: String,
    age: u32,
    address: Address,
}

#[derive(Debug, Deserialize, PartialEq)]
enum Status {
    Active,
    Inactive { reason: String },
    Pending(u32),
}

#[derive(Debug, Deserialize, PartialEq)]
struct Company {
    name: String,
    employees: Vec<Person>,
    status: Status,
}

// Test nested structs
roundtrip_struct! {
    test_simple_nested_struct {
        struct Test {
            person: Person = Person {
                name: "John Doe".into(),
                age: 30,
                address: Address {
                    street: "123 Main St".into(),
                    city: "Anytown".into(),
                    zip: 12345,
                }
            }
        }
    }

    test_multiple_nested_structs {
        struct Test {
            primary: Address = Address {
                street: "123 Main St".into(),
                city: "Primary City".into(),
                zip: 11111,
            },
            secondary: Address = Address {
                street: "456 Oak Ave".into(),
                city: "Secondary City".into(),
                zip: 22222,
            }
        }
    }

    test_deeply_nested_struct {
        struct Test {
            company: Company = Company {
                name: "Tech Corp".into(),
                employees: vec![
                    Person {
                        name: "Alice".into(),
                        age: 25,
                        address: Address {
                            street: "789 Pine St".into(),
                            city: "Tech Town".into(),
                            zip: 33333,
                        }
                    },
                    Person {
                        name: "Bob".into(),
                        age: 35,
                        address: Address {
                            street: "321 Elm St".into(),
                            city: "Code City".into(),
                            zip: 44444,
                        }
                    }
                ],
                status: Status::Active,
            }
        }
    }

    test_nested_with_complex_status {
        struct Test {
            company: Company = Company {
                name: "StartupCo".into(),
                employees: vec![],
                status: Status::Inactive { reason: "Funding issues".into() },
            }
        }
    }
}

// Test tuple structs with various arities
roundtrip_tuple_struct! {
    test_tuple_struct_single {
        struct SingleTuple(u32) = SingleTuple(42)
    }

    test_tuple_struct_pair {
        struct Pair(String, u32) = Pair("hello".into(), 123)
    }

    test_tuple_struct_triple {
        struct Triple(bool, f64, String) = Triple(true, 3.14159, "pi".into())
    }

    test_tuple_struct_complex {
        struct Complex(Vec<u32>, BTreeMap<String, String>, Option<bool>) = Complex(
            vec![1, 2, 3],
            {
                let mut map = BTreeMap::new();
                map.insert("key1".into(), "value1".into());
                map.insert("key2".into(), "value2".into());
                map
            },
            Some(false)
        )
    }

    test_tuple_struct_nested {
        struct NestedTuple(Address, Person) = NestedTuple(
            Address {
                street: "999 Tuple St".into(),
                city: "Tuple Town".into(),
                zip: 99999,
            },
            Person {
                name: "Tuple Person".into(),
                age: 42,
                address: Address {
                    street: "888 Nested Ave".into(),
                    city: "Nested City".into(),
                    zip: 88888,
                }
            }
        )
    }
}

// Test unit structs
roundtrip_unit_struct! {
    test_unit_struct {
        struct UnitStruct;
    }

    test_unit_struct_with_attrs {
        #[non_exhaustive]
        struct NonExhaustiveUnit;
    }
}

// NOTE: Renamed fields tests are not included because the Debug format
// still uses the original field names, while serde expects the renamed ones.
// This is a fundamental limitation of the debug-format-based approach.

// Test mixed field types combining primitives, collections, enums, structs
roundtrip_struct! {
    test_mixed_field_types {
        struct Test {
            id: u64 = 12345,
            name: String = "Mixed Test".into(),
            active: bool = true,
            score: f64 = 98.5,
            tags: Vec<String> = vec!["rust".into(), "serde".into(), "debug".into()],
            metadata: BTreeMap<String, String> = {
                let mut map = BTreeMap::new();
                map.insert("version".into(), "1.0".into());
                map.insert("author".into(), "test".into());
                map
            },
            permissions: BTreeSet<String> = {
                let mut set = BTreeSet::new();
                set.insert("read".into());
                set.insert("write".into());
                set
            },
            address: Option<Address> = Some(Address {
                street: "123 Mixed St".into(),
                city: "Mixed City".into(),
                zip: 54321,
            }),
            status: Status = Status::Pending(100)
        }
    }

    test_all_optional_fields {
        struct Test {
            maybe_string: Option<String> = None,
            maybe_number: Option<u32> = Some(42),
            maybe_address: Option<Address> = None,
            maybe_vec: Option<Vec<String>> = Some(vec!["optional".into()]),
            maybe_map: Option<BTreeMap<String, u32>> = None
        }
    }

    test_complex_collections {
        struct Test {
            nested_vec: Vec<Vec<u32>> = vec![vec![1, 2], vec![3, 4, 5], vec![]],
            vec_of_structs: Vec<Address> = vec![
                Address {
                    street: "First St".into(),
                    city: "First City".into(),
                    zip: 10001,
                },
                Address {
                    street: "Second St".into(),
                    city: "Second City".into(),
                    zip: 20002,
                }
            ],
            map_of_vecs: BTreeMap<String, Vec<u32>> = {
                let mut map = BTreeMap::new();
                map.insert("evens".into(), vec![2, 4, 6]);
                map.insert("odds".into(), vec![1, 3, 5]);
                map
            },
            optional_nested: Option<Vec<Option<String>>> = Some(vec![
                Some("first".into()),
                None,
                Some("third".into())
            ])
        }
    }
}

// Test newtype structs
roundtrip_tuple_struct! {
    test_newtype_struct_string {
        struct UserId(String) = UserId("user_123".into())
    }

    test_newtype_struct_number {
        struct Temperature(f64) = Temperature(98.6)
    }

    test_newtype_struct_complex {
        struct Wrapper(BTreeMap<String, Vec<u32>>) = Wrapper({
            let mut map = BTreeMap::new();
            map.insert("numbers".into(), vec![1, 2, 3, 4, 5]);
            map.insert("empty".into(), vec![]);
            map
        })
    }
}

// Test more complex non-exhaustive cases
roundtrip_struct! {
    test_non_exhaustive_with_complex_fields {
        #[non_exhaustive]
        struct Test {
            id: u64 = u64::MAX,
            data: Vec<Person> = vec![
                Person {
                    name: "NonExhaustive Person".into(),
                    age: 99,
                    address: Address {
                        street: "NonExhaustive St".into(),
                        city: "NonExhaustive City".into(),
                        zip: 99999,
                    }
                }
            ],
            status: Status = Status::Inactive { reason: "Non-exhaustive test".into() },
            metadata: BTreeMap<String, Option<String>> = {
                let mut map = BTreeMap::new();
                map.insert("key1".into(), Some("value1".into()));
                map.insert("key2".into(), None);
                map.insert("key3".into(), Some("value3".into()));
                map
            }
        }
    }
}

// NOTE: Non-exhaustive struct with renamed fields test removed due to the same
// limitation described above regarding renamed fields.

// Test deeply nested hierarchies
roundtrip_struct! {
    test_deeply_nested_hierarchy {
        struct Level1 {
            data: String = "level1".into(),
            level2: Level2 = Level2 {
                data: "level2".into(),
                level3: Level3 {
                    data: "level3".into(),
                    level4: Level4 {
                        data: "level4".into(),
                        final_value: 42,
                    }
                }
            }
        }
    }

    test_recursive_like_structure {
        struct Node {
            value: u32 = 100,
            children: Vec<Node> = vec![
                Node {
                    value: 101,
                    children: vec![
                        Node {
                            value: 102,
                            children: vec![]
                        }
                    ]
                },
                Node {
                    value: 103,
                    children: vec![]
                }
            ]
        }
    }
}

// Additional helper structs for deep nesting tests
#[derive(Debug, Deserialize, PartialEq)]
struct Level2 {
    data: String,
    level3: Level3,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Level3 {
    data: String,
    level4: Level4,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Level4 {
    data: String,
    final_value: u32,
}

// Test edge cases with empty collections and special values
roundtrip_struct! {
    test_empty_collections {
        struct Test {
            empty_vec: Vec<String> = vec![],
            empty_map: BTreeMap<String, u32> = BTreeMap::new(),
            empty_set: BTreeSet<String> = BTreeSet::new(),
            simple_vec: Vec<String> = vec!["a".into(), "b".into(), "c".into()]
        }
    }

    test_special_string_values {
        struct Test {
            simple_string: String = "hello world".into(),
            number_string: String = "12345".into(),
            symbol_string: String = "!@#$%^&*()".into(),
            single_char: String = "x".into()
        }
    }

    test_extreme_numeric_values {
        struct Test {
            zero_u32: u32 = 0,
            max_u32: u32 = u32::MAX,
            zero_i32: i32 = 0,
            min_i32: i32 = i32::MIN,
            max_i32: i32 = i32::MAX,
            zero_f64: f64 = 0.0,
            negative_zero_f64: f64 = -0.0,
            large_f64: f64 = 1e10,
            small_f64: f64 = 1e-10
        }
    }
}