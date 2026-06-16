module Mux4If {
    input sel: u2;
    input a: u8;
    input b: u8;
    input c: u8;
    input d: u8;

    output out: u8;

    out = if sel == 0 {
        a
    } else {
        if sel == 1 {
            b
        } else {
            if sel == 2 {
                c
            } else {
                d
            }
        }
    };
}
