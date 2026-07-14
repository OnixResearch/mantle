#include "Vtiny_adder.h"
#include "verilated.h"

#include <charconv>
#include <iostream>
#include <string_view>

unsigned reference_add(unsigned a, unsigned b);

namespace {
bool parse_input(std::string_view text, unsigned& a, unsigned& b) {
    const auto comma = text.find(',');
    if (comma == std::string_view::npos) {
        return false;
    }
    const auto left = text.substr(0, comma);
    const auto right = text.substr(comma + 1U);
    const auto left_result = std::from_chars(left.data(), left.data() + left.size(), a);
    const auto right_result = std::from_chars(right.data(), right.data() + right.size(), b);
    return left_result.ec == std::errc{} && right_result.ec == std::errc{};
}
}  // namespace

int main(int argc, char** argv) {
    constexpr int expected_argument_count = 2;
    if (argc != expected_argument_count) {
        std::cerr << "expected one a,b vector\n";
        return 2;
    }

    unsigned a = 0;
    unsigned b = 0;
    if (!parse_input(argv[1], a, b)) {
        std::cerr << "invalid a,b vector\n";
        return 3;
    }

    Vtiny_adder dut;
    dut.a = a;
    dut.b = b;
    dut.eval();
    const unsigned observed = dut.sum;
    const unsigned expected = reference_add(a, b);
    if (observed != expected) {
        std::cerr << "reference mismatch expected=" << expected << " observed=" << observed << '\n';
        return 4;
    }
    std::cout << observed << '\n';
    return 0;
}
