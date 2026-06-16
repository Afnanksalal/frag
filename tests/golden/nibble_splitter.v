module NibbleSplitter(
    input [7:0] data,
    output [3:0] high,
    output [3:0] low,
    output top,
    output bit2,
    output [3:0] masked_low
);

localparam [7:0] mask = 15;

assign high = data[7:4];
assign low = data[3:0];
assign top = data[7];
assign bit2 = data[2];
assign masked_low = {((((data & mask) >> 3) & 1) != 0), ((((data & mask) >> 2) & 1) != 0), ((((data & mask) >> 1) & 1) != 0), ((((data & mask) >> 0) & 1) != 0)};

endmodule
