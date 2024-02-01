use std::borrow::Cow;
use std::fmt;

use crate::lex::{Token, TokenKind};

#[derive(Clone, Debug)]
pub(crate) struct LexerError<'de> {
    pub(crate) found: &'de str,
    pub(crate) expected: Expected<'de>,
}

impl<'de> LexerError<'de> {
    pub(crate) fn unexpected_token(found: &'de str, expected: impl Into<Expected<'de>>) -> Self {
        Self {
            found,
            expected: expected.into(),
        }
    }

    pub(crate) fn unexpected_eof(expected: impl Into<Expected<'de>>) -> Self {
        Self {
            found: "",
            expected: expected.into(),
        }
    }
}

impl fmt::Display for LexerError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.found == "" {
            write!(f, "unexpected end of file, expected {}", self.expected)
        } else {
            write!(
                f,
                "unexpected token `{}`, expected {}",
                self.found, self.expected
            )
        }
    }
}

mod detail {
    use std::borrow::Cow;

    use super::*;

    // Needs to be Error so that the debug representation matches the outer name.
    #[derive(Clone, Debug)]
    pub(crate) enum Error<'de> {
        Custom(String),
        Lexer(LexerError<'de>),
        ParseInt {
            value: &'de str,
            error: std::num::ParseIntError,
        },
        ParseFloat {
            value: &'de str,
            error: std::num::ParseFloatError,
        },
        InvalidStringLiteral {
            message: Cow<'static, str>,
        },
    }
}

pub(crate) use self::detail::Error as ErrorDetail;

#[derive(Clone)]
pub struct Error<'de>(ErrorDetail<'de>);

impl<'de> Error<'de> {
    pub(crate) fn parse_int(value: &'de str, error: std::num::ParseIntError) -> Self {
        Self(ErrorDetail::ParseInt { value, error })
    }

    pub(crate) fn parse_float(value: &'de str, error: std::num::ParseFloatError) -> Self {
        Self(ErrorDetail::ParseFloat { value, error })
    }

    pub(crate) fn unexpected_token(token: Token<'de>, expected: impl Into<Expected<'de>>) -> Self {
        Self(ErrorDetail::Lexer(LexerError::unexpected_token(
            token.value,
            expected,
        )))
    }

    pub(crate) fn invalid_string_literal(
        _value: &'de str,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self(ErrorDetail::InvalidStringLiteral {
            message: message.into(),
        })
    }
}

impl<'de> From<LexerError<'de>> for Error<'de> {
    fn from(error: LexerError<'de>) -> Self {
        Self(ErrorDetail::Lexer(error))
    }
}

impl<'de> fmt::Debug for Error<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> fmt::Display for Error<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ErrorDetail::Custom(msg) => f.write_str(msg),
            ErrorDetail::Lexer(err) => err.fmt(f),
            ErrorDetail::ParseInt { value, error } => {
                write!(f, "invalid integer literal `{value}`: {error}")
            }
            ErrorDetail::ParseFloat { value, error } => {
                write!(f, "invalid float literal `{value}`: {error}")
            }
            ErrorDetail::InvalidStringLiteral { message } => {
                write!(f, "invalid string literal: {message}")
            }
        }
    }
}

impl std::error::Error for Error<'_> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            ErrorDetail::ParseInt { error, .. } => Some(error),
            ErrorDetail::ParseFloat { error, .. } => Some(error),
            _ => None,
        }
    }
}

impl serde::de::Error for Error<'_> {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self(ErrorDetail::Custom(msg.to_string()))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum Expected<'de> {
    Token(TokenKind),
    Punct(char),
    Custom(&'de str),
}

impl From<TokenKind> for Expected<'_> {
    fn from(value: TokenKind) -> Self {
        Self::Token(value)
    }
}

impl<'de> From<&'de str> for Expected<'de> {
    fn from(value: &'de str) -> Self {
        Self::Custom(value)
    }
}

impl From<char> for Expected<'_> {
    fn from(value: char) -> Self {
        Self::Punct(value)
    }
}

impl fmt::Display for Expected<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Punct(c) => write!(f, "`{c}`"),
            Self::Custom(msg) => f.write_str(msg),
            Self::Token(kind) => kind.fmt(f),
        }
    }
}
