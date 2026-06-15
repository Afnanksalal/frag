module Register8 {
    input clk: bit;
    input data: u8;

    output q: u8;

    reg q_reg: u8;

    q = q_reg;

    on rising(clk) {
        q_reg = data;
    }
}
