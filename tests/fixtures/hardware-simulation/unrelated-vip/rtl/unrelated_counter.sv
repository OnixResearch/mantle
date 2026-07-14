module unrelated_counter (
    input logic clock,
    output logic [31:0] count
);
    always_ff @(posedge clock) begin
        count <= count + 1'b1;
    end
endmodule
