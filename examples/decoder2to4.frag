module Decoder2To4 {
    input a0: bit;
    input a1: bit;

    output y0: bit;
    output y1: bit;
    output y2: bit;
    output y3: bit;

    y0 = !a1 & !a0;
    y1 = !a1 & a0;
    y2 = a1 & !a0;
    y3 = a1 & a0;
}
