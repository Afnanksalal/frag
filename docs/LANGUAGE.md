# Frag Language Reference

This document describes the language implemented by the current compiler.

Frag is intentionally small. The design favors explicit hardware structure over convenience features.

## Module

Every source file currently contains one module:

```frag
module Name {
    // declarations
    // assignments
    // processes
}
```

Module names and signal names use identifier syntax:

```text
[A-Za-z_][A-Za-z0-9_]*
```

## Types

Frag supports unsigned bit vectors:

```frag
bit   // one bit
u4    // four-bit unsigned value
u8    // eight-bit unsigned value
u32   // 32-bit unsigned value
```

Widths must be between 1 and 128 bits.

`bool` is accepted as an alias for `bit`.

## Declarations

```frag
input a: bit;
output sum: bit;
wire temp: u4;
reg count: u8;
const mask: u4 = 0b0011;
```

Declaration kinds:

- `input`: module input port
- `output`: module output port
- `wire`: internal combinational signal
- `reg`: sequential state updated in an `on rising` or `on falling` block
- `const`: compile-time constant emitted as a Verilog `localparam`

## Combinational Assignments

```frag
sum = a ^ b;
carry = a & b;
```

Combinational targets must be `output` or `wire` signals.

Each combinational target may have one driver.

The semantic analyzer rejects circular combinational dependencies:

```frag
a = b;
b = a;
```

## Sequential Processes

```frag
on rising(clk) {
    count = count + 1;
}
```

```frag
on falling(clk) {
    sample = data;
}
```

Rules:

- The clock must be a one-bit `input`.
- Sequential assignment targets must be `reg` signals.
- A register can be driven by only one process.
- Assignments inside a process emit Verilog nonblocking assignments (`<=`).

## Operators

Unary:

```text
!   logical not
~   bitwise not
-   unsigned wrapping negation in simulation
```

Binary, from high to low precedence:

```text
* / %
+ -
<< >>
< <= > >=
== !=
&
^
|
&&
||
```

## Conditional Expressions

Frag supports mux-friendly conditional expressions:

```frag
out = if sel { a } else { b };
```

The condition is treated as false when it evaluates to zero and true otherwise.
Both branches are ordinary expressions, so conditionals can be nested:

```frag
out = if sel == 0 {
    a
} else {
    if sel == 1 { b } else { c }
};
```

The Verilog backend emits a ternary expression:

```verilog
assign out = (sel ? a : b);
```

The result width is the wider of the two selectable branches. Assigning that
result to a narrower target is rejected unless the whole expression is an
unsized constant that fits.

## Case Expressions

Case expressions select one value from multiple patterns:

```frag
out = case sel {
    0 => a,
    1 => b,
    2 => c,
    else => d
};
```

Rules:

- A case expression must contain exactly one `else` arm.
- The `else` arm must be last.
- Constant duplicate patterns are rejected.
- Pattern expressions cannot be wider than the selector.
- Constant patterns must fit in the selector width.
- Pattern and value expressions are resolved and checked like other
  expressions.
- The result width is the widest arm value.

The Verilog backend emits nested ternary expressions.

## Literals

```frag
0
42
0b1010
0x2a
true
false
```

Integer literals are unsigned. The semantic analyzer allows unsized constants to fit into the target width when possible:

```frag
output out: u4;
out = 3; // accepted
```

This is rejected:

```frag
output out: bit;
input value: u4;
out = value; // width mismatch
```

## Comments

```frag
// line comment
# line comment
/* block comment */
```

## Current Limits

- One module per source file
- No module instantiation
- No memories
- No arrays
- No loops
- No generics
- No signed arithmetic
- No reset syntax yet

These limits describe the currently implemented language surface.
