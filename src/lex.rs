use std::fmt;

use crate::error::{Expected, LexerError};

#[derive(Copy, Clone, Debug)]
pub(crate) struct Token<'de> {
    pub kind: TokenKind,
    pub value: &'de str,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    /// An alphanumeric identifier token. It must start with a letter but then
    /// can be any series of valid identifier tokens.
    ///
    /// Examples include `TokenKind`, `true`, and `false`.
    Ident,

    /// A punctuation token.
    ///
    /// Examples:
    /// - `:`
    /// - `{` and `}`
    /// - `[` and `]`
    /// - `..`
    /// - `,`
    /// - `-`
    Punct,

    /// Any integer value.
    ///
    /// Examples:
    /// - `0`
    /// - `0x000`
    /// - `0o1234`
    /// - `0b011011`
    Integer,

    /// A floating-point value.
    ///
    /// Examples:
    /// - `0.0`
    /// - `42.0`
    /// - `5e+3`
    /// - `99.1212e-22`
    Float,

    /// A string value, in quotes.
    ///
    /// This will undo unicode escapes done by `escape_debug`.
    ///
    /// Examples
    /// - `"test"`
    /// - `"\u{41} blah"`
    String,

    /// A character value in single quotes.
    Char,

    /// The end-of-file token.
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::String => "a string",
            Self::Char => "a character literal",
            Self::Integer => "an integer",
            Self::Float => "a floating-point number",
            Self::Punct => "a punctuation token",
            Self::Ident => "an identifier",
            Self::Eof => "end-of-file",
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Lexer<'de> {
    data: &'de str,
}

impl<'de> Lexer<'de> {
    pub fn new(data: &'de str) -> Self {
        Self { data }
    }

    fn skip_whitespace(&mut self) {
        self.data = self.data.trim_start();
    }

    fn advance(&mut self, bytes: usize) {
        self.data = self.data.split_at(bytes).1;
    }

    fn take_char(&mut self) -> Option<char> {
        let (index, c) = self.data.char_indices().next()?;
        self.advance(index);
        Some(c)
    }

    fn peek_char(&mut self) -> Option<char> {
        self.data.chars().next()
    }

    fn unexpected_token(&self, expected: impl Into<Expected<'de>>) -> LexerError<'de> {
        LexerError {
            found: first_char(self.data),
            expected: expected.into(),
        }
    }

    fn unexpected_eof(&self, expected: impl Into<Expected<'de>>) -> LexerError<'de> {
        LexerError {
            found: &self.data[self.data.len()..],
            expected: expected.into(),
        }
    }

    fn try_parse<F, T>(&mut self, func: F) -> Result<T, LexerError<'de>>
    where
        F: FnOnce(&mut Self) -> Result<T, LexerError<'de>>,
    {
        let mut copy = self.clone();
        let result = func(&mut copy);

        if result.is_ok() {
            *self = copy;
        }

        result
    }

    fn parse_consumed<F>(&mut self, func: F) -> Result<Token<'de>, LexerError<'de>>
    where
        F: FnOnce(&mut Self) -> Result<TokenKind, LexerError<'de>>,
    {
        let copy = self.data;
        let kind = self.try_parse(func)?;

        let start = copy.as_ptr();
        let end = copy.as_ptr().wrapping_add(copy.len());
        assert!((start..=end).contains(&self.data.as_ptr()));

        let offset = self.data.as_ptr() as usize - start as usize;
        Ok(Token {
            kind,
            value: &copy[..offset],
        })
    }

    pub fn parse_token(&mut self) -> Result<Token<'de>, LexerError<'de>> {
        self.skip_whitespace();
        self.parse_consumed(|this| match this.peek_char() {
            None => Ok(TokenKind::Eof),
            Some('\"') => this.parse_string(),
            Some('\'') => this.parse_char(),
            Some('0'..='9') => this.parse_number(),
            Some(c) if unicode_ident::is_xid_start(c) => this.parse_ident(),
            Some('.') => this.parse_dotdot(),
            Some('{' | '}' | '[' | ']' | ':' | ',' | '(' | ')' | '+' | '-') => {
                this.advance(1);
                Ok(TokenKind::Punct)
            }
            Some(_) => Err(this.unexpected_token("a valid token")),
        })
    }

    fn parse_string(&mut self) -> Result<TokenKind, LexerError<'de>> {
        self.data = match self.data.strip_prefix("\"") {
            Some(rest) => rest,
            None => return Err(self.unexpected_token(TokenKind::String)),
        };

        while let Some(idx) = self.data.find('\"') {
            self.advance(idx);

            if self.data.as_bytes().get(idx - 1) == Some(&b'\\') {
                continue;
            }

            break;
        }

        match self.data.as_bytes().get(0) {
            Some(b'\"') => {
                self.advance(1);
                Ok(TokenKind::String)
            }
            _ => Err(LexerError::unexpected_token(self.data, TokenKind::String)),
        }
    }

    fn parse_char(&mut self) -> Result<TokenKind, LexerError<'de>> {
        self.data = match self.data.strip_prefix("\'") {
            Some(rest) => rest,
            None => return Err(self.unexpected_token(TokenKind::Char)),
        };

        while let Some(idx) = self.data.find('\'') {
            self.advance(idx);

            if self.data.as_bytes().get(idx - 1) == Some(&b'\\') {
                continue;
            }

            break;
        }

        match self.data.as_bytes().get(0) {
            Some(b'\'') => {
                self.advance(1);
                Ok(TokenKind::Char)
            }
            _ => Err(LexerError::unexpected_token(self.data, TokenKind::Char)),
        }
    }

    fn parse_ident(&mut self) -> Result<TokenKind, LexerError<'de>> {
        match self.data.chars().next() {
            Some(c) if unicode_ident::is_xid_start(c) => (),
            Some(_) => return Err(self.unexpected_token(TokenKind::Ident)),
            None => return Err(self.unexpected_eof(TokenKind::Ident)),
        };

        let index = self
            .data
            .char_indices()
            .skip(1)
            .skip_while(|&(_, c)| unicode_ident::is_xid_continue(c))
            .next()
            .map(|(idx, _)| idx)
            .unwrap_or(self.data.len());

        self.advance(index);

        Ok(TokenKind::Ident)
    }

    fn parse_number(&mut self) -> Result<TokenKind, LexerError<'de>> {
        // This token parsing method is somewhat different from the others
        // because can return two different token types depending on what it
        // parses: integers and floating point numbers.
        //
        // The regexes that match either look like this:
        // - number: ([0-9]|0[xob][0-9A-Fa-f])[0-9A-Fa-f]*
        // - float:  [0-9]+\.[0-9]+([eE](+|-)?[0-9]+)?

        // First off, we need to check for the `0[xob]` prefix.
        match self.take_char() {
            Some('0') => {
                if matches!(self.peek_char(), Some('x' | 'X' | 'o' | 'O' | 'b' | 'B')) {
                    self.advance(1);

                    // We definitely have an integer and just need to parse the
                    // remaining digits in the number.
                    self.parse_once(TokenKind::Integer, |c| c.is_ascii_hexdigit())?;
                    self.parse_repeated(|c| c.is_ascii_hexdigit());
                    return Ok(TokenKind::Integer);
                }
            }
            Some('1'..='9') => (),
            Some(_) => return Err(self.unexpected_token("a number")),
            None => return Err(self.unexpected_eof("a number")),
        }

        self.parse_repeated(|c| c.is_ascii_digit());

        // We've now parsed a sequence of digits. We can still have either a number or a
        // float but if we've parsed a number then we should be done now. The next char
        // decides what it will be.

        match self.peek_char() {
            // We've got a float.
            Some('.' | 'e' | 'E') => (),
            // Anything else means we've got an integer.
            _ => return Ok(TokenKind::Integer),
        }

        // Parse the `\.[0-9]+` part of the float.
        if matches!(self.peek_char(), Some('.')) {
            self.advance(1);
            self.parse_once(TokenKind::Float, |c| c.is_ascii_digit())?;
            self.parse_repeated(|c| c.is_ascii_digit());
        }

        if matches!(self.peek_char(), Some('e' | 'E')) {
            self.advance(1);

            if matches!(self.peek_char(), Some('+' | '-')) {
                self.advance(1);
            }

            self.parse_once(TokenKind::Float, |c| c.is_ascii_digit())?;
            self.parse_repeated(|c| c.is_ascii_digit());
        }

        Ok(TokenKind::Float)
    }

    fn parse_dotdot(&mut self) -> Result<TokenKind, LexerError<'de>> {
        self.parse_once("..", |c| c == '.')?;
        self.parse_once("..", |c| c == '.')?;
        Ok(TokenKind::Punct)
    }

    fn parse_once<F>(
        &mut self,
        expected: impl Into<Expected<'de>>,
        pred: F,
    ) -> Result<(), LexerError<'de>>
    where
        F: FnOnce(char) -> bool,
    {
        match self.data.chars().next() {
            Some(c) if pred(c) => {
                self.advance(c.len_utf8());
                Ok(())
            }
            Some(_) => Err(LexerError::unexpected_token(
                first_char(self.data),
                expected,
            )),
            None => Err(LexerError::unexpected_eof(expected)),
        }
    }

    /// Parses values as long as they match the provided predicate.
    fn parse_repeated<F>(&mut self, mut pred: F)
    where
        F: FnMut(char) -> bool,
    {
        let index = self
            .data
            .char_indices()
            .skip_while(|&(_, c)| pred(c))
            .next()
            .map(|(idx, _)| idx)
            .unwrap_or(self.data.len());

        self.advance(index);
    }
}

fn first_char(s: &str) -> &str {
    match s.chars().next() {
        Some(c) => &s[..c.len_utf8()],
        None => s,
    }
}
