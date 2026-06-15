//! Recursive descent parser for Frag.
//!
//! The parser consumes lexer tokens and produces the source-level AST. It does
//! not perform name resolution or width checks; those belong to semantic
//! analysis.

use crate::ast::{
    Assignment, BinaryOp, DeclKind, Declaration, Edge, Expr, Module, Process, Type, UnaryOp,
};
use crate::diagnostic::{Diagnostic, Result, Span};
use crate::lexer::{lex, Token, TokenKind};
use std::mem::discriminant;

/// Parse one source string into a Frag module AST.
pub fn parse_source(source: &str) -> Result<Module> {
    let tokens = lex(source)?;
    Parser::new(tokens).parse_module()
}

/// Parser state over a token vector.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create a parser from tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse the complete token stream as one module.
    pub fn parse_module(&mut self) -> Result<Module> {
        let start = self
            .expect_simple(TokenKind::Module, "`module`")?
            .span
            .start;
        let name = self.expect_identifier()?;
        self.expect_simple(TokenKind::LeftBrace, "`{`")?;

        let mut declarations = Vec::new();
        let mut assignments = Vec::new();
        let mut processes = Vec::new();

        while !self.at_simple(&TokenKind::RightBrace) && !self.at_simple(&TokenKind::Eof) {
            match &self.peek().kind {
                TokenKind::Input | TokenKind::Output | TokenKind::Wire | TokenKind::Reg => {
                    declarations.push(self.parse_declaration()?);
                }
                TokenKind::Const => declarations.push(self.parse_const_declaration()?),
                TokenKind::On => processes.push(self.parse_process()?),
                TokenKind::Identifier(_) => assignments.push(self.parse_assignment()?),
                token => {
                    return Err(Diagnostic::at(
                        self.peek().span,
                        format!(
                            "Expected declaration, assignment, or process, found {}",
                            token
                        ),
                    ));
                }
            }
        }

        let end = self.expect_simple(TokenKind::RightBrace, "`}`")?.span.end;
        self.expect_simple(TokenKind::Eof, "end of file")?;

        Ok(Module {
            name,
            declarations,
            assignments,
            processes,
            span: Span::new(start, end),
        })
    }

    fn parse_declaration(&mut self) -> Result<Declaration> {
        let first = self.bump();
        let kind = match first.kind {
            TokenKind::Input => DeclKind::Input,
            TokenKind::Output => DeclKind::Output,
            TokenKind::Wire => DeclKind::Wire,
            TokenKind::Reg => DeclKind::Reg,
            _ => unreachable!(),
        };

        let name = self.expect_identifier()?;
        self.expect_simple(TokenKind::Colon, "`:`")?;
        let ty = self.parse_type()?;
        let end = self.expect_simple(TokenKind::Semicolon, "`;`")?.span.end;

        Ok(Declaration {
            kind,
            name,
            ty,
            value: None,
            span: Span::new(first.span.start, end),
        })
    }

    fn parse_const_declaration(&mut self) -> Result<Declaration> {
        let first = self.expect_simple(TokenKind::Const, "`const`")?;
        let name = self.expect_identifier()?;
        self.expect_simple(TokenKind::Colon, "`:`")?;
        let ty = self.parse_type()?;
        self.expect_simple(TokenKind::Equal, "`=`")?;
        let value = self.parse_expr()?;
        let end = self.expect_simple(TokenKind::Semicolon, "`;`")?.span.end;

        Ok(Declaration {
            kind: DeclKind::Const,
            name,
            ty,
            value: Some(value),
            span: Span::new(first.span.start, end),
        })
    }

    fn parse_type(&mut self) -> Result<Type> {
        let token = self.bump();
        match token.kind {
            TokenKind::Bit | TokenKind::BoolType => Ok(Type::bit()),
            TokenKind::Identifier(name) => {
                if let Some(width_text) = name.strip_prefix('u') {
                    let width = width_text.parse::<u32>().map_err(|_| {
                        Diagnostic::at(token.span, format!("Invalid unsigned type `{}`", name))
                    })?;
                    if width == 0 || width > 128 {
                        return Err(Diagnostic::at(
                            token.span,
                            "Integer widths must be between 1 and 128 bits",
                        ));
                    }
                    Ok(Type { width })
                } else {
                    Err(Diagnostic::at(
                        token.span,
                        format!("Unknown type `{}`; expected `bit` or `uN`", name),
                    ))
                }
            }
            other => Err(Diagnostic::at(
                token.span,
                format!("Expected type, found {}", other),
            )),
        }
    }

    fn parse_process(&mut self) -> Result<Process> {
        let start = self.expect_simple(TokenKind::On, "`on`")?.span.start;
        let edge_token = self.bump();
        let edge = match edge_token.kind {
            TokenKind::Rising => Edge::Rising,
            TokenKind::Falling => Edge::Falling,
            other => {
                return Err(Diagnostic::at(
                    edge_token.span,
                    format!("Expected `rising` or `falling`, found {}", other),
                ));
            }
        };

        self.expect_simple(TokenKind::LeftParen, "`(`")?;
        let clock = self.expect_identifier()?;
        self.expect_simple(TokenKind::RightParen, "`)`")?;
        self.expect_simple(TokenKind::LeftBrace, "`{`")?;

        let mut assignments = Vec::new();
        while !self.at_simple(&TokenKind::RightBrace) && !self.at_simple(&TokenKind::Eof) {
            assignments.push(self.parse_assignment()?);
        }
        let end = self.expect_simple(TokenKind::RightBrace, "`}`")?.span.end;

        Ok(Process {
            edge,
            clock,
            assignments,
            span: Span::new(start, end),
        })
    }

    fn parse_assignment(&mut self) -> Result<Assignment> {
        let target_token = self.bump();
        let (target, start) = match target_token.kind {
            TokenKind::Identifier(name) => (name, target_token.span.start),
            _ => unreachable!(),
        };
        self.expect_simple(TokenKind::Equal, "`=`")?;
        let expr = self.parse_expr()?;
        let end = self.expect_simple(TokenKind::Semicolon, "`;`")?.span.end;
        Ok(Assignment {
            target,
            expr,
            span: Span::new(start, end),
        })
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_logic_or()
    }

    fn parse_logic_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logic_and()?;
        while self.match_simple(&TokenKind::PipePipe) {
            let right = self.parse_logic_and()?;
            expr = binary(BinaryOp::LogicOr, expr, right);
        }
        Ok(expr)
    }

    fn parse_logic_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_bit_or()?;
        while self.match_simple(&TokenKind::AmpAmp) {
            let right = self.parse_bit_or()?;
            expr = binary(BinaryOp::LogicAnd, expr, right);
        }
        Ok(expr)
    }

    fn parse_bit_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_bit_xor()?;
        while self.match_simple(&TokenKind::Pipe) {
            let right = self.parse_bit_xor()?;
            expr = binary(BinaryOp::BitOr, expr, right);
        }
        Ok(expr)
    }

    fn parse_bit_xor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_bit_and()?;
        while self.match_simple(&TokenKind::Caret) {
            let right = self.parse_bit_and()?;
            expr = binary(BinaryOp::BitXor, expr, right);
        }
        Ok(expr)
    }

    fn parse_bit_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;
        while self.match_simple(&TokenKind::Amp) {
            let right = self.parse_equality()?;
            expr = binary(BinaryOp::BitAnd, expr, right);
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;
        loop {
            if self.match_simple(&TokenKind::EqualEqual) {
                let right = self.parse_comparison()?;
                expr = binary(BinaryOp::Eq, expr, right);
            } else if self.match_simple(&TokenKind::BangEqual) {
                let right = self.parse_comparison()?;
                expr = binary(BinaryOp::Ne, expr, right);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_shift()?;
        loop {
            let op = if self.match_simple(&TokenKind::Less) {
                Some(BinaryOp::Lt)
            } else if self.match_simple(&TokenKind::LessEqual) {
                Some(BinaryOp::Le)
            } else if self.match_simple(&TokenKind::Greater) {
                Some(BinaryOp::Gt)
            } else if self.match_simple(&TokenKind::GreaterEqual) {
                Some(BinaryOp::Ge)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };
            let right = self.parse_shift()?;
            expr = binary(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<Expr> {
        let mut expr = self.parse_term()?;
        loop {
            let op = if self.match_simple(&TokenKind::ShiftLeft) {
                Some(BinaryOp::Shl)
            } else if self.match_simple(&TokenKind::ShiftRight) {
                Some(BinaryOp::Shr)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };
            let right = self.parse_term()?;
            expr = binary(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut expr = self.parse_factor()?;
        loop {
            let op = if self.match_simple(&TokenKind::Plus) {
                Some(BinaryOp::Add)
            } else if self.match_simple(&TokenKind::Minus) {
                Some(BinaryOp::Sub)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };
            let right = self.parse_factor()?;
            expr = binary(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = if self.match_simple(&TokenKind::Star) {
                Some(BinaryOp::Mul)
            } else if self.match_simple(&TokenKind::Slash) {
                Some(BinaryOp::Div)
            } else if self.match_simple(&TokenKind::Percent) {
                Some(BinaryOp::Mod)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };
            let right = self.parse_unary()?;
            expr = binary(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        let token = self.peek().clone();
        if self.match_simple(&TokenKind::Bang) {
            let expr = self.parse_unary()?;
            return Ok(unary(UnaryOp::LogicNot, token.span.start, expr));
        }
        if self.match_simple(&TokenKind::Tilde) {
            let expr = self.parse_unary()?;
            return Ok(unary(UnaryOp::BitNot, token.span.start, expr));
        }
        if self.match_simple(&TokenKind::Minus) {
            let expr = self.parse_unary()?;
            return Ok(unary(UnaryOp::Neg, token.span.start, expr));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let token = self.bump();
        match token.kind {
            TokenKind::Number(value) => Ok(Expr::Number {
                value,
                span: token.span,
            }),
            TokenKind::BoolLiteral(value) => Ok(Expr::Bool {
                value,
                span: token.span,
            }),
            TokenKind::Identifier(name) => Ok(Expr::Signal {
                name,
                span: token.span,
            }),
            TokenKind::LeftParen => {
                let expr = self.parse_expr()?;
                self.expect_simple(TokenKind::RightParen, "`)`")?;
                Ok(expr)
            }
            other => Err(Diagnostic::at(
                token.span,
                format!("Expected expression, found {}", other),
            )),
        }
    }

    fn expect_identifier(&mut self) -> Result<String> {
        let token = self.bump();
        match token.kind {
            TokenKind::Identifier(name) => Ok(name),
            other => Err(Diagnostic::at(
                token.span,
                format!("Expected identifier, found {}", other),
            )),
        }
    }

    fn expect_simple(&mut self, kind: TokenKind, expected: &str) -> Result<Token> {
        if self.at_simple(&kind) {
            Ok(self.bump())
        } else {
            Err(Diagnostic::at(
                self.peek().span,
                format!("Expected {}, found {}", expected, self.peek().kind),
            ))
        }
    }

    fn match_simple(&mut self, kind: &TokenKind) -> bool {
        if self.at_simple(kind) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn at_simple(&self, kind: &TokenKind) -> bool {
        discriminant(&self.peek().kind) == discriminant(kind)
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .or_else(|| self.tokens.last())
            .expect("parser must receive at least EOF")
    }

    fn bump(&mut self) -> Token {
        let token = self.peek().clone();
        if !self.at_simple(&TokenKind::Eof) {
            self.pos += 1;
        }
        token
    }
}

fn unary(op: UnaryOp, start: usize, expr: Expr) -> Expr {
    let span = Span::new(start, expr.span().end);
    Expr::Unary {
        op,
        expr: Box::new(expr),
        span,
    }
}

fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    let span = left.span().join(right.span());
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
        span,
    }
}
