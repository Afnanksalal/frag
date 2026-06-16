//! Netlist-style intermediate representation.
//!
//! The IR is intentionally separate from the AST. Backends consume this checked,
//! dependency-ordered representation instead of reinterpreting source syntax.

use crate::ast::{Assignment, BinaryOp, DeclKind, Edge, Expr, Module, UnaryOp};
use crate::diagnostic::{Diagnostic, Result};
use crate::semantic::{self, Analysis, Symbol, SymbolKind};
use std::collections::BTreeMap;
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrConstant {
    /// Constant name.
    pub name: String,
    /// Constant width in bits.
    pub width: u32,
    /// Lowered constant expression.
    pub expr: IrExpr,
}

/// IR assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrAssign {
    /// Assignment target.
    pub target: String,
    /// Lowered assignment expression.
    pub expr: IrExpr,
}

/// Clocked IR process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrProcess {
    /// Triggering clock edge.
    pub edge: Edge,
    /// Clock signal.
    pub clock: String,
    /// Register assignments in the process.
    pub assignments: Vec<IrAssign>,
}

/// Typed expression node used by IR backends.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrExpr {
    /// Constant integer value.
    Const {
        /// Constant value.
        value: u128,
        /// Expression result width.
        width: u32,
    },
    /// Reference to a signal or constant.
    Signal {
        /// Referenced name.
        name: String,
        /// Referenced value width.
        width: u32,
    },
    /// Unary operation.
    Unary {
        /// Operator.
        op: UnaryOp,
        /// Operand.
        expr: Box<IrExpr>,
        /// Expression result width.
        width: u32,
    },
    /// Binary operation.
    Binary {
        /// Operator.
        op: BinaryOp,
        /// Left operand.
        left: Box<IrExpr>,
        /// Right operand.
        right: Box<IrExpr>,
        /// Expression result width.
        width: u32,
    },
    /// Two-input mux.
    Mux {
        /// Select expression; nonzero selects `when_true`.
        select: Box<IrExpr>,
        /// Value selected when `select` is nonzero.
        when_true: Box<IrExpr>,
        /// Value selected when `select` is zero.
        when_false: Box<IrExpr>,
        /// Expression result width.
        width: u32,
    },
}

impl IrExpr {
    /// Result width in bits.
    pub fn width(&self) -> u32 {
        match self {
            IrExpr::Const { width, .. }
            | IrExpr::Signal { width, .. }
            | IrExpr::Unary { width, .. }
            | IrExpr::Binary { width, .. }
            | IrExpr::Mux { width, .. } => *width,
        }
    }
}

/// Validate structural IR invariants.
///
/// Semantic analysis is responsible for user-facing source diagnostics. This
/// pass checks the backend contract after lowering: all references resolve,
/// expression widths are internally consistent, and assignment targets are
/// legal for their combinational or sequential context.
pub fn validate(module: &IrModule) -> Result<()> {
    let bindings = bindings(module)?;

    for signal in &module.signals {
        validate_width(signal.width, &format!("signal `{}`", signal.name))?;
    }

    for constant in &module.constants {
        validate_width(constant.width, &format!("constant `{}`", constant.name))?;
        validate_expr(&constant.expr, &bindings)?;
        validate_assignment_width("constant", &constant.name, constant.width, &constant.expr)?;
    }

    for assignment in &module.combinational {
        let Some(binding) = bindings.get(&assignment.target) else {
            return Err(Diagnostic::new(format!(
                "IR assignment target `{}` is not declared",
                assignment.target
            )));
        };
        if !matches!(binding.kind, IrBindingKind::Output | IrBindingKind::Wire) {
            return Err(Diagnostic::new(format!(
                "IR combinational target `{}` must be an output or wire",
                assignment.target
            )));
        }
        validate_expr(&assignment.expr, &bindings)?;
        validate_assignment_width(
            "assignment",
            &assignment.target,
            binding.width,
            &assignment.expr,
        )?;
    }

    for process in &module.processes {
        let Some(clock) = bindings.get(&process.clock) else {
            return Err(Diagnostic::new(format!(
                "IR process clock `{}` is not declared",
                process.clock
            )));
        };
        if clock.kind != IrBindingKind::Input || clock.width != 1 {
            return Err(Diagnostic::new(format!(
                "IR process clock `{}` must be a one-bit input",
                process.clock
            )));
        }

        for assignment in &process.assignments {
            let Some(binding) = bindings.get(&assignment.target) else {
                return Err(Diagnostic::new(format!(
                    "IR sequential target `{}` is not declared",
                    assignment.target
                )));
            };
            if binding.kind != IrBindingKind::Reg {
                return Err(Diagnostic::new(format!(
                    "IR sequential target `{}` must be a register",
                    assignment.target
                )));
            }
            validate_expr(&assignment.expr, &bindings)?;
            validate_assignment_width(
                "sequential assignment",
                &assignment.target,
                binding.width,
                &assignment.expr,
            )?;
        }
    }

    Ok(())
}

/// Lower a semantically checked AST module into IR.
pub fn lower(module: &Module, analysis: &Analysis) -> IrModule {
    let mut signals = Vec::new();
    let mut constants = Vec::new();

    for idx in &analysis.const_order {
        let decl = &module.declarations[*idx];
        let value = decl
            .value
            .as_ref()
            .expect("const declarations always have values");
        constants.push(IrConstant {
            name: decl.name.clone(),
            width: decl.ty.width,
            expr: lower_expr(value, &analysis.symbols),
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
        .map(|idx| lower_assignment(&module.assignments[*idx], &analysis.symbols))
        .collect();

    let processes = module
        .processes
        .iter()
        .map(|process| IrProcess {
            edge: process.edge,
            clock: process.clock.clone(),
            assignments: process
                .assignments
                .iter()
                .map(|assignment| lower_assignment(assignment, &analysis.symbols))
                .collect(),
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

fn lower_assignment(assignment: &Assignment, symbols: &BTreeMap<String, Symbol>) -> IrAssign {
    IrAssign {
        target: assignment.target.clone(),
        expr: lower_expr(&assignment.expr, symbols),
    }
}

fn lower_expr(expr: &Expr, symbols: &BTreeMap<String, Symbol>) -> IrExpr {
    let width = semantic::expr_width(expr, symbols);
    match expr {
        Expr::Number { value, .. } => IrExpr::Const {
            value: *value,
            width,
        },
        Expr::Bool { value, .. } => IrExpr::Const {
            value: (*value) as u128,
            width,
        },
        Expr::Signal { name, .. } => IrExpr::Signal {
            name: name.clone(),
            width,
        },
        Expr::Unary { op, expr, .. } => IrExpr::Unary {
            op: *op,
            expr: Box::new(lower_expr(expr, symbols)),
            width,
        },
        Expr::Binary {
            op, left, right, ..
        } => IrExpr::Binary {
            op: *op,
            left: Box::new(lower_expr(left, symbols)),
            right: Box::new(lower_expr(right, symbols)),
            width,
        },
        Expr::Conditional {
            condition,
            then_expr,
            else_expr,
            ..
        } => IrExpr::Mux {
            select: Box::new(lower_expr(condition, symbols)),
            when_true: Box::new(lower_expr(then_expr, symbols)),
            when_false: Box::new(lower_expr(else_expr, symbols)),
            width,
        },
    }
}

impl IrModule {
    /// Find a non-constant signal by name.
    pub fn signal(&self, name: &str) -> Option<&IrSignal> {
        self.signals.iter().find(|signal| signal.name == name)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IrBindingKind {
    Input,
    Output,
    Wire,
    Reg,
    Const,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IrBinding {
    kind: IrBindingKind,
    width: u32,
}

fn bindings(module: &IrModule) -> Result<BTreeMap<String, IrBinding>> {
    let mut bindings = BTreeMap::new();
    for signal in &module.signals {
        insert_binding(
            &mut bindings,
            &signal.name,
            IrBinding {
                kind: IrBindingKind::from(signal.kind),
                width: signal.width,
            },
        )?;
    }
    for constant in &module.constants {
        insert_binding(
            &mut bindings,
            &constant.name,
            IrBinding {
                kind: IrBindingKind::Const,
                width: constant.width,
            },
        )?;
    }
    Ok(bindings)
}

fn insert_binding(
    bindings: &mut BTreeMap<String, IrBinding>,
    name: &str,
    binding: IrBinding,
) -> Result<()> {
    if bindings.insert(name.to_string(), binding).is_some() {
        return Err(Diagnostic::new(format!(
            "IR contains duplicate binding `{}`",
            name
        )));
    }
    Ok(())
}

fn validate_expr(expr: &IrExpr, bindings: &BTreeMap<String, IrBinding>) -> Result<()> {
    validate_width(expr.width(), "expression")?;
    match expr {
        IrExpr::Const { value, width } => {
            if semantic::min_bits(*value) > *width {
                return Err(Diagnostic::new(format!(
                    "IR constant value {} does not fit in {} bit(s)",
                    value, width
                )));
            }
        }
        IrExpr::Signal { name, width } => {
            let Some(binding) = bindings.get(name) else {
                return Err(Diagnostic::new(format!(
                    "IR expression references undeclared `{}`",
                    name
                )));
            };
            if binding.width != *width {
                return Err(Diagnostic::new(format!(
                    "IR reference `{}` has width {}, expected {}",
                    name, width, binding.width
                )));
            }
        }
        IrExpr::Unary { op, expr, width } => {
            validate_expr(expr, bindings)?;
            let expected = match op {
                UnaryOp::LogicNot => 1,
                UnaryOp::BitNot | UnaryOp::Neg => expr.width(),
            };
            validate_expected_width(*width, expected, "unary expression")?;
        }
        IrExpr::Binary {
            op,
            left,
            right,
            width,
        } => {
            validate_expr(left, bindings)?;
            validate_expr(right, bindings)?;
            let expected = match op {
                BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge
                | BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::LogicAnd
                | BinaryOp::LogicOr => 1,
                BinaryOp::Shl | BinaryOp::Shr => left.width(),
                _ => left.width().max(right.width()),
            };
            validate_expected_width(*width, expected, "binary expression")?;
        }
        IrExpr::Mux {
            select,
            when_true,
            when_false,
            width,
        } => {
            validate_expr(select, bindings)?;
            validate_expr(when_true, bindings)?;
            validate_expr(when_false, bindings)?;
            let expected = when_true.width().max(when_false.width());
            validate_expected_width(*width, expected, "mux expression")?;
        }
    }
    Ok(())
}

fn validate_assignment_width(
    context: &str,
    target: &str,
    target_width: u32,
    expr: &IrExpr,
) -> Result<()> {
    if expr.width() > target_width {
        return Err(Diagnostic::new(format!(
            "IR {} `{}` has expression width {} greater than target width {}",
            context,
            target,
            expr.width(),
            target_width
        )));
    }
    Ok(())
}

fn validate_width(width: u32, context: &str) -> Result<()> {
    if width == 0 || width > 128 {
        return Err(Diagnostic::new(format!(
            "IR {} has invalid width {}; expected 1..=128",
            context, width
        )));
    }
    Ok(())
}

fn validate_expected_width(actual: u32, expected: u32, context: &str) -> Result<()> {
    if actual != expected {
        return Err(Diagnostic::new(format!(
            "IR {} has width {}, expected {}",
            context, actual, expected
        )));
    }
    Ok(())
}

impl From<IrSignalKind> for IrBindingKind {
    fn from(kind: IrSignalKind) -> Self {
        match kind {
            IrSignalKind::Input => IrBindingKind::Input,
            IrSignalKind::Output => IrBindingKind::Output,
            IrSignalKind::Wire => IrBindingKind::Wire,
            IrSignalKind::Reg => IrBindingKind::Reg,
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
        IrExpr::Binary {
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
        IrExpr::Unary { op, expr, .. } => {
            writeln!(f, "{}Gate {}", indent, op_name_unary(*op))?;
            writeln!(f, "{}  Input: {}", indent, expr_inline(expr))?;
            writeln!(f, "{}  Output: {}", indent, assignment.target)
        }
        IrExpr::Mux {
            select,
            when_true,
            when_false,
            ..
        } => {
            writeln!(f, "{}Gate MUX", indent)?;
            writeln!(f, "{}  Select: {}", indent, expr_inline(select))?;
            writeln!(f, "{}  When 1: {}", indent, expr_inline(when_true))?;
            writeln!(f, "{}  When 0: {}", indent, expr_inline(when_false))?;
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
pub fn expr_inline(expr: &IrExpr) -> String {
    match expr {
        IrExpr::Const { value, .. } => value.to_string(),
        IrExpr::Signal { name, .. } => name.clone(),
        IrExpr::Unary { op, expr, .. } => format!("({}{})", op, expr_inline(expr)),
        IrExpr::Binary {
            op, left, right, ..
        } => format!("({} {} {})", expr_inline(left), op, expr_inline(right)),
        IrExpr::Mux {
            select,
            when_true,
            when_false,
            ..
        } => format!(
            "(if {} then {} else {})",
            expr_inline(select),
            expr_inline(when_true),
            expr_inline(when_false)
        ),
    }
}

fn width(width: u32) -> String {
    if width == 1 {
        "bit".to_string()
    } else {
        format!("u{}", width)
    }
}

fn op_name(op: BinaryOp) -> &'static str {
    use BinaryOp::*;
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

fn op_name_unary(op: UnaryOp) -> &'static str {
    use UnaryOp::*;
    match op {
        LogicNot => "NOT",
        BitNot => "BIT_NOT",
        Neg => "NEG",
    }
}
