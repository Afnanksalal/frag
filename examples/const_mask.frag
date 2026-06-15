module ConstMask {
    input value: u4;

    output low: bit;
    output masked: u4;

    const mask: u4 = 0b0011;

    low = value != 0;
    masked = value & mask;
}
