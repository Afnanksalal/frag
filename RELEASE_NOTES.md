# Frag v0.1.0-alpha.2

This alpha release adds conditional expressions and updates the project
documentation to match the current compiler surface.

## Highlights

- Added `if condition { then_expr } else { else_expr }` conditional expressions
- Lowered conditional expressions as MUX nodes in the IR
- Emitted Verilog ternary expressions for conditionals
- Added simulator support for conditional expression evaluation
- Added DOT and Mermaid graph output for MUX nodes
- Added `examples/mux4_if.frag`
- Added tests for conditional expression lowering, simulation, graph output,
  Verilog output, and width checking
- Reworked README and roadmap documentation into a concise technical format

## Example

```frag
module Mux2If {
    input sel: bit;
    input a: u8;
    input b: u8;

    output out: u8;

    out = if sel { a } else { b };
}
```

Generated Verilog:

```verilog
assign out = (sel ? a : b);
```

## Compatibility

Frag remains pre-1.0. The language and CLI may change between alpha releases.

Major missing features still include module instantiation, `case`, reset
syntax, arrays, memories, signed arithmetic, and bit indexing/slicing.
