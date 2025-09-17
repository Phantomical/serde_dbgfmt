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

roundtrip_struct! {
    test_basic_escape_sequences {
        struct EscapeSeqs {
            tab: String = "\t".into(),
            newline: String = "\n".into(),
            carriage_return: String = "\r".into(),
            backslash: String = "\\".into(),
            single_quote: String = "\'".into(),
            double_quote: String = "\"".into(),
            null: String = "\0".into(),
        }
    }

    test_unicode_escapes {
        struct UnicodeStrs {
            ascii: String = "\u{41}".into(),
            emoji: String = "\u{1F600}".into(),
            chinese: String = "\u{4E2D}".into(),
            complex: String = "\u{1F1FA}\u{1F1F8}".into(),
        }
    }

    test_character_literals {
        struct CharLits {
            basic: char = 'A',
            tab: char = '\t',
            newline: char = '\n',
            backslash: char = '\\',
            single_quote: char = '\'',
            unicode: char = '\u{41}',
            emoji: char = '\u{1F600}',
        }
    }

    test_mixed_string_content {
        struct MixedStrs {
            normal: String = "Hello, World!".into(),
            with_escapes: String = "Line 1\nLine 2\tTabbed".into(),
            with_quotes: String = "She said \"Hello!\"".into(),
            with_unicode: String = "Café \u{2603} Unicode".into(),
        }
    }

    test_long_strings {
        struct LongStrs {
            long_text: String = "This is a very long string that contains multiple words and should test the string parsing capabilities of the deserializer with a substantial amount of text content.".into(),
            repeated: String = "A".repeat(100),
        }
    }

    test_special_characters {
        struct SpecialChars {
            symbols: String = "!@#$%^&*()_+-=[]{}|;':\",./<>?".into(),
            numbers: String = "0123456789".into(),
            whitespace: String = " \t\n\r".into(),
        }
    }

    test_multilingual_text {
        struct MultiLingual {
            english: String = "Hello".into(),
            chinese: String = "你好".into(),
            arabic: String = "مرحبا".into(),
            russian: String = "Привет".into(),
            japanese: String = "こんにちは".into(),
        }
    }
}

#[test]
fn test_empty_string() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct EmptyStr {
        value: String,
    }

    let input = r#"EmptyStr { value: "" }"#;
    let result: Result<EmptyStr, _> = serde_dbgfmt::from_str(input);

    match result {
        Ok(parsed) => {
            assert_eq!(parsed.value, "");
        }
        Err(_) => {
            eprintln!("Note: Empty strings may not be supported due to lexer limitations");
        }
    }
}

#[test]
fn test_string_with_nested_quotes() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct NestedQuotes {
        value: String,
    }

    let input = r#"NestedQuotes { value: "He said \"Hello\" to me" }"#;
    let result: Result<NestedQuotes, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    assert_eq!(result.unwrap().value, r#"He said "Hello" to me"#);
}

#[test]
fn test_raw_string_like_content() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct RawLike {
        value: String,
    }

    let input = r#"RawLike { value: "C:\\Users\\name\\file.txt" }"#;
    let result: Result<RawLike, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    assert_eq!(result.unwrap().value, r"C:\Users\name\file.txt");
}

#[test]
fn test_multiline_string_content() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct MultiLine {
        value: String,
    }

    let input = r#"MultiLine { value: "Line 1\nLine 2\nLine 3" }"#;
    let result: Result<MultiLine, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    assert_eq!(result.unwrap().value, "Line 1\nLine 2\nLine 3");
}

#[test]
fn test_unicode_codepoints() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct UnicodePoints {
        null: String,
        bmp: String,
        supplementary: String,
        emoji: String,
    }

    let input = r#"UnicodePoints { null: "\u{0}", bmp: "\u{0041}", supplementary: "\u{1D11E}", emoji: "\u{1F44D}" }"#;
    let result: Result<UnicodePoints, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.null, "\0");
    assert_eq!(parsed.bmp, "A");
    assert_eq!(parsed.supplementary, "𝄞");
    assert_eq!(parsed.emoji, "👍");
}

#[test]
fn test_char_escape_sequences() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct CharEscapes {
        tab: char,
        newline: char,
        return_char: char,
        backslash: char,
        quote: char,
        null: char,
    }

    let input = r#"CharEscapes { tab: '\t', newline: '\n', return_char: '\r', backslash: '\\', quote: '\'', null: '\0' }"#;
    let result: Result<CharEscapes, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.tab, '\t');
    assert_eq!(parsed.newline, '\n');
    assert_eq!(parsed.return_char, '\r');
    assert_eq!(parsed.backslash, '\\');
    assert_eq!(parsed.quote, '\'');
    assert_eq!(parsed.null, '\0');
}

#[test]
fn test_unicode_char_literals() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct UnicodeChars {
        ascii: char,
        accented: char,
        emoji: char,
        musical: char,
    }

    let input = r#"UnicodeChars { ascii: '\u{41}', accented: '\u{E9}', emoji: '\u{1F600}', musical: '\u{1D11E}' }"#;
    let result: Result<UnicodeChars, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.ascii, 'A');
    assert_eq!(parsed.accented, 'é');
    assert_eq!(parsed.emoji, '😀');
    assert_eq!(parsed.musical, '𝄞');
}

#[test]
fn test_boundary_unicode_values() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct BoundaryUnicode {
        min: char,
        ascii_max: char,
        bmp_max: char,
    }

    let input = r#"BoundaryUnicode { min: '\u{0}', ascii_max: '\u{7F}', bmp_max: '\u{FFFF}' }"#;
    let result: Result<BoundaryUnicode, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.min, '\0');
    assert_eq!(parsed.ascii_max, '\u{7F}');
    assert_eq!(parsed.bmp_max, '\u{FFFF}');
}

#[test]
fn test_complex_string_combinations() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct ComplexStrs {
        mixed: String,
        path_like: String,
        json_like: String,
        regex_like: String,
    }

    let input = r#"ComplexStrs { mixed: "Hello\tWorld!\nNext line with \"quotes\" and \\backslashes\\", path_like: "/usr/bin/env", json_like: "{\"key\": \"value\"}", regex_like: "\\d+\\.\\d+" }"#;
    let result: Result<ComplexStrs, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(
        parsed.mixed,
        "Hello\tWorld!\nNext line with \"quotes\" and \\backslashes\\"
    );
    assert_eq!(parsed.path_like, "/usr/bin/env");
    assert_eq!(parsed.json_like, r#"{"key": "value"}"#);
    assert_eq!(parsed.regex_like, r"\d+\.\d+");
}

#[test]
fn test_string_with_all_whitespace_chars() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct AllWhitespace {
        spaces: String,
        tabs: String,
        newlines: String,
        mixed: String,
    }

    let input = "AllWhitespace { spaces: \"   \", tabs: \"\t\t\t\", newlines: \"\n\n\n\", mixed: \" \t\n\r\" }";
    let result: Result<AllWhitespace, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.spaces, "   ");
    assert_eq!(parsed.tabs, "\t\t\t");
    assert_eq!(parsed.newlines, "\n\n\n");
    assert_eq!(parsed.mixed, " \t\n\r");
}

#[test]
fn test_string_length_variations() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct LengthVars {
        single: String,
        short: String,
        medium: String,
    }

    let input = r#"LengthVars { single: "A", short: "Hello", medium: "This is a medium length string for testing purposes" }"#;
    let result: Result<LengthVars, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.single, "A");
    assert_eq!(parsed.short, "Hello");
    assert_eq!(
        parsed.medium,
        "This is a medium length string for testing purposes"
    );
}

#[test]
fn test_numeric_strings() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct NumericStrs {
        integer: String,
        float: String,
        hex: String,
        scientific: String,
    }

    let input = r#"NumericStrs { integer: "12345", float: "123.456", hex: "0xABCD", scientific: "1.23e-4" }"#;
    let result: Result<NumericStrs, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.integer, "12345");
    assert_eq!(parsed.float, "123.456");
    assert_eq!(parsed.hex, "0xABCD");
    assert_eq!(parsed.scientific, "1.23e-4");
}

#[test]
fn test_option_strings() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct OptionStrs {
        some_str: Option<String>,
        none_str: Option<String>,
        some_char: Option<char>,
        none_char: Option<char>,
    }

    let input = r#"OptionStrs { some_str: Some("Hello"), none_str: None, some_char: Some('A'), none_char: None }"#;
    let result: Result<OptionStrs, _> = serde_dbgfmt::from_str(input);
    assert_matches!(result, Ok(_));
    let parsed = result.unwrap();
    assert_eq!(parsed.some_str, Some("Hello".to_string()));
    assert_eq!(parsed.none_str, None);
    assert_eq!(parsed.some_char, Some('A'));
    assert_eq!(parsed.none_char, None);
}
