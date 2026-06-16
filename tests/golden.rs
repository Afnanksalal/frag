use frag_compiler::{compile, graph, verilog};

#[test]
fn half_adder_ir_matches_golden() {
    let source = include_str!("../examples/half_adder.frag");
    let compiled = compile(source).expect("half adder should compile");
    assert_eq!(
        normalize(&compiled.ir.to_string()),
        normalize(include_str!("golden/half_adder.ir"))
    );
}

#[test]
fn half_adder_verilog_matches_golden() {
    let source = include_str!("../examples/half_adder.frag");
    let compiled = compile(source).expect("half adder should compile");
    assert_eq!(
        normalize(&verilog::emit(&compiled.ir)),
        normalize(include_str!("golden/half_adder.v"))
    );
}

#[test]
fn mux4_if_ir_matches_golden() {
    let source = include_str!("../examples/mux4_if.frag");
    let compiled = compile(source).expect("mux4_if should compile");
    assert_eq!(
        normalize(&compiled.ir.to_string()),
        normalize(include_str!("golden/mux4_if.ir"))
    );
}

#[test]
fn mux4_if_mermaid_matches_golden() {
    let source = include_str!("../examples/mux4_if.frag");
    let compiled = compile(source).expect("mux4_if should compile");
    assert_eq!(
        normalize(&graph::emit_mermaid(&compiled.ir)),
        normalize(include_str!("golden/mux4_if.mmd"))
    );
}

#[test]
fn mux4_case_ir_matches_golden() {
    let source = include_str!("../examples/mux4_case.frag");
    let compiled = compile(source).expect("mux4_case should compile");
    assert_eq!(
        normalize(&compiled.ir.to_string()),
        normalize(include_str!("golden/mux4_case.ir"))
    );
}

#[test]
fn mux4_case_verilog_matches_golden() {
    let source = include_str!("../examples/mux4_case.frag");
    let compiled = compile(source).expect("mux4_case should compile");
    assert_eq!(
        normalize(&verilog::emit(&compiled.ir)),
        normalize(include_str!("golden/mux4_case.v"))
    );
}

#[test]
fn mux4_case_mermaid_matches_golden() {
    let source = include_str!("../examples/mux4_case.frag");
    let compiled = compile(source).expect("mux4_case should compile");
    assert_eq!(
        normalize(&graph::emit_mermaid(&compiled.ir)),
        normalize(include_str!("golden/mux4_case.mmd"))
    );
}

#[test]
fn nibble_splitter_ir_matches_golden() {
    let source = include_str!("../examples/nibble_splitter.frag");
    let compiled = compile(source).expect("nibble_splitter should compile");
    assert_eq!(
        normalize(&compiled.ir.to_string()),
        normalize(include_str!("golden/nibble_splitter.ir"))
    );
}

#[test]
fn nibble_splitter_verilog_matches_golden() {
    let source = include_str!("../examples/nibble_splitter.frag");
    let compiled = compile(source).expect("nibble_splitter should compile");
    assert_eq!(
        normalize(&verilog::emit(&compiled.ir)),
        normalize(include_str!("golden/nibble_splitter.v"))
    );
}

#[test]
fn nibble_splitter_mermaid_matches_golden() {
    let source = include_str!("../examples/nibble_splitter.frag");
    let compiled = compile(source).expect("nibble_splitter should compile");
    assert_eq!(
        normalize(&graph::emit_mermaid(&compiled.ir)),
        normalize(include_str!("golden/nibble_splitter.mmd"))
    );
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n")
}
