/* Processless seam for the ORIGINAL mrustc token wire protocol.
 * The source compiler records complete genuine stdin and exits75; Mantle runs
 * the exact Produced proc-macro executable, then resumes against actual stdout.
 * No handshake byte, token, status or macro result is synthesized here. */
#ifndef MANTLE_CAIRN_PROC_MACRO_H
#define MANTLE_CAIRN_PROC_MACRO_H
#include <cerrno>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <initializer_list>
#include <memory>
#include <stdexcept>
#include <limits.h>
#include <string>
#include <sys/stat.h>
#include <unistd.h>
extern char **environ;

class CairnProcMacro {
    static const std::size_t BYTES_MAX = 64u * 1024u * 1024u;
    bool m_replay;
    std::size_t m_sent = 0;
    std::size_t m_received = 0;
    std::ifstream m_expected;
    std::ifstream m_response;
    std::ofstream m_request;
    static std::string text(const std::string& path) {
        std::ifstream file(path, std::ios::binary);
        if(!file) throw std::runtime_error("cannot read original macro request identity");
        std::string value;
        char buffer[4096];
        while(file.read(buffer, sizeof buffer) || file.gcount()) {
            if(value.size() + static_cast<std::size_t>(file.gcount()) > BYTES_MAX)
                throw std::runtime_error("macro identity exceeds bound");
            value.append(buffer, static_cast<std::size_t>(file.gcount()));
        }
        if(!file.eof()) throw std::runtime_error("cannot read complete macro identity");
        return value;
    }
    static void save(const std::string& path, const char* value) {
        std::ofstream file(path, std::ios::binary | std::ios::out | std::ios::trunc);
        if(!file || !file.write(value, std::strlen(value)) || !file.flush())
            throw std::runtime_error("cannot write actual macro request identity");
    }
    static void save_environment(const std::string& path) {
        std::ofstream file(path, std::ios::binary | std::ios::out | std::ios::trunc);
        if(!file) throw std::runtime_error("cannot record actual macro environment");
        std::size_t total = 0;
        for(char** variable = environ; *variable; ++variable) {
            const char* equals = std::strchr(*variable, '=');
            if(!equals || equals == *variable) throw std::runtime_error("invalid original macro environment");
            const auto key_size = static_cast<std::size_t>(equals - *variable);
            const auto value_size = std::strlen(equals + 1);
            if(key_size > BYTES_MAX || value_size > BYTES_MAX || key_size + value_size + 8 > BYTES_MAX - total)
                throw std::runtime_error("macro environment exceeds bound");
            total += key_size + value_size + 8;
            for(auto size : {key_size, value_size}) for(unsigned byte = 0; byte != 4; ++byte)
                file.put(static_cast<char>((size >> (8 * byte)) & 0xff));
            file.write(*variable, key_size);
            file.write(equals + 1, value_size);
        }
        if(!file.flush()) throw std::runtime_error("cannot flush actual macro environment");
    }
    static void check_environment(const std::string& path) {
        std::ifstream expected(path, std::ios::binary);
        if(!expected) throw std::runtime_error("missing actual macro environment history");
        for(char** variable = environ; *variable; ++variable) {
            const char* equals = std::strchr(*variable, '=');
            if(!equals) throw std::runtime_error("invalid resumed macro environment");
            const auto key_size = static_cast<std::size_t>(equals - *variable);
            const auto value_size = std::strlen(equals + 1);
            for(auto size : {key_size, value_size}) for(unsigned byte = 0; byte != 4; ++byte)
                if(expected.get() != static_cast<int>((size >> (8 * byte)) & 0xff))
                    throw std::runtime_error("compiler macro environment changed during resume");
            const char* parts[] = {*variable, equals + 1};
            for(const char* part : parts) {
                const auto size = part == *variable ? key_size : value_size;
                for(std::size_t i = 0; i != size; ++i)
                    if(expected.get() != static_cast<unsigned char>(part[i]))
                        throw std::runtime_error("compiler macro environment changed during resume");
            }
        }
        if(expected.get() != std::char_traits<char>::eof() || !expected.eof())
            throw std::runtime_error("compiler macro environment history is not complete");
    }
public:
    CairnProcMacro(const char* executable, const char* name) {
        static unsigned long ordinal = 0;
        const char* history = std::getenv("MRUSTC_CAIRN_MACRO_HISTORY");
        const char* work = std::getenv("MRUSTC_CAIRN_MACRO_WORK");
        if(!history || !work) throw std::runtime_error("missing macro phase namespaces");
        char cwd[PATH_MAX];
        if(!getcwd(cwd, sizeof cwd)) throw std::runtime_error("cannot record original macro working directory");
        const auto prior = std::string(history) + "/" + std::to_string(ordinal++);
        struct stat metadata;
        if(stat(prior.c_str(), &metadata) == 0) {
            if(!S_ISDIR(metadata.st_mode) || text(prior + "/executable") != executable
                || text(prior + "/macro-name") != name || text(prior + "/cwd") != cwd)
                throw std::runtime_error("compiler macro executable/name changed during resume");
            m_replay = true;
            m_expected.open(prior + "/stdin", std::ios::binary);
            m_response.open(prior + "/stdout", std::ios::binary);
            if(!m_expected || !m_response) throw std::runtime_error("missing actual macro execution history");
            check_environment(prior + "/environment");
        } else {
            if(errno != ENOENT) throw std::runtime_error("cannot inspect macro execution history");
            m_replay = false;
            // This is the source compiler's declared output tree, not /tmp or
            // another process's output. A new request never overwrites a prior.
            if(mkdir(work, 0700) && errno != EEXIST) throw std::runtime_error("cannot create compiler phase work tree");
            const auto request = std::string(work) + "/macro-request";
            if(mkdir(request.c_str(), 0700)) throw std::runtime_error("macro request already exists");
            save(request + "/executable", executable);
            save(request + "/macro-name", name);
            save(request + "/cwd", cwd);
            save_environment(request + "/environment");
            m_request.open(request + "/stdin", std::ios::binary | std::ios::out | std::ios::trunc);
            if(!m_request) throw std::runtime_error("cannot record actual macro input");
        }
    }
    void send(const void* bytes, std::size_t size) {
        if(size > BYTES_MAX - m_sent) throw std::runtime_error("macro stdin exceeds bound");
        m_sent += size;
        if(m_replay) {
            const auto* source = static_cast<const unsigned char*>(bytes);
            for(std::size_t i = 0; i != size; ++i) {
                const auto actual = m_expected.get();
                if(actual == std::char_traits<char>::eof() || static_cast<unsigned char>(actual) != source[i])
                    throw std::runtime_error("compiler macro stdin changed during resume");
            }
        } else if(!m_request.write(static_cast<const char*>(bytes), size)) {
            throw std::runtime_error("cannot write complete macro stdin");
        }
    }
    void finish() {
        if(m_replay) {
            if(m_expected.get() != std::char_traits<char>::eof() || !m_expected.eof())
                throw std::runtime_error("compiler macro stdin no longer complete");
        } else {
            m_request.flush();
            if(!m_request) throw std::runtime_error("cannot flush complete macro stdin");
            m_request.close();
            if(!m_request) throw std::runtime_error("cannot close complete macro stdin");
            std::exit(75);
        }
    }
    bool check_good() {
        // Read the ORIGINAL macro executable's observed startup byte.
        const auto value = m_response.get();
        if(value != std::char_traits<char>::eof()) ++m_received;
        return value == 0;
    }
    void receive(void* bytes, std::size_t size) {
        if(size > BYTES_MAX - m_received) throw std::runtime_error("macro stdout exceeds bound");
        m_received += size;
        if(!m_replay || !m_response.read(static_cast<char*>(bytes), size))
            throw std::runtime_error("unexpected end of actual macro stdout");
    }
};
#endif
