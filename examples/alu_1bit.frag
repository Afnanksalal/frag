module Alu1Bit {
    input a: bit;
    input b: bit;

    output sum: bit;
    output carry: bit;
    output both: bit;
    output either: bit;

    sum = a ^ b;
    carry = a & b;
    both = a & b;
    either = a | b;
}
