module Majority3 {
    input a: bit;
    input b: bit;
    input c: bit;

    output out: bit;

    out = (a & b) | (a & c) | (b & c);
}
