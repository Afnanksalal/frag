use frag_compiler::ir::{self, IrAssign, IrExpr, IrModule, IrSignal, IrSignalKind};
use frag_compiler::simulator::{SimOptions, SimulationResult};
use frag_compiler::{compile, graph, simulator, verilog};
use std::collections::BTreeMap;
use std::fs;

#[test]
fn half_adder_generates_verilog() {
    let source = include_str!("../examples/half_adder.frag");
    let compiled = compile(source).expect("half adder should compile");
    let text = verilog::emit(&compiled.ir);

    assert!(text.contains("module HalfAdder"));
    assert!(text.contains("assign sum = (a ^ b);"));
    assert!(text.contains("assign carry = (a & b);"));
}

#[test]
fn arbitrary_combinational_module_lowers_and_simulates() {
    let source = r#"
module WeirdMixer42 {
    input left: u4;
    input right: u4;
    input enable: bit;

    output mixed: u4;
    output live: bit;

    wire stage: u4;

    const bump: u4 = 2;
    const mask: u4 = bump + 1;

    stage = (left ^ right) & mask;
    mixed = stage + bump;
    live = (mixed != 0) && enable;
}
"#;

    let compiled = compile(source).expect("arbitrary module should compile");
    assert_eq!(compiled.ir.name, "WeirdMixer42");
    assert!(compiled
        .ir
        .signals
        .iter()
        .any(|signal| signal.name == "stage"));
    assert_eq!(compiled.ir.constants[0].name, "bump");
    assert_eq!(compiled.ir.constants[1].name, "mask");

    let mut inputs = BTreeMap::new();
    inputs.insert("left".to_string(), 1);
    inputs.insert("right".to_string(), 2);
    inputs.insert("enable".to_string(), 1);

    let result = simulator::run(&compiled.ir, &SimOptions { ticks: 1, inputs })
        .expect("simulation should work");
    let SimulationResult::TruthTable(table) = result else {
        panic!("combinational module should produce a truth table");
    };

    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.rows[0]["stage"], 3);
    assert_eq!(table.rows[0]["mixed"], 5);
    assert_eq!(table.rows[0]["live"], 1);
}

#[test]
fn conditional_expression_lowers_to_mux_and_simulates() {
    let source = r#"
module IfMux {
    input sel: bit;
    input a: u4;
    input b: u4;

    output out: u4;

    out = if sel { a } else { b };
}
"#;

    let compiled = compile(source).expect("conditional mux should compile");

    let IrExpr::Mux {
        select,
        when_true,
        when_false,
        width,
    } = &compiled.ir.combinational[0].expr
    else {
        panic!("conditional expression should lower to an IR mux");
    };
    assert_eq!(*width, 4);
    assert!(matches!(
        select.as_ref(),
        IrExpr::Signal { name, width: 1 } if name == "sel"
    ));
    assert!(matches!(
        when_true.as_ref(),
        IrExpr::Signal { name, width: 4 } if name == "a"
    ));
    assert!(matches!(
        when_false.as_ref(),
        IrExpr::Signal { name, width: 4 } if name == "b"
    ));

    let ir_text = compiled.ir.to_string();
    assert!(ir_text.contains("Gate MUX"));
    assert!(ir_text.contains("Select: sel"));

    let verilog = verilog::emit(&compiled.ir);
    assert!(verilog.contains("assign out = (sel ? a : b);"));

    let dot = graph::emit_dot(&compiled.ir);
    assert!(dot.contains("MUX"));
    assert!(dot.contains("[label=\"sel\"]"));

    let mermaid = graph::emit_mermaid(&compiled.ir);
    assert!(mermaid.contains("MUX"));

    let mut inputs = BTreeMap::new();
    inputs.insert("sel".to_string(), 1);
    inputs.insert("a".to_string(), 9);
    inputs.insert("b".to_string(), 3);
    let result = simulator::run(&compiled.ir, &SimOptions { ticks: 1, inputs })
        .expect("selected true branch should simulate");
    let SimulationResult::TruthTable(table) = result else {
        panic!("conditional mux should produce a truth table");
    };
    assert_eq!(table.rows[0]["out"], 9);

    let mut inputs = BTreeMap::new();
    inputs.insert("sel".to_string(), 0);
    inputs.insert("a".to_string(), 9);
    inputs.insert("b".to_string(), 3);
    let result = simulator::run(&compiled.ir, &SimOptions { ticks: 1, inputs })
        .expect("selected false branch should simulate");
    let SimulationResult::TruthTable(table) = result else {
        panic!("conditional mux should produce a truth table");
    };
    assert_eq!(table.rows[0]["out"], 3);
}

#[test]
fn simulator_masks_intermediate_ir_expression_widths() {
    let source = r#"
module ShiftedNot {
    input a: u4;
    output out: u4;

    out = ~a >> 1;
}
"#;

    let compiled = compile(source).expect("shifted bit-not should compile");
    let IrExpr::Binary { width, .. } = &compiled.ir.combinational[0].expr else {
        panic!("shifted bit-not should lower to a binary expression");
    };
    assert_eq!(*width, 4);

    let mut inputs = BTreeMap::new();
    inputs.insert("a".to_string(), 0);
    let result = simulator::run(&compiled.ir, &SimOptions { ticks: 1, inputs })
        .expect("simulation should work");
    let SimulationResult::TruthTable(table) = result else {
        panic!("combinational module should produce a truth table");
    };
    assert_eq!(table.rows[0]["out"], 7);
}

#[test]
fn ir_validation_rejects_unknown_references() {
    let module = IrModule {
        name: "BrokenIr".to_string(),
        signals: vec![IrSignal {
            name: "out".to_string(),
            kind: IrSignalKind::Output,
            width: 1,
        }],
        constants: Vec::new(),
        combinational: vec![IrAssign {
            target: "out".to_string(),
            expr: IrExpr::Signal {
                name: "missing".to_string(),
                width: 1,
            },
        }],
        processes: Vec::new(),
    };

    let error = ir::validate(&module).expect_err("unknown IR reference should fail");
    assert!(error.message.contains("undeclared `missing`"));
}

#[test]
fn ir_validation_rejects_width_invariants() {
    let module = IrModule {
        name: "BadWidthIr".to_string(),
        signals: vec![
            IrSignal {
                name: "a".to_string(),
                kind: IrSignalKind::Input,
                width: 4,
            },
            IrSignal {
                name: "out".to_string(),
                kind: IrSignalKind::Output,
                width: 4,
            },
        ],
        constants: Vec::new(),
        combinational: vec![IrAssign {
            target: "out".to_string(),
            expr: IrExpr::Unary {
                op: frag_compiler::ast::UnaryOp::LogicNot,
                expr: Box::new(IrExpr::Signal {
                    name: "a".to_string(),
                    width: 4,
                }),
                width: 4,
            },
        }],
        processes: Vec::new(),
    };

    let error = ir::validate(&module).expect_err("invalid IR expression width should fail");
    assert!(error.message.contains("unary expression"));
}

#[test]
fn conditional_expression_rejects_unsafe_branch_width() {
    let source = r#"
module BadConditionalWidth {
    input sel: bit;
    input wide: u4;

    output out: bit;

    out = if sel { wide } else { 0 };
}
"#;

    let error = compile(source).expect_err("wide branch should fail width checking");
    assert!(error.message.contains("Width mismatch"));
}

#[test]
fn constant_forward_reference_is_ordered_before_backends() {
    let source = r#"
module ForwardConst {
    input passthrough: u4;

    output out: u4;
    output same: bit;

    const second: u4 = first + 1;
    const first: u4 = 2;

    out = second;
    same = passthrough == out;
}
"#;

    let compiled = compile(source).expect("forward constant reference should compile");
    let constant_names = compiled
        .ir
        .constants
        .iter()
        .map(|constant| constant.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(constant_names, vec!["first", "second"]);

    let text = verilog::emit(&compiled.ir);
    let first_pos = text.find("localparam [3:0] first = 2;").unwrap();
    let second_pos = text.find("localparam [3:0] second = (first + 1);").unwrap();
    assert!(first_pos < second_pos);
}

#[test]
fn reports_unknown_signal() {
    let source = r#"
module Broken {
    input a: bit;
    output y: bit;
    y = a ^ missing;
}
"#;

    let error = compile(source).expect_err("unknown signal should fail");
    assert!(error.message.contains("Unknown signal `missing`"));
}

#[test]
fn reports_combinational_cycle() {
    let source = r#"
module Cycle {
    input a: bit;
    output y: bit;
    wire w: bit;
    y = w;
    w = y;
}
"#;

    let error = compile(source).expect_err("cycle should fail");
    assert!(error.message.contains("Circular combinational reference"));
}

#[test]
fn reports_width_mismatch() {
    let source = r#"
module BadWidth {
    input wide: u4;
    output small: bit;
    small = wide;
}
"#;

    let error = compile(source).expect_err("wide expression into bit should fail");
    assert!(error.message.contains("Width mismatch"));
}

#[test]
fn reports_duplicate_declaration() {
    let source = r#"
module Duplicate {
    input a: bit;
    output a: bit;
    a = 0;
}
"#;

    let error = compile(source).expect_err("duplicate declaration should fail");
    assert!(error.message.contains("Duplicate declaration"));
}

#[test]
fn reports_const_cycle() {
    let source = r#"
module ConstCycle {
    output out: u4;
    const a: u4 = b;
    const b: u4 = a;
    out = a;
}
"#;

    let error = compile(source).expect_err("constant cycle should fail");
    assert!(error.message.contains("Circular constant dependency"));
}

#[test]
fn reports_invalid_sequential_target() {
    let source = r#"
module BadSeq {
    input clk: bit;
    output out: bit;
    out = 0;
    on rising(clk) {
        out = 1;
    }
}
"#;

    let error = compile(source).expect_err("sequential output assignment should fail");
    assert!(error.message.contains("must be a register"));
}

#[test]
fn half_adder_truth_table_contains_expected_rows() {
    let source = include_str!("../examples/half_adder.frag");
    let compiled = compile(source).expect("half adder should compile");
    let result = simulator::run(&compiled.ir, &SimOptions::default()).expect("simulation works");

    let SimulationResult::TruthTable(table) = result else {
        panic!("half adder should produce a truth table");
    };

    assert_eq!(table.rows.len(), 4);
    let row = table
        .rows
        .iter()
        .find(|row| row["a"] == 1 && row["b"] == 1)
        .expect("1 + 1 row exists");
    assert_eq!(row["sum"], 0);
    assert_eq!(row["carry"], 1);
}

#[test]
fn every_example_compiles_through_ir() {
    let mut checked = 0;
    for entry in fs::read_dir("examples").expect("examples directory exists") {
        let path = entry.expect("directory entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("frag") {
            continue;
        }
        let source = fs::read_to_string(&path).expect("example is readable");
        let compiled = compile(&source)
            .unwrap_or_else(|error| panic!("{} should compile: {}", path.display(), error.message));
        assert!(!compiled.ir.name.is_empty());
        checked += 1;
    }

    assert!(
        checked >= 10,
        "expected at least ten examples, found {checked}"
    );
}

#[test]
fn counter_ticks_forward() {
    let source = include_str!("../examples/counter.frag");
    let compiled = compile(source).expect("counter should compile");
    let options = SimOptions {
        ticks: 4,
        ..SimOptions::default()
    };
    let result = simulator::run(&compiled.ir, &options).expect("simulation works");

    let SimulationResult::Waveform(waveform) = result else {
        panic!("counter should produce a waveform");
    };

    assert_eq!(waveform.values["count"], vec![0, 1, 2, 3]);
    assert_eq!(waveform.values["count_reg"], vec![0, 1, 2, 3]);
}
