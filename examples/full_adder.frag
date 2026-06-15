module FullAdder {
    input a: bit;
    input b: bit;
    input cin: bit;

    output sum: bit;
    output carry: bit;

    wire axb: bit;

    axb = a ^ b;
    sum = axb ^ cin;
    carry = (a & b) | (cin & axb);
}
