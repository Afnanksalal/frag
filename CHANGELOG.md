# Changelog

All notable changes to Frag are documented here.

This project follows semantic versioning once the language stabilizes. Until then, pre-1.0 releases may still introduce breaking language or CLI changes.

## Unreleased

No unreleased changes yet.

## v0.1.0-alpha.5 - 2026-06-16

### Added

- `control_datapath.frag` stress example covering state, nested conditionals,
  case expressions, bit selection, arithmetic, flags, VCD, and graph output
- Regression test for bare `<` and `>` comparison tokenization

### Fixed

- Lexer now recognizes bare `<` and `>` operators, not only `<=`, `>=`, `<<`,
  and `>>`

## v0.1.0-alpha.4 - 2026-06-16

### Added

- Bit indexing with `expr[index]`
- Descending inclusive bit slicing with `expr[msb:lsb]`
- `nibble_splitter.frag` example
- Golden IR, Verilog, and Mermaid tests for bit selection
- Exhaustive simulator coverage for byte slicing behavior

### Changed

- DOT and Mermaid graph output now resolve constant references to constant nodes
- Verilog emission uses direct part-selects for named values and shift/mask emission for sliced expressions

## v0.1.0-alpha.3 - 2026-06-16

### Changed

- IR assignments and constants now store typed `IrExpr` nodes instead of AST expressions
- Verilog, simulator, DOT, and Mermaid backends now consume IR expressions directly
- Simulator masks intermediate expression results to their IR widths
- Architecture documentation now describes the typed IR contract
- CI and release workflows use current GitHub action majors to avoid Node 20 deprecation warnings
- Roadmap and language docs now include implemented case-expression syntax

### Added

- Regression coverage for IR mux lowering and intermediate bit-width masking
- `frag check` command for frontend, semantic, and IR validation
- IR validation pass for backend invariants
- Golden output tests for representative IR, Verilog, and Mermaid output
- `case selector { pattern => value, else => value }` expressions
- `mux4_case.frag` example
- Formal grammar reference in `docs/GRAMMAR.md`
- Exhaustive property-style simulator tests for case selection and width masking

## v0.1.0-alpha.2 - 2026-06-16

### Added

- Conditional expressions using `if condition { then_expr } else { else_expr }`
- `mux4_if.frag` example
- IR, Verilog, simulator, DOT, and Mermaid support for conditional expressions
- Pipeline tests for conditional expression lowering, simulation, graph output, and width checking

### Changed

- README and roadmap documentation now use a shorter technical repository style

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
