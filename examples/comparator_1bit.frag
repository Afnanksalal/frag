module Comparator1Bit {
    input a: bit;
    input b: bit;

    output eq: bit;
    output lt: bit;
    output gt: bit;

    eq = !(a ^ b);
    lt = !a & b;
    gt = a & !b;
}
