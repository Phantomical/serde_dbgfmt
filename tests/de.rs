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

            let mut de = serde_defmt::Deserializer::new(&text);

            let dst: $name = serde_path_to_error::deserialize(&mut de)
                .unwrap_or_else(|e| panic!("{}", e));
            de.end().expect("failed to deserialize");

            assert_eq!(src, dst);
        }
    )*}
}

roundtrip_struct! {
    test_basic_struct {
        struct Test {
            a: u32 = 88,
            b: String = "test".into()
        }
    }

    test_nonexhaustive_struct {
        #[non_exhaustive]
        struct Test {
            a: u32 = u32::MAX,
            b: u32 = 0,
            c: u32 = 77,
        }
    }

    test_all_bool_values {
        struct DataTypes {
            t: bool = true,
            f: bool = false,
        }
    }

    test_all_integers {
        struct Integers {
            v_u8: u8 = u8::MAX,
            v_u16: u16 = u16::MAX,
            v_u32: u32 = u32::MAX,
            v_u64: u64 = u64::MAX,
            v_u128: u128 = u128::MAX,

            v_i8_min: i8 = i8::MIN,
            v_i16_min: i16 = i16::MIN,
            v_i32_min: i32 = i32::MIN,
            v_i64_min: i64 = i64::MIN,
            v_i128_min: i128 = i128::MIN,

            v_i8_max: i8 = i8::MAX,
            v_i16_max: i16 = i16::MAX,
            v_i32_max: i32 = i32::MAX,
            v_i64_max: i64 = i64::MAX,
            v_i128_max: i128 = i128::MAX,
        }
    }

    test_f32 {
        struct Floats {
            one: f32 = 1.0,
            negative: f32 = -1.0,
        }
    }

    test_f64 {
        struct Floats {
            one: f64 = 1.0,
            negative: f64 = -1.0,
        }
    }

    test_btreemap {
        struct Map {
            map: BTreeMap<String, String> = {
                let iter = [
                    ("test", "value"),
                    ("a", "b"),
                    ("    ", "  "),
                    ("true", "false")
                ];

                BTreeMap::from_iter(
                    iter.into_iter()
                        .map(|(k, v)| (k.into(), v.into()))
                )
            },
        }
    }

    test_btreeset {
        struct Map {
            map: BTreeSet<String> = {
                let iter = [
                    "test", "value",
                    "a", "b",
                    "    ", "  ",
                    "true", "false"
                ];

                BTreeSet::from_iter(iter.into_iter().map(|v| v.into()))
            },
        }
    }

    test_vector {
        struct Test {
            vec: Vec<u32> = vec![0, 1, u32::MAX, 9919, 1337]
        }
    }
}
