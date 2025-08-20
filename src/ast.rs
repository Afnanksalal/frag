use crate::lexer::Token;

/// Abstract Syntax Tree (AST) nodes for expressions.
#[derive(Clone, Debug)]
pub enum Expr {
    /// Numeric literal: `42`
    Number(i64),
    /// Boolean literal: `true` or `false`
    Bool(bool),
    /// Variable reference: `x`
    Variable(String),
    /// Function call: `foo(a, b, c)`
    FunctionCall { name: String, args: Vec<Expr> },
    /// Binary operation: `a + b`, `x == y`, etc.
    BinaryOp { op: Token, left: Box<Expr>, right: Box<Expr> },
    /// Unary operation: `-x`, `!y`
    UnaryOp { op: Token, expr: Box<Expr> },
}

/// AST nodes for statements.
#[derive(Clone, Debug)]
pub enum Stmt {
    /// Standalone expression as a statement.
    ExprStmt(Expr),
    /// Variable declaration: `let x = 10;`
    LetDecl { name: String, value: Expr },
}

/// Represents the entire program as a sequence of statements.
#[derive(Clone, Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
