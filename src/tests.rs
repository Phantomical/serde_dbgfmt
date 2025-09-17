use crate::error::LexerError;
use crate::lex::{Lexer, Token, TokenKind};

macro_rules! declare_lexer_tests
{
    {$(
        $name:ident => $text:literal == $tokens:expr;
    )*} => {$(
        #[test]
        fn $name() {
            use std::iter::Iterator;
            use crate::lex::TokenKind;

            let expected: Vec<(TokenKind, &str)> = $tokens
                .into_iter()
                .chain([(TokenKind::Eof, "")])
                .collect();

            let tokens = match crate::tests::lex_all($text) {
                Ok(tokens) => tokens,
                Err(e) => panic!("Failed to lex `{}`: {e}", $text),
            };

            let iter = tokens.iter().zip(expected.iter()).enumerate();
            for (index, (token, &(kind, value))) in iter {
                assert_eq!(
                    token.kind, kind,
                    "token kind at index {index} in test `{}` did not match",
                    $text
                );
                assert_eq!(
                    token.value, value,
                    "token value at index {index} in test `{}` did not match",
                    $text
                );
            }

            assert_eq!(
                tokens.len(),
                expected.len(),
                "number of tokens in expression \"{}\" did not match the number of tokens expected",
                $text
            );
        }
    )*}
}

fn lex_all(input: &str) -> Result<Vec<Token>, LexerError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.parse_token()?;
        let is_eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if is_eof {
            break;
        }
    }

    Ok(tokens)
}

mod lexer {
    use crate::error::LexerError;
    use crate::lex::{Lexer, Token, TokenKind};

    declare_lexer_tests! {
        // Whitespace tests
        whitespace_spaces => "   " == [];
        whitespace_other => "\t\n\r" == [];

        // Empty input
        empty_input => "" == [];

        // Basic identifiers
        ident_test => "test" == [(TokenKind::Ident, "test")];
        ident_hello_world => "hello_world" == [(TokenKind::Ident, "hello_world")];
        ident_test123 => "Test123" == [(TokenKind::Ident, "Test123")];
        ident_single_a => "a" == [(TokenKind::Ident, "a")];
        ident_single_z => "Z" == [(TokenKind::Ident, "Z")];

        // Unicode identifiers
        ident_cafe => "café" == [(TokenKind::Ident, "café")];
        ident_greek => "αβγ" == [(TokenKind::Ident, "αβγ")];
        ident_japanese => "変数" == [(TokenKind::Ident, "変数")];

        // Multiple identifiers
        idents_hello_world => "hello world" == [
            (TokenKind::Ident, "hello"),
            (TokenKind::Ident, "world")
        ];

        // Single punctuation marks
        punct_open_brace => "{" == [(TokenKind::Punct, "{")];
        punct_close_brace => "}" == [(TokenKind::Punct, "}")];
        punct_open_bracket => "[" == [(TokenKind::Punct, "[")];
        punct_close_bracket => "]" == [(TokenKind::Punct, "]")];
        punct_colon => ":" == [(TokenKind::Punct, ":")];
        punct_comma => "," == [(TokenKind::Punct, ",")];
        punct_open_paren => "(" == [(TokenKind::Punct, "(")];
        punct_close_paren => ")" == [(TokenKind::Punct, ")")];
        punct_plus => "+" == [(TokenKind::Punct, "+")];
        punct_minus => "-" == [(TokenKind::Punct, "-")];
        punct_double_dot => ".." == [(TokenKind::Punct, "..")];

        // Multiple punctuation
        punct_braces_brackets => "{}[]" == [
            (TokenKind::Punct, "{"),
            (TokenKind::Punct, "}"),
            (TokenKind::Punct, "["),
            (TokenKind::Punct, "]")
        ];

        // Decimal integers
        int_zero => "0" == [(TokenKind::Integer, "0")];
        int_one => "1" == [(TokenKind::Integer, "1")];
        int_42 => "42" == [(TokenKind::Integer, "42")];
        int_large => "123456789" == [(TokenKind::Integer, "123456789")];

        // Hexadecimal integers
        hex_zero => "0x0" == [(TokenKind::Integer, "0x0")];
        hex_one => "0x1" == [(TokenKind::Integer, "0x1")];
        hex_ff_lower => "0xff" == [(TokenKind::Integer, "0xff")];
        hex_ff_upper => "0xFF" == [(TokenKind::Integer, "0xFF")];
        hex_123abc => "0x123abc" == [(TokenKind::Integer, "0x123abc")];
        hex_x_upper => "0X0" == [(TokenKind::Integer, "0X0")];
        hex_deadbeef => "0Xdeadbeef" == [(TokenKind::Integer, "0Xdeadbeef")];

        // Octal integers
        oct_zero => "0o0" == [(TokenKind::Integer, "0o0")];
        oct_seven => "0o7" == [(TokenKind::Integer, "0o7")];
        oct_777 => "0o777" == [(TokenKind::Integer, "0o777")];
        oct_o_upper => "0O0" == [(TokenKind::Integer, "0O0")];
        oct_123 => "0O123" == [(TokenKind::Integer, "0O123")];

        // Binary integers
        bin_zero => "0b0" == [(TokenKind::Integer, "0b0")];
        bin_one => "0b1" == [(TokenKind::Integer, "0b1")];
        bin_101010 => "0b101010" == [(TokenKind::Integer, "0b101010")];
        bin_b_upper => "0B0" == [(TokenKind::Integer, "0B0")];
        bin_11110000 => "0B11110000" == [(TokenKind::Integer, "0B11110000")];

        // Multiple integers
        ints_42_0xff => "42 0xff" == [
            (TokenKind::Integer, "42"),
            (TokenKind::Integer, "0xff")
        ];

        // Basic floats
        float_zero => "0.0" == [(TokenKind::Float, "0.0")];
        float_one => "1.0" == [(TokenKind::Float, "1.0")];
        float_42_5 => "42.5" == [(TokenKind::Float, "42.5")];
        float_123_456789 => "123.456789" == [(TokenKind::Float, "123.456789")];

        // Scientific notation with 'e'
        sci_1e0 => "1e0" == [(TokenKind::Float, "1e0")];
        sci_1e5 => "1e5" == [(TokenKind::Float, "1e5")];
        sci_1e_minus5 => "1e-5" == [(TokenKind::Float, "1e-5")];
        sci_1e_plus5 => "1e+5" == [(TokenKind::Float, "1e+5")];
        sci_123e_minus10 => "123e-10" == [(TokenKind::Float, "123e-10")];

        // Scientific notation with 'E'
        sci_1e0_upper => "1E0" == [(TokenKind::Float, "1E0")];
        sci_1e5_upper => "1E5" == [(TokenKind::Float, "1E5")];
        sci_1e_minus5_upper => "1E-5" == [(TokenKind::Float, "1E-5")];
        sci_1e_plus5_upper => "1E+5" == [(TokenKind::Float, "1E+5")];

        // Combined decimal and scientific notation
        sci_1_5e0 => "1.5e0" == [(TokenKind::Float, "1.5e0")];
        sci_42_0e_plus3 => "42.0e+3" == [(TokenKind::Float, "42.0e+3")];
        sci_99_1212e_minus22 => "99.1212e-22" == [(TokenKind::Float, "99.1212e-22")];

        // Multiple floats
        floats_1_0_2_5 => "1.0 2.5" == [
            (TokenKind::Float, "1.0"),
            (TokenKind::Float, "2.5")
        ];

        // Basic strings
        string_empty => "\"\"" == [(TokenKind::String, "\"\"")];
        string_hello => "\"hello\"" == [(TokenKind::String, "\"hello\"")];
        string_hello_world => "\"hello world\"" == [(TokenKind::String, "\"hello world\"")];
        string_test_123 => "\"test 123 !@#\"" == [(TokenKind::String, "\"test 123 !@#\"")];
        string_escaped_quotes => r#""hello \"world\"""# == [(TokenKind::String, "\"hello \\\"world\\\"\"")];
        string_unicode_escape => "\"\\u{41} blah\"" == [(TokenKind::String, "\"\\u{41} blah\"")];
        string_newline => "\"line1\\nline2\"" == [(TokenKind::String, "\"line1\\nline2\"")];
        string_tab => "\"tab\\there\"" == [(TokenKind::String, "\"tab\\there\"")];

        char_multislash => r#"'\\'"# == [(TokenKind::Char, r#"'\\'"#)];
        string_multislash => r#""\\""# == [(TokenKind::String, r#""\\""#)];

        // Multiple strings
        strings_first_second => "\"first\" \"second\"" == [
            (TokenKind::String, "\"first\""),
            (TokenKind::String, "\"second\"")
        ];

        // Basic characters
        char_a => "'a'" == [(TokenKind::Char, "'a'")];
        char_z => "'Z'" == [(TokenKind::Char, "'Z'")];
        char_zero => "'0'" == [(TokenKind::Char, "'0'")];
        char_at => "'@'" == [(TokenKind::Char, "'@'")];
        char_newline => r"'\n'" == [(TokenKind::Char, r"'\n'")];
        char_tab => r"'\t'" == [(TokenKind::Char, r"'\t'")];
        char_backslash => r"'\\'" == [(TokenKind::Char, r"'\\'")];
        char_alpha => "'α'" == [(TokenKind::Char, "'α'")];
        char_crab => "'🦀'" == [(TokenKind::Char, "'🦀'")];

        // Multiple characters
        chars_a_b => "'a' 'b'" == [
            (TokenKind::Char, "'a'"),
            (TokenKind::Char, "'b'")
        ];

        // Complex combinations - struct-like debug output
        struct_test => "Test { field: \"value\" }" == [
            (TokenKind::Ident, "Test"),
            (TokenKind::Punct, "{"),
            (TokenKind::Ident, "field"),
            (TokenKind::Punct, ":"),
            (TokenKind::String, "\"value\""),
            (TokenKind::Punct, "}")
        ];

        // Tuple struct debug output
        tuple_struct => "Point(1, 2)" == [
            (TokenKind::Ident, "Point"),
            (TokenKind::Punct, "("),
            (TokenKind::Integer, "1"),
            (TokenKind::Punct, ","),
            (TokenKind::Integer, "2"),
            (TokenKind::Punct, ")")
        ];

        // Array/Vec debug output
        array_123 => "[1, 2, 3]" == [
            (TokenKind::Punct, "["),
            (TokenKind::Integer, "1"),
            (TokenKind::Punct, ","),
            (TokenKind::Integer, "2"),
            (TokenKind::Punct, ","),
            (TokenKind::Integer, "3"),
            (TokenKind::Punct, "]")
        ];


        // Mixed numeric types
        mixed_nums => "42 3.14 0xff" == [
            (TokenKind::Integer, "42"),
            (TokenKind::Float, "3.14"),
            (TokenKind::Integer, "0xff")
        ];

        // Nested structures
        nested_struct => "Outer { inner: Inner { value: 42 } }" == [
            (TokenKind::Ident, "Outer"),
            (TokenKind::Punct, "{"),
            (TokenKind::Ident, "inner"),
            (TokenKind::Punct, ":"),
            (TokenKind::Ident, "Inner"),
            (TokenKind::Punct, "{"),
            (TokenKind::Ident, "value"),
            (TokenKind::Punct, ":"),
            (TokenKind::Integer, "42"),
            (TokenKind::Punct, "}"),
            (TokenKind::Punct, "}")
        ];

        // Whitespace handling
        whitespace_padded => "  test  " == [(TokenKind::Ident, "test")];
        whitespace_tabs_newlines => "\ttest\n" == [(TokenKind::Ident, "test")];
        whitespace_crlf => "\r\ntest\r\n" == [(TokenKind::Ident, "test")];
        whitespace_between => "a   b" == [
            (TokenKind::Ident, "a"),
            (TokenKind::Ident, "b")
        ];
        whitespace_mixed => "{\n\t\"test\"\n}" == [
            (TokenKind::Punct, "{"),
            (TokenKind::String, "\"test\""),
            (TokenKind::Punct, "}")
        ];

        // Edge cases
        edge_zero_a => "0a" == [
            (TokenKind::Integer, "0"),
            (TokenKind::Ident, "a")
        ];
        edge_123abc => "123abc" == [
            (TokenKind::Integer, "123"),
            (TokenKind::Ident, "abc")
        ];

        // Unicode edge cases
        unicode_string => "\"🦀 Rust 🔥\"" == [(TokenKind::String, "\"🦀 Rust 🔥\"")];

        // Scientific notation edge cases
        sci_1e10 => "1e10" == [(TokenKind::Float, "1e10")];
        sci_123e_minus5 => "123E-5" == [(TokenKind::Float, "123E-5")];
        sci_1_space_e => "1 e" == [
            (TokenKind::Integer, "1"),
            (TokenKind::Ident, "e")
        ];
    }

    #[test]
    fn test_very_long_tokens() {
        use super::{lex_all, TokenKind};

        // Very long identifier
        let long_ident = "a".repeat(1000);
        let tokens = lex_all(&long_ident).expect("lexing should succeed");
        assert_eq!(tokens.len(), 2); // ident + eof
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].value, long_ident);
        assert_eq!(tokens[1].kind, TokenKind::Eof);

        // Very long string
        let long_string_content = "x".repeat(1000);
        let long_string = format!("\"{}\"", long_string_content);
        let tokens = lex_all(&long_string).expect("lexing should succeed");
        assert_eq!(tokens.len(), 2); // string + eof
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[0].value, long_string);
        assert_eq!(tokens[1].kind, TokenKind::Eof);

        // Very long number
        let long_number = "1".repeat(1000);
        let tokens = lex_all(&long_number).expect("lexing should succeed");
        assert_eq!(tokens.len(), 2); // integer + eof
        assert_eq!(tokens[0].kind, TokenKind::Integer);
        assert_eq!(tokens[0].value, long_number);
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }

    fn lex_all(input: &str) -> Result<Vec<Token>, LexerError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();

        loop {
            let token = lexer.parse_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn assert_lex_error(input: &str) {
        let result = lex_all(input);
        assert!(
            result.is_err(),
            "expected lexing error for input: {}",
            input
        );
    }

    #[test]
    fn test_error_cases() {
        // Invalid characters
        assert_lex_error("@");
        assert_lex_error("#");
        assert_lex_error("$");
        assert_lex_error("%");
        assert_lex_error("&");
        assert_lex_error("*");
        assert_lex_error("=");
        assert_lex_error("?");
        assert_lex_error("\\");
        assert_lex_error("|");
        assert_lex_error("~");
        assert_lex_error("`");

        // Unterminated strings
        assert_lex_error("\"unterminated");
        assert_lex_error("\"unterminated\\");

        // Unterminated characters
        assert_lex_error("'unterminated");
        assert_lex_error("'a");

        // Single dot (not double dot)
        assert_lex_error(".");

        // Invalid number sequences would be caught by number parsing,
        // but the lexer should accept them as tokens
        // The actual validation happens in the parser
    }

    #[test]
    fn test_token_is_punct_method() {
        let token = Token {
            kind: TokenKind::Punct,
            value: "{",
        };
        assert!(token.is_punct("{"));
        assert!(!token.is_punct("}"));

        let token = Token {
            kind: TokenKind::Ident,
            value: "test",
        };
        assert!(!token.is_punct("test"));
    }
}
