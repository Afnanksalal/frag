use std::iter::Peekable;
use std::str::Chars;

/// Tokens for the language lexer.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(i64),
    Bool(bool),
    Identifier(String),
    Let,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    AndAnd,
    OrOr,
    Not,
    Equal,
    LeftParen,
    RightParen,
    Comma,
    Semicolon,
    Eof,
}

/// Lexer for tokenizing source code.
pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer from source code.
    pub fn new(src: &'a str) -> Self {
        Self {
            source: src.chars().peekable(),
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.source.peek() {
                Some(c) if c.is_whitespace() => {
                    self.source.next();
                }
                Some(&'/') => {
                    let mut temp = self.source.clone();
                    temp.next(); // Consume '/' in temp
                    if matches!(temp.peek(), Some(&'/')) {
                        // Confirmed '//', consume from self and skip line
                        self.source.next(); // '/'
                        self.source.next(); // '/'
                        while let Some(ch) = self.source.next() {
                            if ch == '\n' {
                                break;
                            }
                        }
                    } else {
                        // Single '/', do not consume, break to handle as token
                        break;
                    }
                }
                Some(&'#') => {
                    self.source.next(); // '#'
                    while let Some(ch) = self.source.next() {
                        if ch == '\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace_and_comments();

        match self.source.next() {
            Some('+') => Some(Token::Plus),
            Some('-') => Some(Token::Minus),
            Some('*') => Some(Token::Star),
            Some('/') => Some(Token::Slash),
            Some('%') => Some(Token::Percent),
            Some('=') => {
                if matches!(self.source.peek(), Some(&'=')) {
                    self.source.next();
                    Some(Token::EqualEqual)
                } else {
                    Some(Token::Equal)
                }
            }
            Some('!') => {
                if matches!(self.source.peek(), Some(&'=')) {
                    self.source.next();
                    Some(Token::NotEqual)
                } else {
                    Some(Token::Not)
                }
            }
            Some('<') => {
                if matches!(self.source.peek(), Some(&'=')) {
                    self.source.next();
                    Some(Token::LessEqual)
                } else {
                    Some(Token::Less)
                }
            }
            Some('>') => {
                if matches!(self.source.peek(), Some(&'=')) {
                    self.source.next();
                    Some(Token::GreaterEqual)
                } else {
                    Some(Token::Greater)
                }
            }
            Some('&') => {
                if matches!(self.source.peek(), Some('&')) {
                    self.source.next();
                    Some(Token::AndAnd)
                } else {
                    // Skip unknown and continue
                    self.next()
                }
            }
            Some('|') => {
                if matches!(self.source.peek(), Some('|')) {
                    self.source.next();
                    Some(Token::OrOr)
                } else {
                    // Skip unknown and continue
                    self.next()
                }
            }
            Some('(') => Some(Token::LeftParen),
            Some(')') => Some(Token::RightParen),
            Some(',') => Some(Token::Comma),
            Some(';') => Some(Token::Semicolon),
            Some(c) if c.is_ascii_digit() => {
                let mut s = c.to_string();
                while let Some(&d) = self.source.peek() {
                    if d.is_ascii_digit() {
                        s.push(d);
                        self.source.next();
                    } else {
                        break;
                    }
                }
                Some(Token::Number(s.parse().expect("Invalid number")))
            }
            Some(c) if c.is_alphabetic() || c == '_' => {
                let mut s = c.to_string();
                while let Some(&d) = self.source.peek() {
                    if d.is_alphanumeric() || d == '_' {
                        s.push(d);
                        self.source.next();
                    } else {
                        break;
                    }
                }
                match s.as_str() {
                    "let" => Some(Token::Let),
                    "true" => Some(Token::Bool(true)),
                    "false" => Some(Token::Bool(false)),
                    _ => Some(Token::Identifier(s)),
                }
            }
            Some(_) => {
                // Skip unknown character and continue
                self.source.next();
                self.next()
            }
            None => Some(Token::Eof),
        }
    }
}
