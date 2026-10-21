/* Source-owned continuation boundary for the original CMake evaluator.
 * No command is launched here. A missing result suspends the original evaluator
 * with the exact request; a present result is admitted only after byte-for-byte
 * request matching. Results are supplied by separately declared native leaves.
 */
#pragma once
#include <cerrno>
#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fcntl.h>
#include <dirent.h>
#include <limits>
#include <string>
#include <vector>
#include <sys/stat.h>
#include <unistd.h>
#include <time.h>
extern char** environ;
#include "cmUVHandlePtr.h"

namespace mantle_cmake_phase {
constexpr std::size_t Limit = 64u * 1024u * 1024u;
[[noreturn]] inline void fail(const char* message)
{
  std::fprintf(stderr, "CMake source phase: %s\n", message);
  std::fflush(stderr);
  std::_Exit(78);
}
inline bool active() { return std::getenv("MANTLE_CAIRN_CMAKE_PHASE") != nullptr; }
inline void bytes(std::string& to, const std::string& value)
{
  if (value.size() > Limit - 4 || to.size() > Limit - value.size() - 4)
    fail("frame exceeds source-phase bound");
  auto n = static_cast<std::uint32_t>(value.size());
  for (unsigned i = 0; i != 4; ++i) to.push_back(static_cast<char>(n >> (8*i)));
  to.append(value);
}
inline void number(std::string& to, std::uint32_t n)
{
  for (unsigned i = 0; i != 4; ++i) to.push_back(static_cast<char>(n >> (8*i)));
}
inline std::uint32_t number(const std::string& from, std::size_t& at)
{
  if (at > from.size() || from.size() - at < 4) fail("truncated integer");
  std::uint32_t n = 0;
  for (unsigned i = 0; i != 4; ++i)
    n |= static_cast<std::uint32_t>(static_cast<unsigned char>(from[at++])) << (8*i);
  return n;
}
inline std::string bytes(const std::string& from, std::size_t& at)
{
  auto n = number(from, at);
  if (n > Limit || at > from.size() || from.size() - at < n) fail("truncated byte frame");
  auto result = from.substr(at, n);
  at += n;
  return result;
}
inline std::string read(const std::string& path)
{
  FILE* f = std::fopen(path.c_str(), "rb");
  if (!f) fail("cannot open declared result file");
  std::string result;
  char buffer[16384];
  for (;;) {
    auto n = std::fread(buffer, 1, sizeof buffer, f);
    if (result.size() > Limit - n) fail("result exceeds source-phase bound");
    result.append(buffer, n);
    if (n != sizeof buffer) {
      if (std::ferror(f)) fail("cannot read declared result file");
      break;
    }
  }
  if (std::fclose(f)) fail("cannot close declared result file");
  return result;
}
inline void write(const std::string& path, const std::string& value)
{
  FILE* f = std::fopen(path.c_str(), "wb");
  if (!f) fail("cannot open declared phase output");
  if (std::fwrite(value.data(), 1, value.size(), f) != value.size())
    fail("cannot write declared phase output");
  if (std::fclose(f)) fail("cannot close declared phase output");
}
inline void restore_file(const std::string& source, const std::string& destination)
{
  struct stat st;
  if (::lstat(source.c_str(), &st) || !S_ISREG(st.st_mode)) fail("effect blob is not regular");
  FILE* from = std::fopen(source.c_str(), "rb");
  FILE* to = std::fopen(destination.c_str(), "wb");
  if (!from || !to) fail("cannot open original file effect");
  char buffer[65536];
  for (;;) {
    auto n = std::fread(buffer, 1, sizeof buffer, from);
    if (std::fwrite(buffer, 1, n, to) != n) fail("cannot restore file effect bytes");
    if (n != sizeof buffer) { if (std::ferror(from)) fail("cannot read file effect bytes"); break; }
  }
  if (std::fclose(from) || std::fclose(to)) fail("cannot close original file effect");
}
inline std::string cwd()
{
  std::vector<char> buffer(4096);
  for (;;) {
    if (::getcwd(buffer.data(), buffer.size())) return buffer.data();
    if (errno != ERANGE || buffer.size() >= 1024u*1024u) fail("cannot resolve phase cwd");
    buffer.resize(buffer.size()*2);
  }
}
inline bool relative(const std::string& path)
{
  if (path.empty() || path.front() == '/' || path.find('\0') != std::string::npos) return false;
  std::size_t at = 0;
  for (;;) {
    auto end = path.find('/', at);
    auto part = path.substr(at, end == std::string::npos ? end : end-at);
    if (part.empty() || part == "." || part == "..") return false;
    if (end == std::string::npos) return true;
    at = end+1;
  }
}
inline void mkdirs(const std::string& path)
{
  for (std::size_t at = 1; (at = path.find('/', at)) != std::string::npos; ++at) {
    auto prefix = path.substr(0, at);
    if (::mkdir(prefix.c_str(), 0700) && errno != EEXIST) fail("cannot restore result directory");
    struct stat st;
    if (::lstat(prefix.c_str(), &st) || !S_ISDIR(st.st_mode)) fail("non-directory result ancestor");
  }
}
struct Result {
  std::vector<int> codes;
  std::string out;
  std::string err;
  std::uint32_t inputConsumed = 0;
};
struct State {
  std::string history;
  std::uint32_t ordinal = 0;
  std::uint32_t records = 0;
  State()
  {
    if (!active()) return;
    const char* path = std::getenv("MANTLE_CAIRN_CMAKE_HISTORY");
    if (!path) fail("missing declared history input");
    history = path;
    if (read(history + "/format") != "MCCMH03\n") fail("invalid phase history format");
    DIR* directory = ::opendir(history.c_str());
    if (!directory) fail("cannot inspect phase history tree");
    while (auto* entry = ::readdir(directory)) {
      std::string name = entry->d_name;
      if (name == "." || name == ".." || name == "format" || name == "effects") continue;
      if (name.empty() || (name.size() > 1 && name.front() == '0')) fail("noncanonical history ordinal");
      std::uint64_t value = 0;
      for (auto c : name) {
        if (c < '0' || c > '9' || value > 100000) fail("invalid history ordinal");
        value = value*10 + static_cast<unsigned>(c-'0');
      }
      if (value >= 100000 || ++records > 100000) fail("history exceeds action bound");
      struct stat st;
      if (::lstat((history + "/" + name).c_str(), &st) || !S_ISREG(st.st_mode))
        fail("history record is not a regular Produced observation");
    }
    if (::closedir(directory)) fail("cannot close phase history tree");
    for (std::uint32_t i = 0; i < records; ++i) {
      struct stat st;
      if (::lstat((history + "/" + std::to_string(i)).c_str(), &st)) fail("history has an ordinal gap");
    }
  }
};
inline State& state() { static State s; return s; }
inline std::string observation_path()
{
  static std::uint32_t ordinal = 0;
  const char* root = std::getenv("MANTLE_CAIRN_CMAKE_OBSERVATIONS");
  if (!root || std::strncmp(root, "/outputs/", 9) || ordinal >= 100000)
    fail("missing or exhausted original observation tree");
  return std::string(root) + "/" + std::to_string(ordinal++);
}
inline bool observed(const std::string& path)
{
  struct stat st;
  if (::lstat(path.c_str(), &st)) {
    if (errno == ENOENT) return false;
    fail("cannot inspect original nondeterministic observation");
  }
  if (!S_ISREG(st.st_mode)) fail("original observation is not regular");
  return true;
}
template <class Original> inline unsigned int random_seed(Original original)
{
  if (!active()) return original();
  auto path = observation_path();
  if (observed(path)) {
    auto record = read(path);
    std::size_t at = 0;
    if (bytes(record, at) != "random-seed") fail("original random observation drift");
    auto value = number(record, at);
    if (at != record.size()) fail("trailing random observation");
    return value;
  }
  auto value = original();
  std::string record;
  bytes(record, "random-seed"); number(record, value);
  write(path, record);
  return value;
}
template <class Original> inline auto temporary_directory(char* destination, const mode_t* mode, Original original)
  -> decltype(original())
{
  if (!active() || !destination) return original();
  using Status = decltype(original());
  auto path = observation_path();
  std::string requested = destination;
  auto requestedMode = mode ? static_cast<std::uint32_t>(*mode) : 0xffffffffu;
  if (observed(path)) {
    auto record = read(path); std::size_t at = 0;
    if (bytes(record, at) != "temporary-directory" || bytes(record, at) != requested ||
        number(record, at) != requestedMode) fail("original temporary observation drift");
    auto error = number(record, at);
    auto actual = bytes(record, at);
    auto actualMode = number(record, at);
    if (at != record.size() || actual.size() != requested.size() || (actualMode & ~0777u))
      fail("invalid original temporary observation");
    if (error) return Status::POSIX(static_cast<int>(error));
    if (requested.size() < 6 || requested.compare(requested.size()-6, 6, "XXXXXX") ||
        actual.compare(0, actual.size()-6, requested, 0, requested.size()-6))
      fail("original temporary name escapes its request");
    mkdirs(actual);
    if (::mkdir(actual.c_str(), actualMode)) fail("cannot restore original temporary directory");
    std::memcpy(destination, actual.c_str(), actual.size()+1);
    return Status::Success();
  }
  auto status = original();
  auto error = status ? 0u : static_cast<unsigned>(errno);
  if (!status && !error) fail("original temporary failure lacks actual POSIX error");
  struct stat st{};
  if (status && (::lstat(destination, &st) || !S_ISDIR(st.st_mode))) fail("original temporary output is not a directory");
  std::string record;
  bytes(record, "temporary-directory"); bytes(record, requested); number(record, requestedMode);
  number(record, error); bytes(record, destination); number(record, static_cast<std::uint32_t>(st.st_mode & 0777));
  write(path, record);
  return status;
}
inline std::string output(const char* suffix)
{
  const char* root = std::getenv("MANTLE_CAIRN_CMAKE_PHASE");
  if (!root || root[0] != '/') fail("missing absolute phase output root");
  return std::string(root) + "/" + suffix;
}


inline void finish()
{
  if (!active()) return;
  auto& s = state();
  if (s.ordinal != s.records) fail("unconsumed result history after original evaluation");
  write(output("state"), "complete\n");
}
struct Initializer {
  Initializer() { if (active() && std::atexit(finish)) fail("cannot register phase completion"); }
};
inline Initializer initializer;

/* One bounded history file per actual request. Frames contain original request,
 * actual status vector/stdout/stderr/stdin consumption, then typed filesystem effects. A tree of
 * records avoids collecting the entire configure transcript in one allocation.
 * Effect tags: 0 delete, 1 regular file, 2 directory, 3 relative symlink.
 */
inline Result exchange(const std::string& request, std::size_t codeCount)
{
  auto& s = state();
  if (s.ordinal == s.records) {
    std::string framed("MCCMQ01\n", 8);
    number(framed, s.ordinal);
    bytes(framed, request);
    write(output("request"), framed);
    write(output("state"), "pending\n");
    std::fflush(nullptr);
    std::_Exit(75);
  }
  auto history = read(s.history + "/" + std::to_string(s.ordinal));
  if (history.compare(0, 8, "MCCMR03\n")) fail("invalid history record format");
  std::size_t position = 8;
  auto prior = bytes(history, position);
  if (request != prior) fail("original evaluator request drift during replay");
  Result r;
  auto n = number(history, position);
  if (n != codeCount || n == 0 || n > 64) fail("result command cardinality mismatch");
  r.codes.reserve(n);
  while (n--) {
    auto code = number(history, position);
    if (code > 255) fail("result is not an actual normal exit code");
    r.codes.push_back(static_cast<int>(code));
  }
  r.out = bytes(history, position);
  r.err = bytes(history, position);
  r.inputConsumed = number(history, position);
  auto effects = number(history, position);
  if (effects > 65536) fail("result effect cardinality exceeds bound");
  const char* workspace = std::getenv("MANTLE_CAIRN_CMAKE_WORKSPACE");
  if (!workspace || std::strncmp(workspace, "/outputs/", 9))
    fail("result workspace is not a declared output tree");
  while (effects--) {
    auto tag = number(history, position);
    auto path = bytes(history, position);
    auto contents = bytes(history, position);
    auto mode = number(history, position);
    auto secondsLow = number(history, position);
    auto secondsHigh = number(history, position);
    auto nanoseconds = number(history, position);
    if (nanoseconds >= 1000000000u) fail("invalid original effect timestamp");
    if (!relative(path) || (mode & ~0777u) || tag > 3) fail("invalid result effect path/mode/tag");
    auto destination = std::string(workspace) + "/" + path;
    mkdirs(destination);
    struct stat st;
    bool present = ::lstat(destination.c_str(), &st) == 0;
    if (!present && errno != ENOENT) fail("cannot inspect result effect destination");
    if (tag == 0) {
      if (!contents.empty()) fail("deletion effect has contents");
      if (present && (S_ISDIR(st.st_mode) ? ::rmdir(destination.c_str()) : ::unlink(destination.c_str())))
        fail("cannot restore original deletion");
    } else if (tag == 2) {
      if (!contents.empty()) fail("directory effect has contents");
      if (present && !S_ISDIR(st.st_mode)) {
        if (::unlink(destination.c_str())) fail("cannot replace directory destination");
        present = false;
      }
      if (!present && ::mkdir(destination.c_str(), 0700)) fail("cannot restore original directory");
      if (::chmod(destination.c_str(), mode)) fail("cannot restore original directory mode");
    } else {
      if (present && (S_ISDIR(st.st_mode) ? ::rmdir(destination.c_str()) : ::unlink(destination.c_str())))
        fail("cannot replace original effect destination");
      if (tag == 1) {
        auto prefix = "effects/" + std::to_string(s.ordinal) + "/";
        if (!relative(contents) || contents.compare(0, prefix.size(), prefix)) fail("file effect blob has wrong observation owner");
        restore_file(s.history + "/" + contents, destination);
        if (::chmod(destination.c_str(), mode)) fail("cannot restore result mode");
      } else {
        if (contents.empty() || contents.front() == '/' || contents.find('\0') != std::string::npos)
          fail("invalid original symlink destination");
        std::size_t depth = 0;
        for (auto c : path) if (c == '/') ++depth;
        std::size_t at = 0;
        for (;;) {
          auto end = contents.find('/', at);
          auto component = contents.substr(at, end == std::string::npos ? end : end-at);
          if (component == "..") { if (!depth) fail("original symlink escapes workspace"); --depth; }
          else if (!component.empty() && component != ".") ++depth;
          if (end == std::string::npos) break;
          at = end+1;
        }
        if (::symlink(contents.c_str(), destination.c_str())) fail("cannot restore original symlink");
      }
    }
    if (tag) {
      auto seconds = (static_cast<std::uint64_t>(secondsHigh) << 32) | secondsLow;
      struct timespec times[2] = {{static_cast<time_t>(seconds), static_cast<long>(nanoseconds)},
                                 {static_cast<time_t>(seconds), static_cast<long>(nanoseconds)}};
      if (::utimensat(AT_FDCWD, destination.c_str(), times, AT_SYMLINK_NOFOLLOW))
        fail("cannot restore original effect timestamp");
    }
  }
  if (position != history.size()) fail("trailing history observation bytes");
  ++s.ordinal;
  return r;
}
inline std::string request(const std::string& kind, const std::vector<std::vector<std::string>>& commands,
                           const std::string& directory, const std::string& input)
{
  std::string r;
  bytes(r, kind);
  bytes(r, directory.empty() ? cwd() : directory);
  number(r, static_cast<std::uint32_t>(commands.size()));
  for (auto const& command : commands) {
    if (command.empty() || command.size() > 65536) fail("invalid original command cardinality");
    number(r, static_cast<std::uint32_t>(command.size()));
    for (auto const& arg : command) bytes(r, arg);
  }
  bytes(r, input);
  std::vector<std::string> environment;
  for (auto** entry = ::environ; *entry; ++entry) environment.emplace_back(*entry);
  std::sort(environment.begin(), environment.end());
  if (environment.size() > 65536) fail("original environment cardinality exceeds phase bound");
  number(r, static_cast<std::uint32_t>(environment.size()));
  for (auto const& entry : environment) bytes(r, entry);
  return r;
}
inline std::string input(int descriptor)
{
  if (descriptor < 0) return {};
  struct stat st;
  if (::fstat(descriptor, &st)) fail("cannot inspect original input stream");
  /* The sandbox's inherited stdin is /dev/null. Other nonseekable streams
   * require an explicit pipeline leaf rather than being consumed on replay. */
  if (S_ISCHR(st.st_mode)) return {};
  if (!S_ISREG(st.st_mode)) fail("original input requires explicit pipeline lowering");
  auto at = ::lseek(descriptor, 0, SEEK_CUR);
  if (at < 0 || st.st_size < at || static_cast<std::uint64_t>(st.st_size-at) > Limit)
    fail("invalid original input stream extent");
  std::string result(static_cast<std::size_t>(st.st_size-at), '\0');
  std::size_t done = 0;
  while (done != result.size()) {
    auto n = ::pread(descriptor, result.data()+done, result.size()-done, at+done);
    if (n < 0 && errno == EINTR) continue;
    if (n <= 0) fail("cannot read original input stream");
    done += static_cast<std::size_t>(n);
  }
  return result;
}
inline void consume_input(int descriptor, std::uint32_t consumed, std::size_t available)
{
  if (consumed > available || (consumed && descriptor < 0))
    fail("original child stdin consumption exceeds its request");
  if (consumed && ::lseek(descriptor, static_cast<off_t>(consumed), SEEK_CUR) < 0)
    fail("cannot restore original child stdin position");
}
/* The injected cmUVProcessChainBuilder::Start branch calls this only after
 * original Prepare has allocated its streams. uv_write retains ownership until
 * the loop consumes all bytes; no blocking pipe write and no helper child.
 */
struct Write {
  uv_write_t operation;
  std::string data;
  cm::uv_pipe_ptr pipe;
};
inline void stream(cm::uv_pipe_ptr& pipe, int descriptor, bool builtin, std::string data)
{
  if (builtin) {
    if (data.empty()) return;
    auto* pending = new Write{};
    pending->data = std::move(data);
    pending->pipe = std::move(pipe);
    pending->operation.data = pending;
    uv_buf_t buffer = uv_buf_init(pending->data.data(), static_cast<unsigned>(pending->data.size()));
    int rc = uv_write(&pending->operation, pending->pipe, &buffer, 1, [](uv_write_t* op, int status) {
      auto* pending = static_cast<Write*>(op->data);
      delete pending;
      if (status < 0) fail("cannot replay original captured stream");
    });
    if (rc < 0) { delete pending; fail("cannot queue original captured stream"); }
  } else if (descriptor >= 0) {
    std::size_t done = 0;
    while (done != data.size()) {
      auto n = ::write(descriptor, data.data()+done, data.size()-done);
      if (n < 0 && errno == EINTR) continue;
      if (n <= 0) fail("cannot restore original redirected stream");
      done += static_cast<std::size_t>(n);
    }
  }
}
} // namespace mantle_cmake_phase
