module Counter {
    input clk: bit;

    output count: u8;

    reg count_reg: u8;

    count = count_reg;

    on rising(clk) {
        count_reg = count_reg + 1;
    }
}
