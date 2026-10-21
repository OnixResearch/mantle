/* Process-entry metadata frame loader for the original mrustc source.
 * This is compiled into mrustc, never a wrapper or child-launching adapter. */
#ifndef MANTLE_CAIRN_MRUSTC_METADATA_H
#define MANTLE_CAIRN_MRUSTC_METADATA_H
#include <cstdint>
#include <climits>
#include <utility>
#include <cstdlib>
#include <fstream>
#include <stdexcept>
#include <vector>
#include <string>
#include <cerrno>
#include <sys/stat.h>

namespace cairn_metadata {
static const std::size_t FRAME_BYTES_MAX = 64u * 1024u * 1024u;
static std::vector<unsigned char> frame(const char* name) {
    const char* path = std::getenv(name);
    if(!path) return {};
    std::ifstream file(path, std::ios::binary | std::ios::ate);
    if(!file) throw std::runtime_error("cannot open mrustc metadata frame");
    const auto size = file.tellg();
    if(size < 0 || static_cast<unsigned long long>(size) > FRAME_BYTES_MAX)
        throw std::runtime_error("mrustc metadata frame exceeds bound");
    std::vector<unsigned char> bytes(static_cast<std::size_t>(size));
    file.seekg(0);
    if(!bytes.empty() && !file.read(reinterpret_cast<char*>(bytes.data()), bytes.size()))
        throw std::runtime_error("truncated mrustc metadata frame");
    return bytes;
}
static std::uint32_t length(const std::vector<unsigned char>& bytes, std::size_t& cursor) {
    if(cursor > bytes.size() || bytes.size() - cursor < 4)
        throw std::runtime_error("truncated mrustc metadata length");
    std::uint32_t value = 0;
    for(unsigned i = 0; i != 4; ++i) value |= std::uint32_t(bytes[cursor++]) << (8 * i);
    return value;
}
static std::string field(const std::vector<unsigned char>& bytes, std::size_t& cursor, std::size_t size) {
    if(cursor > bytes.size() || size > bytes.size() - cursor)
        throw std::runtime_error("truncated mrustc metadata field");
    std::string value(reinterpret_cast<const char*>(bytes.data() + cursor), size);
    cursor += size;
    if(value.find('\0') != std::string::npos)
        throw std::runtime_error("NUL in mrustc metadata field");
    return value;
}
static void load(int& argc, char**& argv) {
    if(const char* work = std::getenv("MRUSTC_CAIRN_MACRO_WORK")) {
        if(mkdir(work, 0700) && errno != EEXIST)
            throw std::runtime_error("cannot create declared compiler output work tree");
    }
    // Own all argument bytes until the compiler exits: original parser keeps
    // pointers into argv for -L and -l and modifies some option operands.
    static std::vector<std::string> arguments;
    static std::vector<char*> pointers;
    for(int i = 0; i < argc; ++i) arguments.emplace_back(argv[i]);
    auto bytes = frame("MRUSTC_CAIRN_ARGUMENTS");
    std::size_t cursor = 0;
    while(cursor < bytes.size()) {
        auto size = length(bytes, cursor);
        auto option = field(bytes, cursor, size);
        if(option != "--cfg" && option != "-L" && option != "-l")
            throw std::runtime_error("unsupported dynamic mrustc metadata option");
        size = length(bytes, cursor);
        auto operand = field(bytes, cursor, size);
        if(operand.empty() || operand[0] == '-')
            throw std::runtime_error("invalid dynamic mrustc metadata operand");
        arguments.push_back(std::move(option));
        arguments.push_back(std::move(operand));
    }
    bytes = frame("MRUSTC_CAIRN_COMPILE_ENV");
    cursor = 0;
    std::vector<std::pair<std::string, std::string>> variables;
    while(cursor < bytes.size()) {
        const auto key_size = length(bytes, cursor);
        const auto value_size = length(bytes, cursor);
        auto key = field(bytes, cursor, key_size);
        auto value = field(bytes, cursor, value_size);
        if(key.empty() || key.find('=') != std::string::npos || key.compare(0, 7, "MRUSTC_") == 0
            || key.compare(0, 7, "MANTLE_") == 0 || key == "CC" || key.compare(0, 3, "CC_") == 0
            || key == "LD_PRELOAD" || key == "LD_LIBRARY_PATH")
            throw std::runtime_error("invalid dynamic mrustc environment key");
        for(const auto& prior : variables) if(prior.first == key)
            throw std::runtime_error("duplicate dynamic mrustc environment key");
        variables.emplace_back(std::move(key), std::move(value));
    }
    for(const auto& variable : variables)
        if(setenv(variable.first.c_str(), variable.second.c_str(), 1))
            throw std::runtime_error("cannot load mrustc compile environment");
    if(arguments.size() > static_cast<std::size_t>(INT_MAX))
        throw std::runtime_error("mrustc metadata argument count overflow");
    for(auto& argument : arguments) pointers.push_back(&argument[0]);
    argc = static_cast<int>(pointers.size());
    pointers.push_back(nullptr);
    argv = pointers.data();
}
}
#endif
