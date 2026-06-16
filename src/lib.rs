//! Frag compiler library.
//!
//! The public API exposes each compiler stage for inspection and tooling:
//! lexing, parsing, semantic analysis, IR lowering, simulation, graph
//! generation, and Verilog emission. Most callers should start with
//! [`compile`], then pass the returned IR to a backend.

#![forbid(unsafe_code)]

/// Source-level abstract syntax tree.
pub mod ast;
/// Compiler diagnostics and source spans.
pub mod diagnostic;
/// DOT and Mermaid graph emitters.
pub mod graph;
/// Netlist-style intermediate representation.
pub mod ir;
/// Source lexer.
pub mod lexer;
/// Recursive descent parser.
pub mod parser;
/// Semantic analyzer.
pub mod semantic;
/// Built-in simulator and VCD emitter.
pub mod simulator;
/// Verilog backend.
pub mod verilog;

use diagnostic::Result;

#[derive(Clone, Debug)]
pub struct CompileOutput {
    /// Parsed source-level AST.
    pub ast: ast::Module,
    /// Semantic information computed from the AST.
    pub analysis: semantic::Analysis,
    /// Lowered netlist IR used by backends.
    pub ir: ir::IrModule,
}

/// Run the full frontend and IR lowering pipeline for one Frag module.
///
/// This function performs lexing, parsing, semantic analysis, and lowering.
/// It does not emit Verilog or run simulation; those are separate backend
/// steps that consume [`CompileOutput::ir`].
pub fn compile(source: &str) -> Result<CompileOutput> {
    let ast = parser::parse_source(source)?;
    let analysis = semantic::analyze(&ast)?;
    let ir = ir::lower(&ast, &analysis);
    ir::validate(&ir)?;
    Ok(CompileOutput { ast, analysis, ir })
}
