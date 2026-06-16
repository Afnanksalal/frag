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
