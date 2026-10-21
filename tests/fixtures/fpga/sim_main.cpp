#include "Vregistered_adder.h"
#include "verilated.h"
#include <cstdio>

int main(int argc, char** argv) {
    Verilated::commandArgs(argc, argv);
    Vregistered_adder model;
    for (unsigned a = 0; a < 4; ++a) {
        for (unsigned b = 0; b < 4; ++b) {
            model.a = a;
            model.b = b;
            // Two rising edges include input and output registers.
            for (unsigned cycle = 0; cycle < 2; ++cycle) {
                model.clk = 0; model.eval();
                model.clk = 1; model.eval();
            }
            if (model.sum != a + b) {
                std::fprintf(stderr, "registered-adder mismatch %u+%u: %u\n", a, b, unsigned(model.sum));
                return 4;
            }
        }
    }
    model.final();
    std::puts("registered-adder: 16 input pairs passed after two clocks");
    return 0;
}
