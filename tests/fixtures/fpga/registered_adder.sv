`include "config.vh"
module registered_adder #(
    parameter WIDTH = `FIXTURE_WIDTH
) (
    input logic clk,
    input logic [WIDTH-1:0] a,
    input logic [WIDTH-1:0] b,
    output logic [WIDTH:0] sum
);
    logic [WIDTH-1:0] a_q = 0;
    logic [WIDTH-1:0] b_q = 0;
    wire [8:0] full_sum;
    tiny_adder combinational (
        .a({{(8-WIDTH){1'b0}}, a_q}),
        .b({{(8-WIDTH){1'b0}}, b_q}),
        .sum(full_sum)
    );
    always_ff @(posedge clk) begin
        a_q <= a;
        b_q <= b;
        sum <= full_sum[WIDTH:0];
    end
endmodule
