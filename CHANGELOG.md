# Changelog

All notable changes to Frag are documented here.

This project follows semantic versioning once the language stabilizes. Until then, pre-1.0 releases may still introduce breaking language or CLI changes.

## v0.1.0-alpha.1 - 2026-06-15

Initial HDL-focused alpha release.

### Added

- Frag HDL parser for one module per source file
- Inputs, outputs, wires, registers, and constants
- `bit`, `bool`, and `uN` unsigned width syntax
- Combinational assignments
- Sequential `on rising(clk)` and `on falling(clk)` blocks
- Arithmetic, comparison, logical, bitwise, and shift operators
- Source-span diagnostics
- Semantic checks for:
  - duplicate declarations
  - unknown signals
  - invalid assignment targets
  - width mismatches
  - unassigned outputs
  - constant dependency cycles
  - combinational cycles
- Netlist-style IR
- Verilog backend
- Built-in simulator
- VCD waveform generation
- Graphviz DOT output
- Mermaid graph output
- 13 example circuits
- Rust tests and external HDL toolchain integration tests
- GitHub CI and release workflows

### Known Limits

- One module per source file
- No module instantiation yet
- No arrays or memories
- No `if`, `case`, loops, or generics yet
- No reset syntax yet
- No signed arithmetic yet
