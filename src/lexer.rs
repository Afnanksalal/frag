//! Lexer for Frag source text.
//!
//! The lexer is deliberately simple: it turns characters into positioned tokens
//! and leaves grammar and hardware validity decisions to later stages.

use crate::diagnostic::{Diagnostic, Result, Span};
use std::fmt;

/// Token with its source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    /// Token kind.
    pub kind: TokenKind,
    /// Byte span in the original source.
    pub span: Span,
}

/// All tokens recognized by the Frag lexer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Module,
    Input,
    Output,
    Wire,
    Reg,
    Const,
    On,
    Rising,
    Falling,
    If,
    Else,
    Case,
    Bit,
    BoolType,
    BoolLiteral(bool),
    Identifier(String),
    Number(u128),
    Colon,
    Semicolon,
    Comma,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Equal,
    FatArrow,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    Pipe,
    Caret,
    Tilde,
    Bang,
    AmpAmp,
    PipePipe,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    ShiftLeft,
    ShiftRight,
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Module => write!(f, "module"),
            TokenKind::Input => write!(f, "input"),
            TokenKind::Output => write!(f, "output"),
            TokenKind::Wire => write!(f, "wire"),
            TokenKind::Reg => write!(f, "reg"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::On => write!(f, "on"),
            TokenKind::Rising => write!(f, "rising"),
            TokenKind::Falling => write!(f, "falling"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Case => write!(f, "case"),
            TokenKind::Bit => write!(f, "bit"),
            TokenKind::BoolType => write!(f, "bool"),
            TokenKind::BoolLiteral(value) => write!(f, "{}", value),
            TokenKind::Identifier(name) => write!(f, "identifier `{}`", name),
            TokenKind::Number(value) => write!(f, "number `{}`", value),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Amp => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::AmpAmp => write!(f, "&&"),
            TokenKind::PipePipe => write!(f, "||"),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::BangEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::ShiftLeft => write!(f, "<<"),
            TokenKind::ShiftRight => write!(f, ">>"),
            TokenKind::Eof => write!(f, "end of file"),
        }
    }
}

/// Lex source text into a token stream terminated by [`TokenKind::Eof`].
pub fn lex(source: &str) -> Result<Vec<Token>> {
    Lexer::new(source).tokenize()
}

struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            tokens: Vec::new(),
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>> {
        while let Some(byte) = self.peek() {
            match byte {
                b if b.is_ascii_whitespace() => {
                    self.pos += 1;
                }
                b'/' if self.peek_next() == Some(b'/') => self.skip_line_comment(),
                b'#' => self.skip_line_comment(),
                b'/' if self.peek_next() == Some(b'*') => self.skip_block_comment()?,
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lex_identifier(),
                b'0'..=b'9' => self.lex_number()?,
                b':' => self.single(TokenKind::Colon),
                b';' => self.single(TokenKind::Semicolon),
                b',' => self.single(TokenKind::Comma),
                b'{' => self.single(TokenKind::LeftBrace),
                b'}' => self.single(TokenKind::RightBrace),
                b'(' => self.single(TokenKind::LeftParen),
                b')' => self.single(TokenKind::RightParen),
                b'=' if self.peek_next() == Some(b'=') => self.double(TokenKind::EqualEqual),
                b'=' if self.peek_next() == Some(b'>') => self.double(TokenKind::FatArrow),
                b'!' if self.peek_next() == Some(b'=') => self.double(TokenKind::BangEqual),
                b'<' if self.peek_next() == Some(b'=') => self.double(TokenKind::LessEqual),
                b'>' if self.peek_next() == Some(b'=') => self.double(TokenKind::GreaterEqual),
                b'<' if self.peek_next() == Some(b'<') => self.double(TokenKind::ShiftLeft),
                b'>' if self.peek_next() == Some(b'>') => self.double(TokenKind::ShiftRight),
                b'&' if self.peek_next() == Some(b'&') => self.double(TokenKind::AmpAmp),
                b'|' if self.peek_next() == Some(b'|') => self.double(TokenKind::PipePipe),
                b'=' => self.single(TokenKind::Equal),
                b'+' => self.single(TokenKind::Plus),
                b'-' => self.single(TokenKind::Minus),
                b'*' => self.single(TokenKind::Star),
                b'/' => self.single(TokenKind::Slash),
                b'%' => self.single(TokenKind::Percent),
                b'&' => self.single(TokenKind::Amp),
                b'|' => self.single(TokenKind::Pipe),
                b'^' => self.single(TokenKind::Caret),
                b'~' => self.single(TokenKind::Tilde),
                b'!' => self.single(TokenKind::Bang),
                _ => {
                    let span = Span::new(self.pos, self.pos + 1);
                    return Err(Diagnostic::at(
                        span,
                        format!("Unexpected character `{}`", byte as char),
                    ));
                }
            }
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::new(self.source.len(), self.source.len()),
        });
        Ok(self.tokens)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.bytes.get(self.pos + 1).copied()
    }

    fn single(&mut self, kind: TokenKind) {
        let start = self.pos;
        self.pos += 1;
        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.pos),
        });
    }

    fn double(&mut self, kind: TokenKind) {
        let start = self.pos;
        self.pos += 2;
        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.pos),
        });
    }

    fn skip_line_comment(&mut self) {
        while let Some(byte) = self.peek() {
            self.pos += 1;
            if byte == b'\n' {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) -> Result<()> {
        let start = self.pos;
        self.pos += 2;
        while self.pos + 1 < self.bytes.len() {
            if self.peek() == Some(b'*') && self.peek_next() == Some(b'/') {
                self.pos += 2;
                return Ok(());
            }
            self.pos += 1;
        }

        Err(Diagnostic::at(
            Span::new(start, self.source.len()),
            "Unterminated block comment",
        ))
    }

    fn lex_identifier(&mut self) {
        let start = self.pos;
        self.pos += 1;
        while matches!(
            self.peek(),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
        ) {
            self.pos += 1;
        }

        let text = &self.source[start..self.pos];
        let kind = match text {
            "module" => TokenKind::Module,
            "input" => TokenKind::Input,
            "output" => TokenKind::Output,
            "wire" => TokenKind::Wire,
            "reg" => TokenKind::Reg,
            "const" => TokenKind::Const,
            "on" => TokenKind::On,
            "rising" => TokenKind::Rising,
            "falling" => TokenKind::Falling,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "case" => TokenKind::Case,
            "bit" => TokenKind::Bit,
            "bool" => TokenKind::BoolType,
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),
            _ => TokenKind::Identifier(text.to_string()),
        };

        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.pos),
        });
    }

    fn lex_number(&mut self) -> Result<()> {
        let start = self.pos;
        self.pos += 1;
        while matches!(
            self.peek(),
            Some(b'a'..=b'f' | b'A'..=b'F' | b'x' | b'X' | b'0'..=b'9' | b'_')
        ) {
            self.pos += 1;
        }

        let text = &self.source[start..self.pos];
        let (digits, radix) =
            if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
                (rest, 16)
            } else if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
                (rest, 2)
            } else {
                (text, 10)
            };
        let digits = digits.replace('_', "");

        if digits.is_empty() {
            return Err(Diagnostic::at(
                Span::new(start, self.pos),
                format!("Invalid number literal `{}`", text),
            ));
        }

        let value = u128::from_str_radix(&digits, radix).map_err(|_| {
            Diagnostic::at(
                Span::new(start, self.pos),
                format!("Invalid number literal `{}`", text),
            )
        })?;

        self.tokens.push(Token {
            kind: TokenKind::Number(value),
            span: Span::new(start, self.pos),
        });
        Ok(())
    }
}
