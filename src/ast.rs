//! Source-level abstract syntax tree.
//!
//! AST nodes preserve the structure of a Frag source module. They are produced
//! by the parser and then validated by semantic analysis before lowering to IR.

use crate::diagnostic::Span;
use std::fmt;

/// A single Frag module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Module {
    /// Module name used in generated Verilog.
    pub name: String,
    /// Port, wire, register, and constant declarations.
    pub declarations: Vec<Declaration>,
    /// Top-level combinational assignments.
    pub assignments: Vec<Assignment>,
    /// Clocked sequential processes.
    pub processes: Vec<Process>,
    /// Source span covering the module.
    pub span: Span,
}

/// A signal or constant declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    /// Declaration kind.
    pub kind: DeclKind,
    /// Declared name.
    pub name: String,
    /// Declared bit-vector type.
    pub ty: Type,
    /// Constant initializer. Present only for `const` declarations.
    pub value: Option<Expr>,
    /// Source span covering the declaration.
    pub span: Span,
}

/// Kinds of declarations supported by Frag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclKind {
    /// Module input port.
    Input,
    /// Module output port.
    Output,
    /// Internal combinational signal.
    Wire,
    /// Sequential state signal.
    Reg,
    /// Compile-time constant.
    Const,
}

/// Unsigned bit-vector type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Type {
    /// Number of bits in the value.
    pub width: u32,
}

impl Type {
    /// Return the one-bit type.
    pub fn bit() -> Self {
        Self { width: 1 }
    }
}

/// Assignment to a signal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assignment {
    /// Target signal name.
    pub target: String,
    /// Assigned expression.
    pub expr: Expr,
    /// Source span covering the assignment.
    pub span: Span,
}

/// Clocked sequential process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Process {
    /// Triggering clock edge.
    pub edge: Edge,
    /// Clock signal name.
    pub clock: String,
    /// Register assignments executed on the clock edge.
    pub assignments: Vec<Assignment>,
    /// Source span covering the process.
    pub span: Span,
}

/// Supported clock edges.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Edge {
    /// Positive clock edge.
    Rising,
    /// Negative clock edge.
    Falling,
}

/// Expression tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    /// Unsigned integer literal.
    Number {
        /// Literal value.
        value: u128,
        /// Source span covering the literal.
        span: Span,
    },
    /// Boolean literal represented as a one-bit value.
    Bool {
        /// Literal value.
        value: bool,
        /// Source span covering the literal.
        span: Span,
    },
    /// Reference to a declared signal or constant.
    Signal {
        /// Referenced name.
        name: String,
        /// Source span covering the reference.
        span: Span,
    },
    /// Unary expression.
    Unary {
        /// Operator.
        op: UnaryOp,
        /// Operand.
        expr: Box<Expr>,
        /// Source span covering the expression.
        span: Span,
    },
    /// Binary expression.
    Binary {
        /// Operator.
        op: BinaryOp,
        /// Left operand.
        left: Box<Expr>,
        /// Right operand.
        right: Box<Expr>,
        /// Source span covering the expression.
        span: Span,
    },
}

impl Expr {
    /// Return the source span for this expression.
    pub fn span(&self) -> Span {
        match self {
            Expr::Number { span, .. }
            | Expr::Bool { span, .. }
            | Expr::Signal { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. } => *span,
        }
    }
}

/// Unary operators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    /// Logical not (`!`).
    LogicNot,
    /// Bitwise not (`~`).
    BitNot,
    /// Arithmetic negation (`-`).
    Neg,
}

/// Binary operators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
    /// Multiplication.
    Mul,
    /// Division.
    Div,
    /// Remainder.
    Mod,
    /// Shift left.
    Shl,
    /// Shift right.
    Shr,
    /// Less-than comparison.
    Lt,
    /// Less-than-or-equal comparison.
    Le,
    /// Greater-than comparison.
    Gt,
    /// Greater-than-or-equal comparison.
    Ge,
    /// Equality comparison.
    Eq,
    /// Inequality comparison.
    Ne,
    /// Bitwise and.
    BitAnd,
    /// Bitwise xor.
    BitXor,
    /// Bitwise or.
    BitOr,
    /// Logical and.
    LogicAnd,
    /// Logical or.
    LogicOr,
}

impl fmt::Display for DeclKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeclKind::Input => write!(f, "input"),
            DeclKind::Output => write!(f, "output"),
            DeclKind::Wire => write!(f, "wire"),
            DeclKind::Reg => write!(f, "reg"),
            DeclKind::Const => write!(f, "const"),
        }
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Edge::Rising => write!(f, "rising"),
            Edge::Falling => write!(f, "falling"),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.width == 1 {
            write!(f, "bit")
        } else {
            write!(f, "u{}", self.width)
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::LogicNot => write!(f, "!"),
            UnaryOp::BitNot => write!(f, "~"),
            UnaryOp::Neg => write!(f, "-"),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Shl => write!(f, "<<"),
            BinaryOp::Shr => write!(f, ">>"),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Le => write!(f, "<="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Ge => write!(f, ">="),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Ne => write!(f, "!="),
            BinaryOp::BitAnd => write!(f, "&"),
            BinaryOp::BitXor => write!(f, "^"),
            BinaryOp::BitOr => write!(f, "|"),
            BinaryOp::LogicAnd => write!(f, "&&"),
            BinaryOp::LogicOr => write!(f, "||"),
        }
    }
}
