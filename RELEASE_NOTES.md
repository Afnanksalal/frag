# Frag v0.1.0-alpha.1

This is the first HDL-focused alpha release of Frag.

Frag is a tiny Hardware Description Language compiler written in Rust. This release demonstrates the full teaching pipeline:

```text
Frag source -> Lexer -> Parser -> AST -> Semantic analysis -> IR -> Verilog / Simulator / Graph
```

## Highlights

- Parse small Frag HDL modules
- Check signal declarations, widths, assignment targets, and dependency cycles
- Lower checked modules into a netlist-style IR
- Generate Verilog accepted by Icarus Verilog and Verilator
- Simulate combinational truth tables and simple sequential tick traces
- Emit VCD waveforms
- Generate Graphviz DOT and Mermaid circuit graphs
- Includes 13 example circuits

## Install

Download the archive for your platform from the release assets, extract it, and run:

```bash
frag --help
```

Or build from source:

```bash
cargo install --path .
```

## Example

```bash
frag run examples/half_adder.frag
frag verilog examples/half_adder.frag
frag graph examples/half_adder.frag --format mermaid
```

## Alpha Notice

This is an early alpha. The compiler is usable for small examples and demos, but the language is not stable yet.

Major missing features include module instantiation, `if` / `case`, reset syntax, arrays, memories, and signed arithmetic.
