module Mux2 {
    input a: bit;
    input b: bit;
    input sel: bit;

    output out: bit;

    out = (a & !sel) | (b & sel);
}
