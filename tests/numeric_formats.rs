use assert_matches::assert_matches;
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
            let mut de = serde_dbgfmt::Deserializer::new(&text);

            let dst: $name = serde_path_to_error::deserialize(&mut de)
                .unwrap_or_else(|e| panic!("{}", e));
            de.end().expect("failed to deserialize");

            assert_eq!(src, dst);
        }
    )*}
}

macro_rules! test_direct_format {
    ($test_name:ident, $input:expr, $expected:expr, $ty:ty) => {
        #[test]
        fn $test_name() {
            let result: Result<$ty, _> = serde_dbgfmt::from_str($input);
            assert_matches!(result, Ok(_), "Failed to parse: {}", $input);
            assert_eq!(result.unwrap(), $expected);
        }
    };
}

roundtrip_struct! {
    test_hexadecimal_lowercase {
        struct HexLower {
            small: u8 = 0xff,
            medium: u16 = 0xabcd,
            large: u32 = 0xdeadbeef,
            max: u64 = 0x123456789abcdef0,
        }
    }

    test_hexadecimal_mixed_case {
        struct HexMixed {
            small: u8 = 0xff,
            medium: u16 = 0xabcd,
            large: u32 = 0xdeadbeef,
            max: u64 = 0x123456789abcdef0,
        }
    }

    test_octal_numbers {
        struct OctalNums {
            small: u8 = 0o77,
            medium: u16 = 0o1234,
            large: u32 = 0o17777777777,
            zero: u8 = 0o0,
        }
    }

    test_octal_variations {
        struct OctalVars {
            small: u8 = 0o77,
            medium: u16 = 0o1234,
            large: u32 = 0o17777777777,
        }
    }

    test_binary_numbers {
        struct BinaryNums {
            small: u8 = 0b11111111,
            medium: u16 = 0b1010101010101010,
            large: u32 = 0b11110000111100001111000011110000,
            zero: u8 = 0b0,
        }
    }

    test_binary_variations {
        struct BinaryVars {
            small: u8 = 0b11111111,
            medium: u16 = 0b1010101010101010,
            large: u32 = 0b11110000111100001111000011110000,
        }
    }

    test_leading_zeros {
        struct LeadingZeros {
            small: u32 = 000123,
            medium: u32 = 0000456,
            large: u32 = 000000789,
        }
    }

    test_signed_positive_explicit {
        struct PositiveSigned {
            i8_val: i8 = 127,
            i16_val: i16 = 32767,
            i32_val: i32 = 2147483647,
            i64_val: i64 = 9223372036854775807,
        }
    }

    test_signed_negative {
        struct NegativeSigned {
            i8_val: i8 = -128,
            i16_val: i16 = -32768,
            i32_val: i32 = -2147483648,
            i64_val: i64 = -9223372036854775808,
        }
    }

    test_scientific_notation_lowercase {
        struct SciLower {
            simple: f64 = 1e5,
            with_decimal: f64 = 2.5e10,
            negative_exp: f64 = 3.14e-5,
            zero_exp: f64 = 42.0e0,
        }
    }

    test_scientific_notation_uppercase {
        struct SciUpper {
            simple: f64 = 1E5,
            with_decimal: f64 = 2.5E10,
            negative_exp: f64 = 3.14E-5,
            positive_exp: f64 = 1.23E+6,
        }
    }

    test_mixed_float_formats {
        struct MixedFloats {
            regular: f64 = 123.456,
            scientific: f64 = 1.23e4,
            small: f64 = 0.000001,
            large: f64 = 1000000.0,
        }
    }

    test_f32_precision {
        struct F32Test {
            small: f32 = 0.000001,
            large: f32 = 1000000.0,
            scientific: f32 = 1.23e-7,
        }
    }

    test_all_integer_types {
        struct AllInts {
            u8_max: u8 = 255,
            u16_max: u16 = 65535,
            u32_max: u32 = 4294967295,
            u64_max: u64 = 18446744073709551615,
            i8_min: i8 = -128,
            i8_max: i8 = 127,
            i16_min: i16 = -32768,
            i16_max: i16 = 32767,
            i32_min: i32 = -2147483648,
            i32_max: i32 = 2147483647,
            i64_min: i64 = -9223372036854775808,
            i64_max: i64 = 9223372036854775807,
        }
    }

    test_i128_u128_types {
        struct LargeInts {
            u128_max: u128 = 340282366920938463463374607431768211455,
            i128_min: i128 = -170141183460469231731687303715884105728,
            i128_max: i128 = 170141183460469231731687303715884105727,
        }
    }
}

test_direct_format!(test_direct_hex_parsing, "0xff", 255u8, u8);
test_direct_format!(test_direct_hex_case_insensitive, "0xff", 255u8, u8);
test_direct_format!(test_direct_octal_parsing, "0o77", 63u8, u8);
test_direct_format!(test_direct_octal_variations, "0o77", 63u8, u8);
test_direct_format!(test_direct_binary_parsing, "0b11111111", 255u8, u8);
test_direct_format!(test_direct_binary_variations, "0b11111111", 255u8, u8);

#[test]
fn test_explicit_positive_sign() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct WithSign {
        value: i32,
    }

    let input = "WithSign { value: +123 }";
    let result: Result<WithSign, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    assert_eq!(result.unwrap().value, 123);
}

#[test]
fn test_float_with_explicit_positive_sign() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct FloatSign {
        value: f64,
    }

    let input = "FloatSign { value: +123.456 }";
    let result: Result<FloatSign, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    assert_eq!(result.unwrap().value, 123.456);
}

#[test]
fn test_scientific_with_signs() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct SciSigns {
        pos_exp: f64,
        neg_exp: f64,
        pos_num: f64,
        neg_num: f64,
    }

    let input =
        "SciSigns { pos_exp: 1.5e+10, neg_exp: 2.5e-5, pos_num: +3.14e5, neg_num: -2.71e3 }";
    let result: Result<SciSigns, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.pos_exp, 1.5e10);
    assert_eq!(parsed.neg_exp, 2.5e-5);
    assert_eq!(parsed.pos_num, 3.14e5);
    assert_eq!(parsed.neg_num, -2.71e3);
}

#[test]
fn test_zero_variations() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Zeros {
        dec: u32,
        hex: u32,
        oct: u32,
        bin: u32,
        float_zero: f64,
    }

    let input = "Zeros { dec: 0, hex: 0x0, oct: 0o0, bin: 0b0, float_zero: 0.0 }";
    let result: Result<Zeros, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.dec, 0);
    assert_eq!(parsed.hex, 0);
    assert_eq!(parsed.oct, 0);
    assert_eq!(parsed.bin, 0);
    assert_eq!(parsed.float_zero, 0.0);
}

#[test]
fn test_mixed_radix_combinations() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct MixedRadix {
        decimal: u32,
        hex: u32,
        octal: u32,
        binary: u32,
    }

    let input = "MixedRadix { decimal: 255, hex: 0xff, octal: 0o377, binary: 0b11111111 }";
    let result: Result<MixedRadix, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.decimal, 255);
    assert_eq!(parsed.hex, 255);
    assert_eq!(parsed.octal, 255);
    assert_eq!(parsed.binary, 255);
}

#[test]
fn test_large_hex_numbers() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct LargeHex {
        val64: u64,
        val128: u128,
    }

    let input =
        "LargeHex { val64: 0xffffffffffffffff, val128: 0xffffffffffffffffffffffffffffffff }";
    let result: Result<LargeHex, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.val64, u64::MAX);
    assert_eq!(parsed.val128, u128::MAX);
}

#[test]
fn test_edge_float_values() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct EdgeFloats {
        tiny: f64,
        huge: f64,
        precision: f64,
    }

    let input = "EdgeFloats { tiny: 2.2250738585072014e-308, huge: 1.7976931348623157e308, precision: 0.1234567890123456 }";
    let result: Result<EdgeFloats, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
}

#[test]
fn test_negative_hex_signed() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct NegHex {
        value: i32,
    }

    let input = "NegHex { value: -0xff }";
    let result: Result<NegHex, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    assert_eq!(result.unwrap().value, -255);
}

#[test]
fn test_complex_numeric_combinations() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct ComplexNums {
        hex_signed: i64,
        oct_unsigned: u32,
        bin_signed: i16,
        sci_float: f64,
        regular_float: f32,
        zero_pad: u32,
    }

    let input = "ComplexNums { hex_signed: -0xabcdef, oct_unsigned: 0o1234567, bin_signed: -0b101010, sci_float: 1.23e-4, regular_float: 456.789, zero_pad: 007890 }";
    let result: Result<ComplexNums, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.hex_signed, -0xabcdef);
    assert_eq!(parsed.oct_unsigned, 0o1234567);
    assert_eq!(parsed.bin_signed, -0b101010);
    assert_eq!(parsed.sci_float, 1.23e-4);
    assert_eq!(parsed.regular_float, 456.789);
    assert_eq!(parsed.zero_pad, 7890);
}
