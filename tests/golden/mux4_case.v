module Mux4Case(
    input [1:0] sel,
    input [7:0] a,
    input [7:0] b,
    input [7:0] c,
    input [7:0] d,
    output [7:0] out
);

assign out = ((sel == 0) ? a : ((sel == 1) ? b : ((sel == 2) ? c : d)));

endmodule
