module HalfAdder {
    input a: bit;
    input b: bit;

    output sum: bit;
    output carry: bit;

    sum = a ^ b;
    carry = a & b;
}
