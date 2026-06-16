use frag_compiler::simulator::{SimOptions, SimulationResult};
use frag_compiler::{compile, simulator};
use std::collections::BTreeMap;

#[test]
fn mux4_case_matches_reference_model_for_all_selects() {
    let source = include_str!("../examples/mux4_case.frag");
    let compiled = compile(source).expect("mux4_case should compile");
    let data = [10, 20, 30, 40];

    for sel in 0..4 {
        let mut inputs = BTreeMap::new();
        inputs.insert("sel".to_string(), sel);
        inputs.insert("a".to_string(), data[0]);
        inputs.insert("b".to_string(), data[1]);
        inputs.insert("c".to_string(), data[2]);
        inputs.insert("d".to_string(), data[3]);

        let row = simulate_one(&compiled.ir, inputs);
        assert_eq!(row["out"], data[sel as usize]);
    }
}

#[test]
fn u4_bitwise_not_shift_matches_masked_reference_for_all_inputs() {
    let source = r#"
module ShiftedNot {
    input a: u4;
    output out: u4;

    out = ~a >> 1;
}
"#;
    let compiled = compile(source).expect("shifted bit-not should compile");

    for a in 0..16 {
        let mut inputs = BTreeMap::new();
        inputs.insert("a".to_string(), a);

        let row = simulate_one(&compiled.ir, inputs);
        assert_eq!(row["out"], ((!a) & 0xf) >> 1);
    }
}

#[test]
fn byte_slices_match_reference_model_for_all_inputs() {
    let source = include_str!("../examples/nibble_splitter.frag");
    let compiled = compile(source).expect("nibble splitter should compile");

    for data in 0..=255 {
        let mut inputs = BTreeMap::new();
        inputs.insert("data".to_string(), data);

        let row = simulate_one(&compiled.ir, inputs);
        assert_eq!(row["high"], (data >> 4) & 0xf);
        assert_eq!(row["low"], data & 0xf);
        assert_eq!(row["top"], (data >> 7) & 1);
        assert_eq!(row["bit2"], (data >> 2) & 1);
        assert_eq!(row["masked_low"], data & 0xf);
    }
}

fn simulate_one(
    module: &frag_compiler::ir::IrModule,
    inputs: BTreeMap<String, u128>,
) -> BTreeMap<String, u128> {
    let result =
        simulator::run(module, &SimOptions { ticks: 1, inputs }).expect("simulation should work");
    let SimulationResult::TruthTable(table) = result else {
        panic!("combinational module should produce a truth table");
    };
    assert_eq!(table.rows.len(), 1);
    table.rows[0].clone()
}
