use std::borrow::Cow;

use serde::de::value::BorrowedStrDeserializer;
use serde::de::{Deserializer as _, Error as _, *};

use crate::error::Expected;
use crate::lex::{Lexer, Token, TokenKind};
use crate::Error;

/// A serde deserializer for rust's debug format.
pub struct Deserializer<'de> {
    total: &'de str,
    lexer: Lexer<'de>,
}

impl<'de> Deserializer<'de> {
    /// Create a deserializer to deserialize from a string.
    pub fn new(data: &'de str) -> Self {
        Self {
            total: data,
            lexer: Lexer::new(data),
        }
    }

    /// The `end` method should be called after a value has been fully
    /// deserialized. This allows the deserializer to validate that the input
    /// stream is at the end or that it only has trailing whitespace.
    ///
    /// If you would like to parse multiple objects in a stream then you can do
    /// that by calling `deserialize` multiple times and then calling `end` at
    /// the end.
    pub fn end(&mut self) -> Result<(), Error> {
        let token = self.lexer.parse_token()?;
        if token.kind != TokenKind::Eof {
            return Err(Error::unexpected_token(token, TokenKind::Eof));
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Sign {
    Positive,
    Negative,
}

struct Integer<'de> {
    sign: Sign,
    value: &'de str,
    span: &'de str,
}

struct Float<'de> {
    sign: Sign,
    value: &'de str,
    span: &'de str,
    kind: TokenKind,
}

struct Str<'de> {
    span: &'de str,
    value: Cow<'de, str>,
}

impl<'de> Deserializer<'de> {
    fn join_spans(&self, a: &'de str, b: &'de str) -> &'de str {
        let range = self.total.as_bytes().as_ptr_range();
        let range = range.start..=range.end;

        let a_range = a.as_bytes().as_ptr_range();
        let b_range = b.as_bytes().as_ptr_range();

        let start = a_range.start.min(b_range.start);
        let end = a_range.end.max(b_range.end);

        // Ensure that the entire range is contained within self.total
        assert!(range.contains(&start));
        assert!(range.contains(&end));

        // Now, since we know that a and b are part of self.total we can go and figure
        // out the indices we need to get the combined span that covers both.

        let offset1 = start as usize - *range.start() as usize;
        let offset2 = end as usize - *range.start() as usize;

        &self.total[offset1..offset2]
    }

    fn peek(&self) -> Result<Token<'de>, Error> {
        let mut lexer = self.lexer.clone();

        lexer.parse_token().map_err(From::from)
    }

    fn peek2(&self) -> Result<Token<'de>, Error> {
        let mut lexer = self.lexer.clone();

        lexer.parse_token()?;
        lexer.parse_token().map_err(From::from)
    }

    fn parse_integer(&mut self) -> Result<Integer<'de>, Error> {
        let mut token = self.lexer.parse_token()?;
        let mut sign = Sign::Positive;
        let mut sign_span = None;

        if matches!(
            token,
            Token {
                kind: TokenKind::Punct,
                value: "+" | "-"
            }
        ) {
            sign = match token.value {
                "+" => Sign::Positive,
                "-" => Sign::Negative,
                _ => unreachable!(),
            };
            sign_span = Some(token.value);

            token = self.lexer.parse_token()?;
        }

        match token.kind {
            TokenKind::Integer => Ok(Integer {
                sign,
                value: token.value,
                span: match sign_span {
                    Some(span) => self.join_spans(span, token.value),
                    None => token.value,
                },
            }),
            _ => Err(Error::unexpected_token(token, TokenKind::Integer)),
        }
    }

    fn parse_float(&mut self) -> Result<Float<'de>, Error> {
        let mut token = self.lexer.parse_token()?;
        let mut sign = Sign::Positive;
        let mut sign_span = None;

        if matches!(
            token,
            Token {
                kind: TokenKind::Punct,
                value: "+" | "-"
            }
        ) {
            sign = match token.value {
                "+" => Sign::Positive,
                "-" => Sign::Negative,
                _ => unreachable!(),
            };
            sign_span = Some(token.value);

            token = self.lexer.parse_token()?;
        }

        let span = match sign_span {
            Some(span) => self.join_spans(span, token.value),
            None => token.value,
        };

        match token.kind {
            TokenKind::Float => Ok(Float {
                sign,
                value: token.value,
                span,
                kind: token.kind,
            }),
            TokenKind::Ident if token.value.eq_ignore_ascii_case("NaN") => Ok(Float {
                sign,
                value: token.value,
                span,
                kind: token.kind,
            }),
            _ => Err(Error::unexpected_token(token, TokenKind::Float)),
        }
    }

    fn parse_ident(&mut self) -> Result<&'de str, Error> {
        let token = self.lexer.parse_token()?;

        match token.kind {
            TokenKind::Ident => Ok(token.value),
            _ => Err(Error::unexpected_token(token, TokenKind::Ident)),
        }
    }

    fn parse_ident_exact(&mut self, expected: &'de str) -> Result<(), Error> {
        let token = self.lexer.parse_token()?;

        match token.kind {
            TokenKind::Ident if token.value == expected => Ok(()),
            TokenKind::Ident => Err(Error::unexpected_token(token, expected)),
            _ => Err(Error::unexpected_token(token, TokenKind::Ident)),
        }
    }

    fn parse_string(&mut self) -> Result<Str<'de>, Error> {
        let token = self.lexer.parse_token()?;
        if token.kind != TokenKind::String {
            return Err(Error::unexpected_token(token, TokenKind::String));
        }

        let inner = &token.value[1..token.value.len() - 1];
        Ok(Str {
            span: token.value,
            value: unescape(inner)?,
        })
    }

    fn parse_char(&mut self) -> Result<Str<'de>, Error> {
        let token = self.lexer.parse_token()?;
        if token.kind != TokenKind::Char {
            return Err(Error::unexpected_token(token, TokenKind::Char));
        }

        let inner = &token.value[1..token.value.len() - 1];
        Ok(Str {
            span: token.value,
            value: unescape(inner)?,
        })
    }

    fn parse_punct(&mut self, punct: char) -> Result<(), Error> {
        self.parse_punct_ex(punct, |value| {
            let mut buffer = [0u8; 4];
            let text = punct.encode_utf8(&mut buffer);

            value == text
        })
        .map(drop)
    }

    fn parse_punct_ex<F>(
        &mut self,
        expected: impl Into<Expected>,
        func: F,
    ) -> Result<&'de str, Error>
    where
        F: FnOnce(&str) -> bool,
    {
        let token = self.lexer.parse_token()?;
        if token.kind != TokenKind::Punct {
            return Err(Error::unexpected_token(token, expected));
        }

        if !func(token.value) {
            return Err(Error::unexpected_token(token, expected));
        }

        Ok(token.value)
    }

    fn deserialize_struct_dyn<V>(&mut self, name: &'de str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.parse_ident_exact(name)?;
        self.parse_punct('{')?;
        let value = visitor.visit_map(DebugStructAccess(&mut *self))?;
        self.parse_punct('}')?;
        Ok(value)
    }

    fn deserialize_tuple_struct_dyn<V>(
        &mut self,
        name: &'de str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.parse_ident_exact(name)?;
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_unit_struct_dyn<V>(
        &mut self,
        name: &'de str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.parse_ident_exact(name)?;

        visitor.visit_unit()
    }
}

macro_rules! deserialize_unsigned {
    ($deserialize:ident, $uint:ty, $visit:ident) => {
        fn $deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            let int = self.parse_integer()?;
            let result = match int.value.get(..2) {
                _ if int.sign == Sign::Negative => "-1".parse(),
                Some("0x" | "0X") => <$uint>::from_str_radix(&int.value[2..], 16),
                Some("0o" | "0O") => <$uint>::from_str_radix(&int.value[2..], 8),
                Some("0b" | "0B") => <$uint>::from_str_radix(&int.value[2..], 2),
                _ => int.value.parse(),
            };

            match result {
                Ok(value) => visitor.$visit(value),
                Err(e) => Err(Error::parse_int(int.span, e)),
            }
        }
    };
}

macro_rules! deserialize_signed {
    ($deserialize:ident, $int:ty, $visit:ident) => {
        fn $deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            let int = self.parse_integer()?;
            let (rest, radix) = match int.value.get(..2) {
                Some("0x" | "0X") => (&int.value[2..], 16),
                Some("0o" | "0O") => (&int.value[2..], 8),
                Some("0b" | "0B") => (&int.value[2..], 2),
                _ => (int.value, 10),
            };

            let trimmed = match rest.trim_matches('0') {
                "" => "0",
                trimmed => trimmed,
            };

            // Copy the string to a temporary buffer so that we get the proper error type
            // when parsing overlarge signed integers.
            let mut storage = [0xFF; <$int>::MAX.ilog10() as usize + 2];
            storage[0] = match int.sign {
                Sign::Positive => b'+',
                Sign::Negative => b'-',
            };

            let len = trimmed.len().min(storage.len() - 1);
            storage[1..][..len].copy_from_slice(&trimmed.as_bytes()[..len]);

            let value = unsafe { std::str::from_utf8_unchecked(&storage[..len + 1]) };

            match <$int>::from_str_radix(value, radix) {
                Ok(value) => visitor.$visit(value),
                Err(e) => Err(Error::parse_int(int.span, e)),
            }
        }
    };
}

impl<'de> serde::de::Deserializer<'de> for &'_ mut Deserializer<'de> {
    type Error = Error;

    fn is_human_readable(&self) -> bool {
        true
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.peek()?;
        match (token.kind, token.value) {
            (TokenKind::String, _) => self.deserialize_str(visitor),
            (TokenKind::Char, _) => self.deserialize_char(visitor),

            (TokenKind::Integer, _) => self.deserialize_u64(visitor),
            (TokenKind::Float, _) => self.deserialize_f64(visitor),
            (TokenKind::Punct, sign @ ("+" | "-")) => {
                let peek2 = self.peek2()?;
                match peek2.kind {
                    TokenKind::Integer if sign == "+" => self.deserialize_u64(visitor),
                    TokenKind::Integer if sign == "-" => self.deserialize_i64(visitor),
                    TokenKind::Integer => unreachable!(),
                    TokenKind::Float => self.deserialize_f64(visitor),
                    _ => Err(Error::unexpected_token(peek2, "an integer or a float")),
                }
            }

            (TokenKind::Ident, value) => {
                let peek2 = self.peek2()?;
                match (peek2.kind, peek2.value) {
                    (TokenKind::Punct, "{") => self.deserialize_struct_dyn(value, visitor),
                    (TokenKind::Punct, "(") => self.deserialize_tuple_struct_dyn(value, 0, visitor),
                    _ if matches!(value, "true" | "false") => self.deserialize_bool(visitor),
                    _ => self.deserialize_unit_struct_dyn(value, visitor),
                }
            }

            (TokenKind::Punct, "(") => self.deserialize_tuple(0, visitor),
            // TODO: This could also be a set.
            (TokenKind::Punct, "{") => self.deserialize_map(visitor),
            (TokenKind::Punct, "[") => self.deserialize_seq(visitor),

            _ => todo!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.parse_ident()? {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            ident => Err(Error::unexpected_token(
                Token {
                    kind: TokenKind::Ident,
                    value: ident,
                },
                "a boolean",
            )),
        }
    }

    deserialize_signed!(deserialize_i8, i8, visit_i8);
    deserialize_signed!(deserialize_i16, i16, visit_i16);
    deserialize_signed!(deserialize_i32, i32, visit_i32);
    deserialize_signed!(deserialize_i64, i64, visit_i64);
    deserialize_signed!(deserialize_i128, i128, visit_i128);

    deserialize_unsigned!(deserialize_u8, u8, visit_u8);
    deserialize_unsigned!(deserialize_u16, u16, visit_u16);
    deserialize_unsigned!(deserialize_u32, u32, visit_u32);
    deserialize_unsigned!(deserialize_u64, u64, visit_u64);
    deserialize_unsigned!(deserialize_u128, u128, visit_u128);

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let float = self.parse_float()?;
        let value = match float.kind {
            TokenKind::Ident => f32::NAN,
            TokenKind::Float => float
                .value
                .parse()
                .map_err(|e| Error::parse_float(float.span, e))?,
            _ => unreachable!(),
        };

        let value = match float.sign {
            Sign::Positive => value,
            Sign::Negative => -value,
        };

        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let float = self.parse_float()?;
        let value = match float.kind {
            TokenKind::Ident => f64::NAN,
            TokenKind::Float => float
                .value
                .parse()
                .map_err(|e| Error::parse_float(float.span, e))?,
            _ => unreachable!(),
        };

        let value = match float.sign {
            Sign::Positive => value,
            Sign::Negative => -value,
        };

        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let char = self.parse_char()?;
        let mut iter = char.value.chars();

        let value = match iter.next() {
            Some(value) => value,
            None => {
                return Err(Error::invalid_string_literal(
                    char.span,
                    "character literal was empty",
                ))
            }
        };

        if iter.next().is_some() {
            return Err(Error::invalid_string_literal(
                char.span,
                "character literal contained multiple characters",
            ));
        }

        visitor.visit_char(value)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let str = self.parse_string()?;
        match str.value {
            Cow::Owned(value) => visitor.visit_string(value),
            Cow::Borrowed(value) => visitor.visit_borrowed_str(value),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let ident = self.parse_ident()?;

        match ident {
            "Some" => {
                self.parse_punct('{')?;
                let value = visitor.visit_some(&mut *self)?;
                self.parse_punct('}')?;
                Ok(value)
            }
            "None" => visitor.visit_none(),
            ident => Err(Error::unknown_variant(ident, &["Some", "None"])),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_punct('(')?;
        self.parse_punct(')')?;

        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit_struct_dyn(name, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_ident_exact(name)?;
        self.parse_punct('(')?;
        let value = visitor.visit_newtype_struct(&mut *self)?;
        self.parse_punct(')')?;

        Ok(value)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value;

        // Both DebugList and DebugSet correspond to a serde sequence.
        match self.parse_punct_ex("`[` or `{`", |v| matches!(v, "[" | "{"))? {
            "[" => {
                value = visitor.visit_seq(DebugSeqAccess(self))?;
                self.parse_punct(']')?;
            }
            "{" => {
                value = visitor.visit_seq(DebugSeqAccess(self))?;
                self.parse_punct('}')?;
            }
            _ => unreachable!(),
        }

        Ok(value)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_punct('(')?;
        let value = visitor.visit_seq(DebugTupleAccess(&mut *self))?;
        self.parse_punct(')')?;
        Ok(value)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple_struct_dyn(name, len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_punct('{')?;
        let value = visitor.visit_map(DebugMapAccess(&mut *self))?;
        self.parse_punct('}')?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_struct_dyn(name, visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(DebugEnumAccess(&mut *self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let ident = self.parse_ident()?;
        visitor.visit_borrowed_str(ident)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct DebugSeqAccess<'a, 'de>(&'a mut Deserializer<'de>);

impl<'de> SeqAccess<'de> for DebugSeqAccess<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Token {
            kind: TokenKind::Punct,
            value: "]" | "}",
        } = self.0.peek()?
        {
            return Ok(None);
        }

        let value = seed.deserialize(&mut *self.0)?;
        match self.0.peek()? {
            // Trailing commas are permitted to be missing only if there is a closing brace there
            // instead.
            Token {
                kind: TokenKind::Punct,
                value: "]" | "}",
            } => (),
            _ => self.0.parse_punct(',')?,
        }

        Ok(Some(value))
    }
}

struct DebugTupleAccess<'a, 'de>(&'a mut Deserializer<'de>);

impl<'de> SeqAccess<'de> for DebugTupleAccess<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Token {
            kind: TokenKind::Punct,
            value: ")",
        } = self.0.peek()?
        {
            return Ok(None);
        }

        let value = seed.deserialize(&mut *self.0)?;
        match self.0.peek()? {
            // Trailing commas are permitted to be missing only if there is a closing brace there
            // instead.
            Token {
                kind: TokenKind::Punct,
                value: ")",
            } => (),
            _ => self.0.parse_punct(',')?,
        }

        Ok(Some(value))
    }
}

struct DebugMapAccess<'a, 'de>(&'a mut Deserializer<'de>);

impl<'de> MapAccess<'de> for DebugMapAccess<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.0.peek()?.is_punct("}") {
            return Ok(None);
        }

        seed.deserialize(&mut *self.0).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.0.parse_punct(':')?;
        let value = seed.deserialize(&mut *self.0)?;

        match self.0.peek()? {
            Token {
                kind: TokenKind::Punct,
                value: "}",
            } => (),
            _ => self.0.parse_punct(',')?,
        }

        Ok(value)
    }
}

struct DebugStructAccess<'a, 'de>(&'a mut Deserializer<'de>);

impl<'de> MapAccess<'de> for DebugStructAccess<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let token = self.0.peek()?;
        match (token.kind, token.value) {
            (TokenKind::Punct, "}") => return Ok(None),
            // This marks the end of a non-exhaustive struct. Example:
            // Test { a: 4, .. }
            (TokenKind::Punct, "..") => {
                self.0.parse_punct_ex("..", |v| v == "..")?;
                return Ok(None);
            }
            _ => (),
        }

        let ident = self.0.parse_ident()?;
        seed.deserialize(BorrowedStrDeserializer::new(ident))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.0.parse_punct(':')?;
        let value = seed.deserialize(&mut *self.0)?;

        match self.0.peek()? {
            Token {
                kind: TokenKind::Punct,
                value: "}",
            } => (),
            _ => self.0.parse_punct(',')?,
        }

        Ok(value)
    }
}

struct DebugEnumAccess<'a, 'de>(&'a mut Deserializer<'de>);

impl<'de> EnumAccess<'de> for DebugEnumAccess<'_, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let ident = self.0.parse_ident()?;
        let value = seed.deserialize(BorrowedStrDeserializer::<Error>::new(ident))?;

        Ok((value, self))
    }
}

impl<'de> VariantAccess<'de> for DebugEnumAccess<'_, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.0.parse_punct('(')?;
        let value = seed.deserialize(&mut *self.0)?;
        self.0.parse_punct(')')?;
        Ok(value)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.parse_punct('{')?;
        let value = visitor.visit_map(DebugStructAccess(&mut *self.0))?;
        self.0.parse_punct('}')?;
        Ok(value)
    }
}

fn unescape(mut text: &str) -> Result<Cow<'_, str>, Error> {
    let mut next = match text.find('\\') {
        Some(pos) => pos,
        None => return Ok(Cow::Borrowed(text)),
    };

    let mut escaped = String::new();

    loop {
        let (head, rest) = text.split_at(next);
        escaped.push_str(head);
        text = rest;

        let (c, len) = match &text[..2] {
            r"\0" => ('\0', 2),
            r"\t" => ('\t', 2),
            r"\r" => ('\r', 2),
            r"\n" => ('\n', 2),
            r"\\" => ('\\', 2),
            r"\'" => ('\'', 2),
            "\\\"" => ('"', 2),
            r"\u" => {
                let rest = &text[2..]
                    .strip_prefix('{')
                    .ok_or_else(|| Error::invalid_string_literal(text, "invalid unicode escape"))?;

                let offset = rest
                    .chars()
                    .position(|x| !x.is_ascii_hexdigit())
                    .unwrap_or(rest.len());
                let (digits, rest) = rest.split_at(offset);

                rest.strip_prefix('}')
                    .ok_or_else(|| Error::invalid_string_literal(text, "invalid unicode escape"))?;

                let escape = &text[..digits.len() + 4];
                let code = u32::from_str_radix(digits, 16)
                    .map_err(|_| Error::invalid_string_literal(escape, "invalid unicode escape"))?;
                let c = match char::from_u32(code) {
                    Some(c) => c,
                    None => {
                        return Err(Error::invalid_string_literal(
                            text,
                            "unicode escape was not a valid unicode codepoint",
                        ))
                    }
                };

                (c, digits.len() + 4)
            }
            escape => {
                return Err(Error::invalid_string_literal(
                    text,
                    format!("invalid escape sequence '{escape}'"),
                ))
            }
        };

        escaped.push(c);
        text = text.split_at(len).1;

        next = match text.find('\\') {
            Some(pos) => pos,
            None => break,
        };
    }

    escaped.push_str(text);
    Ok(escaped.into())
}

#[cfg(test)]
mod unescape_tests {
    use super::*;

    macro_rules! unescape_test {
        {
            $(
                $( #[$attr:meta] )*
                $test:ident: $input:expr $( => $output:expr )? ;
            )*
        } => {$(
            #[test]
            $( #[$attr] )*
            #[allow(unused_variables)]
            fn $test() {
                let escaped = unescape($input)
                    .expect("escape sequence was invalid");

                $(
                    let expected: &str = $output;

                    assert_eq!(&*escaped, expected);
                )?
            }
        )*}
    }

    unescape_test! {
        basic: "test" => "test";
        null: r"\0" => "\0";
        tab: r"\t" => "\t";
        crlf: r"\r\n" => "\r\n";
        backslash: r"\\" => "\\";

        single_quote: r"\'" => "\'";
        double_quote: r#"\""# => "\"";

        unicode_null: r"\u{0}" => "\u{0}";
        unicode_max: r"\u{109999}" => "\u{109999}";
        unicode_hex_uppercase: r"\u{ABCDE}" => "\u{ABCDE}";
        unicode_hex_lowercase: r"\u{abcde}" => "\u{abcde}";

        #[should_panic]
        unicode_incomplete: r"\u{123";
        #[should_panic]
        invalid_escape: r"\a";

        mixed: r"One, two, three, four!\nI declare a \tab war!\n\\\u{9123}"
            => "One, two, three, four!\nI declare a \tab war!\n\\\u{9123}";
        empty: "" => "";
    }
}
