# Frag v0.1.0-alpha.5

This alpha release adds a larger control/datapath example and fixes bare
comparison tokenization.

## Highlights

- Added `examples/control_datapath.frag`
- The new example exercises registers, wires, constants, nested `if`, nested
  `case`, arithmetic, comparisons, bit indexing/slicing, status flags,
  simulation, VCD output, and graph generation
- Fixed lexing for bare `<` and `>` comparison operators
- Added a regression test for bare comparison parsing and simulation
- Verified the generated control/datapath Verilog with Icarus Verilog and
  Verilator

## Example

```frag
module ControlDatapath {
    input clk: bit;
    input start: bit;
    input opcode: u4;
    input data: u8;

    output result: u8;
    output busy: bit;
    output done: bit;

    reg acc: u8;
    reg state: u3;

    result = acc;
    busy = state != 0;
    done = state == 3;

    on rising(clk) {
        acc = if start { data ^ opcode } else { acc };
        state = case state {
            0 => if start { 1 } else { 0 },
            1 => 2,
            2 => 3,
            else => 0
        };
    }
}
```

Generated Verilog:

```verilog
always @(posedge clk) begin
    acc <= (start ? (data ^ opcode) : acc);
    state <= ((state == 0) ? (start ? 1 : 0) : ((state == 1) ? 2 : ((state == 2) ? 3 : 0)));
end
```

## Compatibility

Frag remains pre-1.0. The language and CLI may change between alpha releases.

Major missing features still include module instantiation, reset syntax,
arrays, memories, signed arithmetic, and concatenation.
