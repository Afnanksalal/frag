//! Semantic analysis for Frag modules.
//!
//! This pass validates that parsed syntax describes legal hardware and computes
//! dependency orders used by IR lowering, simulation, and code generation.

use crate::ast::{Assignment, BinaryOp, DeclKind, Expr, Module, UnaryOp};
use crate::diagnostic::{Diagnostic, Result, Span};
use std::collections::{BTreeMap, HashMap};

/// Semantic information produced for a checked module.
#[derive(Clone, Debug)]
pub struct Analysis {
    /// Symbol table keyed by declaration name.
    pub symbols: BTreeMap<String, Symbol>,
    /// Declaration indices for constants in dependency order.
    pub const_order: Vec<usize>,
    /// Assignment indices for combinational logic in dependency order.
    pub comb_order: Vec<usize>,
}

/// Declared symbol information.
#[derive(Clone, Debug)]
pub struct Symbol {
    /// Symbol kind.
    pub kind: SymbolKind,
    /// Symbol width in bits.
    pub width: u32,
    /// Declaration source span.
    pub span: Span,
}

/// Symbol kinds after semantic classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymbolKind {
    Input,
    Output,
    Wire,
    Reg,
    Const,
}

/// Validate a parsed module and compute semantic metadata.
pub fn analyze(module: &Module) -> Result<Analysis> {
    let mut symbols = BTreeMap::new();

    for decl in &module.declarations {
        if let Some(previous) = symbols.get(&decl.name) {
            let previous: &Symbol = previous;
            return Err(Diagnostic::at(
                decl.span,
                format!(
                    "Duplicate declaration of `{}`; first declared near byte {}",
                    decl.name, previous.span.start
                ),
            ));
        }

        let kind = SymbolKind::from(decl.kind);
        symbols.insert(
            decl.name.clone(),
            Symbol {
                kind,
                width: decl.ty.width,
                span: decl.span,
            },
        );
    }

    for decl in &module.declarations {
        if decl.kind == DeclKind::Const {
            let value = decl
                .value
                .as_ref()
                .expect("const declarations always have values");
            check_expr(value, &symbols)?;
            for reference in signal_refs(value) {
                let symbol = symbols
                    .get(&reference.name)
                    .expect("reference was checked above");
                if symbol.kind != SymbolKind::Const {
                    return Err(Diagnostic::at(
                        reference.span,
                        format!(
                            "Constant `{}` can only depend on constants, but `{}` is a {:?}",
                            decl.name, reference.name, symbol.kind
                        ),
                    ));
                }
            }
            check_width(&decl.name, decl.ty.width, value, &symbols, decl.span)?;
        }
    }

    let mut comb_targets = HashMap::new();
    for assignment in &module.assignments {
        let target = symbol_for_target(&assignment.target, &symbols, assignment.span)?;
        if !matches!(target.kind, SymbolKind::Output | SymbolKind::Wire) {
            return Err(Diagnostic::at(
                assignment.span,
                format!(
                    "Combinational assignment target `{}` must be an output or wire",
                    assignment.target
                ),
            ));
        }

        if let Some(previous) = comb_targets.insert(assignment.target.clone(), assignment.span) {
            return Err(Diagnostic::at(
                assignment.span,
                format!(
                    "Multiple combinational drivers for `{}`; previous assignment starts near byte {}",
                    assignment.target, previous.start
                ),
            ));
        }

        check_expr(&assignment.expr, &symbols)?;
        check_width(
            &assignment.target,
            target.width,
            &assignment.expr,
            &symbols,
            assignment.span,
        )?;
    }

    let mut seq_targets = HashMap::new();
    for process in &module.processes {
        let clock = symbols.get(&process.clock).ok_or_else(|| {
            Diagnostic::at(
                process.span,
                format!("Unknown clock signal `{}`", process.clock),
            )
        })?;
        if clock.kind != SymbolKind::Input || clock.width != 1 {
            return Err(Diagnostic::at(
                process.span,
                format!("Clock `{}` must be a one-bit input signal", process.clock),
            ));
        }

        let mut local_targets = HashMap::new();
        for assignment in &process.assignments {
            let target = symbol_for_target(&assignment.target, &symbols, assignment.span)?;
            if target.kind != SymbolKind::Reg {
                return Err(Diagnostic::at(
                    assignment.span,
                    format!(
                        "Sequential assignment target `{}` must be a register",
                        assignment.target
                    ),
                ));
            }
            if let Some(previous) = local_targets.insert(assignment.target.clone(), assignment.span)
            {
                return Err(Diagnostic::at(
                    assignment.span,
                    format!(
                        "Register `{}` is assigned twice in one process; previous assignment starts near byte {}",
                        assignment.target, previous.start
                    ),
                ));
            }
            if let Some(previous) = seq_targets.insert(assignment.target.clone(), assignment.span) {
                return Err(Diagnostic::at(
                    assignment.span,
                    format!(
                        "Register `{}` is driven by multiple processes; previous assignment starts near byte {}",
                        assignment.target, previous.start
                    ),
                ));
            }

            check_expr(&assignment.expr, &symbols)?;
            check_width(
                &assignment.target,
                target.width,
                &assignment.expr,
                &symbols,
                assignment.span,
            )?;
        }
    }

    let assigned_outputs: Vec<_> = module
        .assignments
        .iter()
        .map(|a| a.target.as_str())
        .collect();
    for decl in &module.declarations {
        if decl.kind == DeclKind::Output && !assigned_outputs.contains(&decl.name.as_str()) {
            return Err(Diagnostic::at(
                decl.span,
                format!("Output `{}` is declared but never assigned", decl.name),
            ));
        }
    }

    let const_order = constant_order(module)?;
    let comb_order = combinational_order(&module.assignments)?;
    Ok(Analysis {
        symbols,
        const_order,
        comb_order,
    })
}

fn symbol_for_target<'a>(
    name: &str,
    symbols: &'a BTreeMap<String, Symbol>,
    span: Span,
) -> Result<&'a Symbol> {
    symbols
        .get(name)
        .ok_or_else(|| Diagnostic::at(span, format!("Unknown assignment target `{}`", name)))
}

fn check_expr(expr: &Expr, symbols: &BTreeMap<String, Symbol>) -> Result<()> {
    match expr {
        Expr::Number { .. } | Expr::Bool { .. } => Ok(()),
        Expr::Signal { name, span } => {
            if symbols.contains_key(name) {
                Ok(())
            } else {
                Err(Diagnostic::at(*span, format!("Unknown signal `{}`", name)))
            }
        }
        Expr::Unary { expr, .. } => check_expr(expr, symbols),
        Expr::Binary { left, right, .. } => {
            check_expr(left, symbols)?;
            check_expr(right, symbols)
        }
    }
}

fn check_width(
    target_name: &str,
    target_width: u32,
    expr: &Expr,
    symbols: &BTreeMap<String, Symbol>,
    span: Span,
) -> Result<()> {
    let expr_width = expr_width(expr, symbols);
    if expr_width == target_width {
        return Ok(());
    }

    if let Some(value) = eval_unsized_const(expr) {
        if min_bits(value) <= target_width {
            return Ok(());
        }
    }

    Err(Diagnostic::at(
        span,
        format!(
            "Width mismatch assigning to `{}`: target is {} bit(s), expression is {} bit(s)",
            target_name, target_width, expr_width
        ),
    ))
}

/// Compute expression width using the checked symbol table.
pub fn expr_width(expr: &Expr, symbols: &BTreeMap<String, Symbol>) -> u32 {
    match expr {
        Expr::Number { value, .. } => min_bits(*value),
        Expr::Bool { .. } => 1,
        Expr::Signal { name, .. } => symbols.get(name).map(|symbol| symbol.width).unwrap_or(1),
        Expr::Unary { op, expr, .. } => match op {
            UnaryOp::LogicNot => 1,
            UnaryOp::BitNot | UnaryOp::Neg => expr_width(expr, symbols),
        },
        Expr::Binary {
            op, left, right, ..
        } => match op {
            BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge
            | BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::LogicAnd
            | BinaryOp::LogicOr => 1,
            BinaryOp::Shl | BinaryOp::Shr => expr_width(left, symbols),
            _ => expr_width(left, symbols).max(expr_width(right, symbols)),
        },
    }
}

/// Return the minimum number of bits required to represent an unsigned value.
pub fn min_bits(value: u128) -> u32 {
    if value == 0 {
        1
    } else {
        128 - value.leading_zeros()
    }
}

fn eval_unsized_const(expr: &Expr) -> Option<u128> {
    match expr {
        Expr::Number { value, .. } => Some(*value),
        Expr::Bool { value, .. } => Some(if *value { 1 } else { 0 }),
        Expr::Signal { .. } => None,
        Expr::Unary { op, expr, .. } => {
            let value = eval_unsized_const(expr)?;
            match op {
                UnaryOp::LogicNot => Some(if value == 0 { 1 } else { 0 }),
                UnaryOp::BitNot | UnaryOp::Neg => None,
            }
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let left = eval_unsized_const(left)?;
            let right = eval_unsized_const(right)?;
            match op {
                BinaryOp::Add => left.checked_add(right),
                BinaryOp::Sub => left.checked_sub(right),
                BinaryOp::Mul => left.checked_mul(right),
                BinaryOp::Div => (right != 0).then_some(left / right),
                BinaryOp::Mod => (right != 0).then_some(left % right),
                BinaryOp::Shl => left.checked_shl(right as u32),
                BinaryOp::Shr => left.checked_shr(right as u32),
                BinaryOp::Lt => Some((left < right) as u128),
                BinaryOp::Le => Some((left <= right) as u128),
                BinaryOp::Gt => Some((left > right) as u128),
                BinaryOp::Ge => Some((left >= right) as u128),
                BinaryOp::Eq => Some((left == right) as u128),
                BinaryOp::Ne => Some((left != right) as u128),
                BinaryOp::BitAnd => Some(left & right),
                BinaryOp::BitXor => Some(left ^ right),
                BinaryOp::BitOr => Some(left | right),
                BinaryOp::LogicAnd => Some(((left != 0) && (right != 0)) as u128),
                BinaryOp::LogicOr => Some(((left != 0) || (right != 0)) as u128),
            }
        }
    }
}

fn constant_order(module: &Module) -> Result<Vec<usize>> {
    let const_decls = module
        .declarations
        .iter()
        .enumerate()
        .filter(|(_, decl)| decl.kind == DeclKind::Const)
        .collect::<Vec<_>>();
    let target_to_idx = const_decls
        .iter()
        .enumerate()
        .map(|(local_idx, (_, decl))| (decl.name.clone(), local_idx))
        .collect::<HashMap<_, _>>();

    let mut deps = vec![Vec::new(); const_decls.len()];
    for (local_idx, (_, decl)) in const_decls.iter().enumerate() {
        let expr = decl
            .value
            .as_ref()
            .expect("const declarations always have values");
        for reference in signal_refs(expr) {
            if let Some(dep_idx) = target_to_idx.get(&reference.name) {
                deps[local_idx].push(*dep_idx);
            }
        }
    }

    let labels = const_decls
        .iter()
        .map(|(_, decl)| decl.name.clone())
        .collect::<Vec<_>>();
    let spans = const_decls
        .iter()
        .map(|(_, decl)| decl.span)
        .collect::<Vec<_>>();
    let local_order = topo_order(&deps, &labels, &spans, "constant dependency")?;

    Ok(local_order
        .into_iter()
        .map(|local_idx| const_decls[local_idx].0)
        .collect())
}

fn combinational_order(assignments: &[Assignment]) -> Result<Vec<usize>> {
    let mut target_to_idx = HashMap::new();
    for (idx, assignment) in assignments.iter().enumerate() {
        target_to_idx.insert(assignment.target.clone(), idx);
    }

    let mut deps = vec![Vec::new(); assignments.len()];
    for (idx, assignment) in assignments.iter().enumerate() {
        for reference in signal_refs(&assignment.expr) {
            if let Some(dep_idx) = target_to_idx.get(&reference.name) {
                deps[idx].push(*dep_idx);
            }
        }
    }

    let labels = assignments
        .iter()
        .map(|assignment| assignment.target.clone())
        .collect::<Vec<_>>();
    let spans = assignments
        .iter()
        .map(|assignment| assignment.span)
        .collect::<Vec<_>>();

    topo_order(&deps, &labels, &spans, "combinational reference")
}

fn visit(
    idx: usize,
    deps: &[Vec<usize>],
    labels: &[String],
    spans: &[Span],
    cycle_name: &str,
    marks: &mut [VisitMark],
    order: &mut Vec<usize>,
) -> Result<()> {
    match marks[idx] {
        VisitMark::Done => return Ok(()),
        VisitMark::Visiting => {
            return Err(Diagnostic::at(
                spans[idx],
                format!("Circular {} involving `{}`", cycle_name, labels[idx]),
            ));
        }
        VisitMark::Fresh => {}
    }

    marks[idx] = VisitMark::Visiting;
    for dep in &deps[idx] {
        visit(*dep, deps, labels, spans, cycle_name, marks, order)?;
    }
    marks[idx] = VisitMark::Done;
    order.push(idx);
    Ok(())
}

fn topo_order(
    deps: &[Vec<usize>],
    labels: &[String],
    spans: &[Span],
    cycle_name: &str,
) -> Result<Vec<usize>> {
    let mut marks = vec![VisitMark::Fresh; deps.len()];
    let mut order = Vec::new();
    for idx in 0..deps.len() {
        visit(idx, deps, labels, spans, cycle_name, &mut marks, &mut order)?;
    }
    Ok(order)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum VisitMark {
    Fresh,
    Visiting,
    Done,
}

#[derive(Clone)]
struct SignalRef {
    name: String,
    span: Span,
}

fn signal_refs(expr: &Expr) -> Vec<SignalRef> {
    let mut refs = Vec::new();
    collect_refs(expr, &mut refs);
    refs
}

fn collect_refs(expr: &Expr, refs: &mut Vec<SignalRef>) {
    match expr {
        Expr::Signal { name, span } => refs.push(SignalRef {
            name: name.clone(),
            span: *span,
        }),
        Expr::Unary { expr, .. } => collect_refs(expr, refs),
        Expr::Binary { left, right, .. } => {
            collect_refs(left, refs);
            collect_refs(right, refs);
        }
        Expr::Number { .. } | Expr::Bool { .. } => {}
    }
}

impl From<DeclKind> for SymbolKind {
    fn from(kind: DeclKind) -> Self {
        match kind {
            DeclKind::Input => SymbolKind::Input,
            DeclKind::Output => SymbolKind::Output,
            DeclKind::Wire => SymbolKind::Wire,
            DeclKind::Reg => SymbolKind::Reg,
            DeclKind::Const => SymbolKind::Const,
        }
    }
}
