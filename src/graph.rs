//! Graph backends for circuit visualization.
//!
//! These emitters are intended for documentation and inspection. They render the
//! same IR consumed by the Verilog backend and simulator.

use crate::ast::{BinaryOp, Edge, UnaryOp};
use crate::ir::{IrExpr, IrModule};

/// Emit a Graphviz DOT graph for an IR module.
pub fn emit_dot(module: &IrModule) -> String {
    let mut graph = DotGraph::default();
    graph.line("digraph Frag {");
    graph.line("  rankdir=LR;");
    graph.line("  node [fontname=\"Arial\"];\n");

    for signal in &module.signals {
        graph.line(&format!(
            "  \"sig:{}\" [label=\"{}\\n{} {}\", shape=box];",
            signal.name,
            signal.name,
            signal.kind,
            width(signal.width)
        ));
    }

    for constant in &module.constants {
        let value = expr_label(&constant.expr);
        graph.line(&format!(
            "  \"const:{}\" [label=\"{}\\nconst {} = {}\", shape=note];",
            constant.name,
            constant.name,
            width(constant.width),
            escape(&value)
        ));
    }

    for assignment in &module.combinational {
        let expr = graph.expr_node(&assignment.expr);
        graph.line(&format!("  \"{}\" -> \"sig:{}\";", expr, assignment.target));
    }

    for (idx, process) in module.processes.iter().enumerate() {
        let process_id = format!("proc:{}", idx);
        graph.line(&format!(
            "  \"{}\" [label=\"{}({})\", shape=diamond];",
            process_id,
            edge(process.edge),
            process.clock
        ));
        graph.line(&format!(
            "  \"sig:{}\" -> \"{}\" [style=dashed];",
            process.clock, process_id
        ));
        for assignment in &process.assignments {
            let expr = graph.expr_node(&assignment.expr);
            graph.line(&format!("  \"{}\" -> \"{}\";", expr, process_id));
            graph.line(&format!(
                "  \"{}\" -> \"sig:{}\" [style=dashed];",
                process_id, assignment.target
            ));
        }
    }

    graph.line("}");
    graph.finish()
}

/// Emit a Mermaid flowchart for an IR module.
pub fn emit_mermaid(module: &IrModule) -> String {
    let mut graph = MermaidGraph::default();
    graph.line("flowchart LR");

    for signal in &module.signals {
        graph.line(&format!(
            "  {}[\"{}<br/>{} {}\"]",
            mermaid_id(&format!("sig_{}", signal.name)),
            signal.name,
            signal.kind,
            width(signal.width)
        ));
    }

    for constant in &module.constants {
        graph.line(&format!(
            "  {}[\"{}<br/>const {} = {}\"]",
            mermaid_id(&format!("const_{}", constant.name)),
            constant.name,
            width(constant.width),
            mermaid_label(&expr_label(&constant.expr))
        ));
    }

    for assignment in &module.combinational {
        let expr = graph.expr_node(&assignment.expr);
        graph.line(&format!(
            "  {} --> {}",
            expr,
            mermaid_id(&format!("sig_{}", assignment.target))
        ));
    }

    for (idx, process) in module.processes.iter().enumerate() {
        let process_id = mermaid_id(&format!("proc_{}", idx));
        graph.line(&format!(
            "  {}{{\"{}({})\"}}",
            process_id,
            edge(process.edge),
            process.clock
        ));
        graph.line(&format!(
            "  {} -.-> {}",
            mermaid_id(&format!("sig_{}", process.clock)),
            process_id
        ));
        for assignment in &process.assignments {
            let expr = graph.expr_node(&assignment.expr);
            graph.line(&format!("  {} --> {}", expr, process_id));
            graph.line(&format!(
                "  {} -.-> {}",
                process_id,
                mermaid_id(&format!("sig_{}", assignment.target))
            ));
        }
    }

    graph.finish()
}

#[derive(Default)]
struct DotGraph {
    lines: Vec<String>,
    next: usize,
}

impl DotGraph {
    fn line(&mut self, line: &str) {
        self.lines.push(line.to_string());
    }

    fn finish(self) -> String {
        self.lines.join("\n") + "\n"
    }

    fn expr_node(&mut self, expr: &IrExpr) -> String {
        match expr {
            IrExpr::Signal { name, .. } => format!("sig:{}", name),
            IrExpr::Const { value, .. } => self.leaf(&value.to_string()),
            IrExpr::Unary { op, expr, .. } => {
                let input = self.expr_node(expr);
                let node = self.op_node(op_name_unary(*op));
                self.line(&format!("  \"{}\" -> \"{}\";", input, node));
                node
            }
            IrExpr::Binary {
                op, left, right, ..
            } => {
                let left = self.expr_node(left);
                let right = self.expr_node(right);
                let node = self.op_node(op_name(*op));
                self.line(&format!("  \"{}\" -> \"{}\";", left, node));
                self.line(&format!("  \"{}\" -> \"{}\";", right, node));
                node
            }
            IrExpr::Mux {
                select,
                when_true,
                when_false,
                ..
            } => {
                let select = self.expr_node(select);
                let when_true = self.expr_node(when_true);
                let when_false = self.expr_node(when_false);
                let node = self.op_node("MUX");
                self.line(&format!(
                    "  \"{}\" -> \"{}\" [label=\"sel\"];",
                    select, node
                ));
                self.line(&format!(
                    "  \"{}\" -> \"{}\" [label=\"1\"];",
                    when_true, node
                ));
                self.line(&format!(
                    "  \"{}\" -> \"{}\" [label=\"0\"];",
                    when_false, node
                ));
                node
            }
        }
    }

    fn leaf(&mut self, label: &str) -> String {
        let node = format!("lit:{}", self.next);
        self.next += 1;
        self.line(&format!(
            "  \"{}\" [label=\"{}\", shape=plaintext];",
            node,
            escape(label)
        ));
        node
    }

    fn op_node(&mut self, label: &str) -> String {
        let node = format!("op:{}", self.next);
        self.next += 1;
        self.line(&format!(
            "  \"{}\" [label=\"{}\", shape=ellipse];",
            node, label
        ));
        node
    }
}

#[derive(Default)]
struct MermaidGraph {
    lines: Vec<String>,
    next: usize,
}

impl MermaidGraph {
    fn line(&mut self, line: &str) {
        self.lines.push(line.to_string());
    }

    fn finish(self) -> String {
        self.lines.join("\n") + "\n"
    }

    fn expr_node(&mut self, expr: &IrExpr) -> String {
        match expr {
            IrExpr::Signal { name, .. } => mermaid_id(&format!("sig_{}", name)),
            IrExpr::Const { value, .. } => self.leaf(&value.to_string()),
            IrExpr::Unary { op, expr, .. } => {
                let input = self.expr_node(expr);
                let node = self.op_node(op_name_unary(*op));
                self.line(&format!("  {} --> {}", input, node));
                node
            }
            IrExpr::Binary {
                op, left, right, ..
            } => {
                let left = self.expr_node(left);
                let right = self.expr_node(right);
                let node = self.op_node(op_name(*op));
                self.line(&format!("  {} --> {}", left, node));
                self.line(&format!("  {} --> {}", right, node));
                node
            }
            IrExpr::Mux {
                select,
                when_true,
                when_false,
                ..
            } => {
                let select = self.expr_node(select);
                let when_true = self.expr_node(when_true);
                let when_false = self.expr_node(when_false);
                let node = self.op_node("MUX");
                self.line(&format!("  {} -- sel --> {}", select, node));
                self.line(&format!("  {} -- 1 --> {}", when_true, node));
                self.line(&format!("  {} -- 0 --> {}", when_false, node));
                node
            }
        }
    }

    fn leaf(&mut self, label: &str) -> String {
        let node = mermaid_id(&format!("lit_{}", self.next));
        self.next += 1;
        self.line(&format!("  {}[\"{}\"]", node, mermaid_label(label)));
        node
    }

    fn op_node(&mut self, label: &str) -> String {
        let node = mermaid_id(&format!("op_{}", self.next));
        self.next += 1;
        self.line(&format!("  {}((\"{}\"))", node, mermaid_label(label)));
        node
    }
}

fn width(width: u32) -> String {
    if width == 1 {
        "bit".to_string()
    } else {
        format!("u{}", width)
    }
}

fn edge(edge: Edge) -> &'static str {
    match edge {
        Edge::Rising => "rising",
        Edge::Falling => "falling",
    }
}

fn expr_label(expr: &IrExpr) -> String {
    match expr {
        IrExpr::Const { value, .. } => value.to_string(),
        IrExpr::Signal { name, .. } => name.clone(),
        IrExpr::Unary { op, expr, .. } => format!("{}{}", op_name_unary(*op), expr_label(expr)),
        IrExpr::Binary {
            op, left, right, ..
        } => format!(
            "{} {} {}",
            expr_label(left),
            op_name(*op),
            expr_label(right)
        ),
        IrExpr::Mux {
            select,
            when_true,
            when_false,
            ..
        } => format!(
            "if {} then {} else {}",
            expr_label(select),
            expr_label(when_true),
            expr_label(when_false)
        ),
    }
}

fn op_name(op: BinaryOp) -> &'static str {
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

fn op_name_unary(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::LogicNot => "!",
        UnaryOp::BitNot => "~",
        UnaryOp::Neg => "-",
    }
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

fn mermaid_label(text: &str) -> String {
    text.replace('"', "'")
}

fn mermaid_id(text: &str) -> String {
    text.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}
