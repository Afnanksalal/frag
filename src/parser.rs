use crate::ast::{Expr, Program, Stmt};
use crate::lexer::Token;
use std::fmt;
use std::iter::Peekable;

use super::lexer::Lexer;

/// Parser errors.
#[derive(Debug, Clone)]
pub enum CompilerError {
    UnexpectedToken(String),
    ExpectedToken(String, String),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::UnexpectedToken(token) => {
                write!(f, "Unexpected token: {}", token)
            }
            CompilerError::ExpectedToken(expected, found) => {
                write!(f, "Expected {}, but found {}", expected, found)
            }
        }
    }
}

type Result<T> = std::result::Result<T, CompilerError>;

/// Parser for constructing AST from tokens.
pub struct Parser<'a> {
    tokens: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser from a lexer.
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            tokens: lexer.peekable(),
        }
    }

    fn peek(&mut self) -> &Token {
        self.tokens.peek().unwrap_or(&Token::Eof)
    }

    fn bump(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn consume(&mut self, expected: Token) -> Result<()> {
        if let Some(t) = self.bump() {
            if t == expected {
                Ok(())
            } else {
                Err(CompilerError::ExpectedToken(
                    format!("{:?}", expected),
                        format!("{:?}", t),
                ))
            }
        } else {
            Err(CompilerError::ExpectedToken(
                format!("{:?}", expected),
                    "End of file".to_string(),
            ))
        }
    }

    /// Parses the entire program.
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut stmts = Vec::new();
        while self.peek() != &Token::Eof {
            let stmt = self.parse_stmt_no_semi()?;
            stmts.push(stmt);
            if self.peek() != &Token::Eof {
                self.consume(Token::Semicolon)?;
            }
        }
        Ok(Program { stmts })
    }

    fn parse_stmt_no_semi(&mut self) -> Result<Stmt> {
        if matches!(self.peek(), Token::Let) {
            self.parse_let_decl_no_semi()
        } else {
            self.parse_expr().map(Stmt::ExprStmt)
        }
    }

    fn parse_let_decl_no_semi(&mut self) -> Result<Stmt> {
        self.consume(Token::Let)?;

        let name = match self.bump() {
            Some(Token::Identifier(name)) => name,
            t => {
                return Err(CompilerError::ExpectedToken(
                    "Identifier".to_string(),
                                                        format!("{:?}", t.unwrap_or(Token::Eof)),
                ));
            }
        };

        self.consume(Token::Equal)?;
        let value = self.parse_expr()?;

        Ok(Stmt::LetDecl { name, value })
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logical_and()?;
        while matches!(self.peek(), Token::OrOr) {
            let op = self.bump().unwrap();
            let rhs = self.parse_logical_and()?;
            expr = Expr::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;
        while matches!(self.peek(), Token::AndAnd) {
            let op = self.bump().unwrap();
            let rhs = self.parse_equality()?;
            expr = Expr::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;
        while matches!(self.peek(), Token::EqualEqual | Token::NotEqual) {
            let op = self.bump().unwrap();
            let rhs = self.parse_comparison()?;
            expr = Expr::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_term()?;
        while matches!(
            self.peek(),
                       Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual
        ) {
            let op = self.bump().unwrap();
            let rhs = self.parse_term()?;
            expr = Expr::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut expr = self.parse_factor()?;
        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = self.bump().unwrap();
            let rhs = self.parse_factor()?;
            expr = Expr::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;
        while matches!(self.peek(), Token::Star | Token::Slash | Token::Percent) {
            let op = self.bump().unwrap();
            let rhs = self.parse_unary()?;
            expr = Expr::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if matches!(self.peek(), Token::Minus | Token::Not) {
            let op = self.bump().unwrap();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op,
                expr: Box::new(expr),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.bump() {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::Bool(b)) => Ok(Expr::Bool(b)),
            Some(Token::Identifier(name)) => {
                if matches!(self.peek(), Token::LeftParen) {
                    self.consume(Token::LeftParen)?;
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RightParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if !matches!(self.peek(), Token::Comma) {
                                break;
                            }
                            self.consume(Token::Comma)?;
                        }
                    }
                    self.consume(Token::RightParen)?;
                    Ok(Expr::FunctionCall { name, args })
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Some(Token::LeftParen) => {
                let expr = self.parse_expr()?;
                self.consume(Token::RightParen)?;
                Ok(expr)
            }
            t => Err(CompilerError::UnexpectedToken(format!("{:?}", t))),
        }
    }
}
