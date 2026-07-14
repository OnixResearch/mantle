module tiny_adder_tb;
    logic [7:0] a;
    logic [7:0] b;
    logic [8:0] sum;

    tiny_adder dut (.a(a), .b(b), .sum(sum));
endmodule
