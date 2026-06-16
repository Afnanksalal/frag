module NibbleSplitter {
    input data: u8;

    output high: u4;
    output low: u4;
    output top: bit;
    output bit2: bit;
    output masked_low: u4;

    const mask: u8 = 0x0f;

    high = data[7:4];
    low = data[3:0];
    top = data[7];
    bit2 = data[2];
    masked_low = (data & mask)[3:0];
}
