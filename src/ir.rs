//! Netlist-style intermediate representation.
//!
//! The IR is intentionally separate from the AST. Backends consume this checked,
//! dependency-ordered representation instead of reinterpreting source syntax.

use crate::ast::{Assignment, DeclKind, Edge, Expr, Module};
use crate::semantic::{Analysis, SymbolKind};
use std::fmt;

/// Lowered module consumed by backends.
#[derive(Clone, Debug)]
pub struct IrModule {
    /// Module name.
    pub name: String,
    /// Non-constant signals.
    pub signals: Vec<IrSignal>,
    /// Constants in dependency order.
    pub constants: Vec<IrConstant>,
    /// Combinational assignments in dependency order.
    pub combinational: Vec<IrAssign>,
    /// Clocked processes.
    pub processes: Vec<IrProcess>,
}

/// Signal in the lowered IR.
#[derive(Clone, Debug)]
pub struct IrSignal {
    /// Signal name.
    pub name: String,
    /// Signal kind.
    pub kind: IrSignalKind,
    /// Signal width in bits.
    pub width: u32,
}

/// Kinds of non-constant IR signals.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrSignalKind {
    /// Module input port.
    Input,
    /// Module output port.
    Output,
    /// Internal combinational signal.
    Wire,
    /// Sequential state signal.
    Reg,
}

/// Compile-time constant in the IR.
#[derive(Clone, Debug)]
pub struct IrConstant {
    /// Constant name.
    pub name: String,
    /// Constant width in bits.
    pub width: u32,
    /// Constant expression.
    pub expr: Expr,
}

/// IR assignment.
#[derive(Clone, Debug)]
pub struct IrAssign {
    /// Assignment target.
    pub target: String,
    /// Assignment expression.
    pub expr: Expr,
}

/// Clocked IR process.
#[derive(Clone, Debug)]
pub struct IrProcess {
    /// Triggering clock edge.
    pub edge: Edge,
    /// Clock signal.
    pub clock: String,
    /// Register assignments in the process.
    pub assignments: Vec<IrAssign>,
}

/// Lower a semantically checked AST module into IR.
pub fn lower(module: &Module, analysis: &Analysis) -> IrModule {
    let mut signals = Vec::new();
    let mut constants = Vec::new();

    for idx in &analysis.const_order {
        let decl = &module.declarations[*idx];
        constants.push(IrConstant {
            name: decl.name.clone(),
            width: decl.ty.width,
            expr: decl
                .value
                .clone()
                .expect("const declarations always have values"),
        });
    }

    for decl in &module.declarations {
        if decl.kind != DeclKind::Const {
            let symbol = analysis
                .symbols
                .get(&decl.name)
                .expect("semantic analysis created a symbol for every declaration");
            signals.push(IrSignal {
                name: decl.name.clone(),
                kind: IrSignalKind::from(symbol.kind),
                width: symbol.width,
            });
        }
    }

    let combinational = analysis
        .comb_order
        .iter()
        .map(|idx| {
            let assignment = &module.assignments[*idx];
            IrAssign {
                target: assignment.target.clone(),
                expr: assignment.expr.clone(),
            }
        })
        .collect();

    let processes = module
        .processes
        .iter()
        .map(|process| IrProcess {
            edge: process.edge,
            clock: process.clock.clone(),
            assignments: process.assignments.iter().map(IrAssign::from).collect(),
        })
        .collect();

    IrModule {
        name: module.name.clone(),
        signals,
        constants,
        combinational,
        processes,
    }
}

impl IrModule {
    /// Find a non-constant signal by name.
    pub fn signal(&self, name: &str) -> Option<&IrSignal> {
        self.signals.iter().find(|signal| signal.name == name)
    }
}

impl From<&Assignment> for IrAssign {
    fn from(assignment: &Assignment) -> Self {
        Self {
            target: assignment.target.clone(),
            expr: assignment.expr.clone(),
        }
    }
}

impl From<SymbolKind> for IrSignalKind {
    fn from(kind: SymbolKind) -> Self {
        match kind {
            SymbolKind::Input => IrSignalKind::Input,
            SymbolKind::Output => IrSignalKind::Output,
            SymbolKind::Wire => IrSignalKind::Wire,
            SymbolKind::Reg => IrSignalKind::Reg,
            SymbolKind::Const => unreachable!("constants are not IR signals"),
        }
    }
}

impl fmt::Display for IrModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Module {}", self.name)?;

        if !self.signals.is_empty() {
            writeln!(f, "Signals")?;
            for signal in &self.signals {
                writeln!(
                    f,
                    "  {} {}: {}",
                    signal.kind,
                    signal.name,
                    width(signal.width)
                )?;
            }
        }

        if !self.constants.is_empty() {
            writeln!(f, "Constants")?;
            for constant in &self.constants {
                writeln!(
                    f,
                    "  {}: {} = {}",
                    constant.name,
                    width(constant.width),
                    expr_inline(&constant.expr)
                )?;
            }
        }

        if !self.combinational.is_empty() {
            writeln!(f, "Combinational")?;
            for assignment in &self.combinational {
                write_assignment(f, assignment, "  ")?;
            }
        }

        if !self.processes.is_empty() {
            writeln!(f, "Sequential")?;
            for process in &self.processes {
                writeln!(f, "  Process {}({})", process.edge, process.clock)?;
                for assignment in &process.assignments {
                    write_assignment(f, assignment, "    ")?;
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for IrSignalKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrSignalKind::Input => write!(f, "input"),
            IrSignalKind::Output => write!(f, "output"),
            IrSignalKind::Wire => write!(f, "wire"),
            IrSignalKind::Reg => write!(f, "reg"),
        }
    }
}

fn write_assignment(
    f: &mut fmt::Formatter<'_>,
    assignment: &IrAssign,
    indent: &str,
) -> fmt::Result {
    match &assignment.expr {
        Expr::Binary {
            op, left, right, ..
        } => {
            writeln!(f, "{}Gate {}", indent, op_name(*op))?;
            writeln!(
                f,
                "{}  Inputs: {}, {}",
                indent,
                expr_inline(left),
                expr_inline(right)
            )?;
            writeln!(f, "{}  Output: {}", indent, assignment.target)
        }
        Expr::Unary { op, expr, .. } => {
            writeln!(f, "{}Gate {}", indent, op_name_unary(*op))?;
            writeln!(f, "{}  Input: {}", indent, expr_inline(expr))?;
            writeln!(f, "{}  Output: {}", indent, assignment.target)
        }
        expr => writeln!(
            f,
            "{}Assign {} = {}",
            indent,
            assignment.target,
            expr_inline(expr)
        ),
    }
}

/// Render an expression as a compact string for human-readable IR output.
pub fn expr_inline(expr: &Expr) -> String {
    match expr {
        Expr::Number { value, .. } => value.to_string(),
        Expr::Bool { value, .. } => {
            if *value {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
        Expr::Signal { name, .. } => name.clone(),
        Expr::Unary { op, expr, .. } => format!("({}{})", op, expr_inline(expr)),
        Expr::Binary {
            op, left, right, ..
        } => format!("({} {} {})", expr_inline(left), op, expr_inline(right)),
    }
}

fn width(width: u32) -> String {
    if width == 1 {
        "bit".to_string()
    } else {
        format!("u{}", width)
    }
}

fn op_name(op: crate::ast::BinaryOp) -> &'static str {
    use crate::ast::BinaryOp::*;
    match op {
        Add => "ADD",
        Sub => "SUB",
        Mul => "MUL",
        Div => "DIV",
        Mod => "MOD",
        Shl => "SHL",
        Shr => "SHR",
        Lt => "LT",
        Le => "LE",
        Gt => "GT",
        Ge => "GE",
        Eq => "EQ",
        Ne => "NE",
        BitAnd => "AND",
        BitXor => "XOR",
        BitOr => "OR",
        LogicAnd => "LOGIC_AND",
        LogicOr => "LOGIC_OR",
    }
}

fn op_name_unary(op: crate::ast::UnaryOp) -> &'static str {
    use crate::ast::UnaryOp::*;
    match op {
        LogicNot => "NOT",
        BitNot => "BIT_NOT",
        Neg => "NEG",
    }
}
