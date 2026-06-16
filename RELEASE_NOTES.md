# Frag v0.1.0-alpha.3

This alpha release adds case expressions and hardens the compiler pipeline
around typed IR validation, golden backend tests, and example-generated Verilog
checks.

## Highlights

- Added `case selector { pattern => value, else => value }` expressions
- Added `examples/mux4_case.frag`
- Lowered case expressions into typed IR case nodes
- Added Verilog, simulator, DOT, and Mermaid support for case expressions
- Added `frag check` for frontend, semantic, and IR validation
- Added an IR validation pass before backend execution
- Added golden tests for representative IR, Verilog, and Mermaid output
- Added property-style simulator tests for case selection and width masking
- Added a formal grammar reference in `docs/GRAMMAR.md`
- Updated CI and release workflows to current GitHub Actions majors

## Example

```frag
module Mux4Case {
    input sel: u2;
    input a: u8;
    input b: u8;
    input c: u8;
    input d: u8;

    output out: u8;

    out = case sel {
        0 => a,
        1 => b,
        2 => c,
        else => d
    };
}
```

Generated Verilog:

```verilog
assign out = ((sel == 0) ? a : ((sel == 1) ? b : ((sel == 2) ? c : d)));
```

## Compatibility

Frag remains pre-1.0. The language and CLI may change between alpha releases.

Major missing features still include module instantiation, reset syntax,
arrays, memories, signed arithmetic, and bit indexing/slicing.
