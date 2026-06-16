# Frag v0.1.0-alpha.4

This alpha release adds bit indexing and slicing across the full compiler
pipeline.

## Highlights

- Added `expr[index]` one-bit indexing
- Added `expr[msb:lsb]` descending inclusive slicing
- Added typed IR slice nodes consumed by all backends
- Added simulator support for indexed and sliced expressions
- Added direct Verilog part-select output for named values
- Added shift/mask Verilog output for sliced compound expressions
- Added DOT and Mermaid output for slice nodes
- Fixed graph output for constant references inside expressions
- Added `examples/nibble_splitter.frag`
- Added golden and exhaustive simulator tests for bit selection

## Example

```frag
module NibbleSplitter {
    input data: u8;

    output high: u4;
    output low: u4;
    output top: bit;

    high = data[7:4];
    low = data[3:0];
    top = data[7];
}
```

Generated Verilog:

```verilog
assign high = data[7:4];
assign low = data[3:0];
assign top = data[7];
```

## Compatibility

Frag remains pre-1.0. The language and CLI may change between alpha releases.

Major missing features still include module instantiation, reset syntax,
arrays, memories, signed arithmetic, and concatenation.
