//! Verilog backend.
//!
//! This backend emits simple synthesizable-style Verilog from the checked IR.

use crate::ast::{BinaryOp, Edge, Expr, UnaryOp};
use crate::ir::{IrModule, IrSignalKind};

/// Emit Verilog for an IR module.
pub fn emit(module: &IrModule) -> String {
    let mut out = String::new();
    let ports: Vec<_> = module
        .signals
        .iter()
        .filter(|signal| matches!(signal.kind, IrSignalKind::Input | IrSignalKind::Output))
        .collect();

    if ports.is_empty() {
        out.push_str(&format!("module {};\n", module.name));
    } else {
        out.push_str(&format!("module {}(\n", module.name));
        for (idx, port) in ports.iter().enumerate() {
            let comma = if idx + 1 == ports.len() { "" } else { "," };
            out.push_str(&format!(
                "    {}{} {}{}\n",
                port_direction(port.kind),
                range(port.width),
                port.name,
                comma
            ));
        }
        out.push_str(");\n");
    }

    if !module.constants.is_empty() || has_internal_signals(module) {
        out.push('\n');
    }

    for constant in &module.constants {
        out.push_str(&format!(
            "localparam{} {} = {};\n",
            range(constant.width),
            constant.name,
            expr(&constant.expr)
        ));
    }

    for signal in &module.signals {
        match signal.kind {
            IrSignalKind::Wire => {
                out.push_str(&format!("wire{} {};\n", range(signal.width), signal.name));
            }
            IrSignalKind::Reg => {
                out.push_str(&format!("reg{} {};\n", range(signal.width), signal.name));
            }
            IrSignalKind::Input | IrSignalKind::Output => {}
        }
    }

    if !module.combinational.is_empty() {
        out.push('\n');
        for assignment in &module.combinational {
            out.push_str(&format!(
                "assign {} = {};\n",
                assignment.target,
                expr(&assignment.expr)
            ));
        }
    }

    for process in &module.processes {
        out.push('\n');
        out.push_str(&format!(
            "always @({} {}) begin\n",
            edge(process.edge),
            process.clock
        ));
        for assignment in &process.assignments {
            out.push_str(&format!(
                "    {} <= {};\n",
                assignment.target,
                expr(&assignment.expr)
            ));
        }
        out.push_str("end\n");
    }

    out.push_str("\nendmodule\n");
    out
}

fn has_internal_signals(module: &IrModule) -> bool {
    module
        .signals
        .iter()
        .any(|signal| matches!(signal.kind, IrSignalKind::Wire | IrSignalKind::Reg))
}

fn port_direction(kind: IrSignalKind) -> &'static str {
    match kind {
        IrSignalKind::Input => "input",
        IrSignalKind::Output => "output",
        IrSignalKind::Wire | IrSignalKind::Reg => unreachable!("internal signals are not ports"),
    }
}

fn range(width: u32) -> String {
    if width == 1 {
        String::new()
    } else {
        format!(" [{}:0]", width - 1)
    }
}

fn edge(edge: Edge) -> &'static str {
    match edge {
        Edge::Rising => "posedge",
        Edge::Falling => "negedge",
    }
}

/// Emit a Verilog expression.
pub fn expr(expr: &Expr) -> String {
    match expr {
        Expr::Number { value, .. } => value.to_string(),
        Expr::Bool { value, .. } => {
            if *value {
                "1'b1".to_string()
            } else {
                "1'b0".to_string()
            }
        }
        Expr::Signal { name, .. } => name.clone(),
        Expr::Unary { op, expr, .. } => format!("({}{})", unary(*op), self::expr(expr)),
        Expr::Binary {
            op, left, right, ..
        } => format!(
            "({} {} {})",
            self::expr(left),
            binary(*op),
            self::expr(right)
        ),
        Expr::Conditional {
            condition,
            then_expr,
            else_expr,
            ..
        } => format!(
            "({} ? {} : {})",
            self::expr(condition),
            self::expr(then_expr),
            self::expr(else_expr)
        ),
    }
}

fn unary(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::LogicNot => "!",
        UnaryOp::BitNot => "~",
        UnaryOp::Neg => "-",
    }
}

fn binary(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Shl => "<<",
        BinaryOp::Shr => ">>",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitXor => "^",
        BinaryOp::BitOr => "|",
        BinaryOp::LogicAnd => "&&",
        BinaryOp::LogicOr => "||",
    }
}
