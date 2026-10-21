/* Source-owned compiler phase transformation. No subprocesses or generated
 * compiler/object/provider inputs. argv: original-tree link-module env-module
 * output-tree [final-overlay-module]. Compile with Produced native tools. */
#define _POSIX_C_SOURCE 200809L
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static void fail(const char *message) {
    fprintf(stderr, "Cairn compiler source phase: %s: %s\n", message, strerror(errno));
    exit(1);
}
static void join(char *output, const char *directory, const char *member) {
    int count = snprintf(output, PATH_MAX, "%s/%s", directory, member);
    if (count < 0 || count >= PATH_MAX) { errno = ENAMETOOLONG; fail("source path"); }
}
static void write_all(int output, const void *bytes, size_t length) {
    const unsigned char *cursor = bytes;
    while (length) {
        ssize_t count = write(output, cursor, length);
        if (count < 0 && errno == EINTR) continue;
        if (count <= 0) fail("write source");
        cursor += count;
        length -= (size_t)count;
    }
}
static void copy_file(const char *input, const char *output, mode_t mode) {
    int source = open(input, O_RDONLY | O_NOFOLLOW);
    if (source < 0) fail(input);
    int destination = open(output, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, mode & 0777);
    if (destination < 0) fail(output);
    unsigned char buffer[65536];
    for (;;) {
        ssize_t count = read(source, buffer, sizeof(buffer));
        if (count < 0 && errno == EINTR) continue;
        if (count < 0) fail("read original source");
        if (!count) break;
        write_all(destination, buffer, (size_t)count);
    }
    if (close(source) || close(destination)) fail("close source");
}
static void copy_tree(const char *input, const char *output) {
    struct stat metadata;
    if (lstat(input, &metadata)) fail(input);
    if (S_ISREG(metadata.st_mode)) {
        copy_file(input, output, metadata.st_mode);
    } else if (S_ISDIR(metadata.st_mode)) {
        if (mkdir(output, 0700)) fail(output);
        DIR *directory = opendir(input);
        if (!directory) fail(input);
        struct dirent *entry;
        errno = 0;
        while ((entry = readdir(directory))) {
            if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
            char source[PATH_MAX], destination[PATH_MAX];
            join(source, input, entry->d_name);
            join(destination, output, entry->d_name);
            copy_tree(source, destination);
            errno = 0;
        }
        if (errno || closedir(directory)) fail("read source directory");
        if (chmod(output, metadata.st_mode & 0777)) fail("source directory mode");
    } else if (S_ISLNK(metadata.st_mode)) {
        char target[PATH_MAX];
        ssize_t count = readlink(input, target, sizeof(target) - 1);
        if (count < 0 || (size_t)count == sizeof(target) - 1) fail("read source symlink");
        target[count] = 0;
        if (symlink(target, output)) fail("copy source symlink");
    } else {
        errno = EINVAL;
        fail("unsupported original source entry");
    }
}
static char *read_source(const char *path) {
    int file = open(path, O_RDONLY | O_NOFOLLOW);
    struct stat metadata;
    if (file < 0 || fstat(file, &metadata)) fail(path);
    if (!S_ISREG(metadata.st_mode) || metadata.st_size < 0 || metadata.st_size > 16 * 1024 * 1024) {
        errno = EFBIG; fail("compiler source bound");
    }
    size_t length = (size_t)metadata.st_size;
    char *source = malloc(length + 1);
    if (!source) fail("allocate compiler source");
    size_t cursor = 0;
    while (cursor < length) {
        ssize_t count = read(file, source + cursor, length - cursor);
        if (count < 0 && errno == EINTR) continue;
        if (count <= 0) fail("truncated compiler source");
        cursor += (size_t)count;
    }
    if (close(file)) fail("close compiler source");
    if (memchr(source, 0, length)) { errno = EINVAL; fail("NUL in compiler source"); }
    source[length] = 0;
    return source;
}
static void replace_once(char **source, const char *anchor, const char *replacement) {
    char *position = strstr(*source, anchor);
    if (!position || strstr(position + strlen(anchor), anchor)) {
        errno = EINVAL; fail("compiler source anchor absent or ambiguous");
    }
    size_t before = (size_t)(position - *source);
    size_t after = strlen(position + strlen(anchor));
    size_t inserted = strlen(replacement);
    char *result = malloc(before + inserted + after + 1);
    if (!result) fail("allocate source rewrite");
    memcpy(result, *source, before);
    memcpy(result + before, replacement, inserted);
    memcpy(result + before + inserted, position + strlen(anchor), after + 1);
    free(*source);
    *source = result;
}
static void save(const char *path, const char *source) {
    int file = open(path, O_WRONLY | O_TRUNC | O_NOFOLLOW);
    if (file < 0) fail(path);
    write_all(file, source, strlen(source));
    if (close(file)) fail("close patched source");
}
int main(int argc, char **argv) {
    if (argc == 4 && !strcmp(argv[1], "--select")) {
        struct stat metadata;
        if (lstat(argv[2], &metadata) || !S_ISREG(metadata.st_mode)) {
            errno = EINVAL; fail("selected runtime member must be a regular file");
        }
        copy_file(argv[2], argv[3], metadata.st_mode);
        return 0;
    }
    if (argc != 5 && argc != 6) { errno = EINVAL; fail("expected original-tree link-module env-module output-tree [final-overlay-module]"); }
    copy_tree(argv[1], argv[4]);
    char original[PATH_MAX], output[PATH_MAX];
    const char *link = "compiler/rustc_codegen_ssa/src/back/link.rs";
    join(original, argv[1], link);
    char *source = read_source(original);
    replace_once(&source, "mod raw_dylib;\n", "mod raw_dylib;\nmod cairn_link_phase;\n");
    replace_once(&source, "    // Invoke the system linker\n",
        "    if cairn_link_phase::emit(sess, &cmd, out_filename, flavor) {\n"
        "        return;\n"
        "    }\n\n"
        "    // Invoke the system linker\n");
    join(output, argv[4], link);
    save(output, source);
    free(source);
    join(output, argv[4], "compiler/rustc_codegen_ssa/src/back/link/cairn_link_phase.rs");
    copy_file(argv[2], output, 0644);
    const char *command = "compiler/rustc_codegen_ssa/src/back/command.rs";
    join(original, argv[1], command);
    source = read_source(original);
    replace_once(&source, "    pub(crate) fn command(&self) -> process::Command {\n",
        "    pub(crate) fn cairn_phase_command_is(&self, expected: &OsStr) -> bool {\n"
        "        matches!(&self.program, Program::Normal(program) if program == expected)\n"
        "            && !self.env_clear && self.env_remove.is_empty()\n"
        "            && self.env.iter().all(|(key, value)| {\n"
        "                (key == \"LC_ALL\" && value == \"C\")\n"
        "                    || (key == \"VSLANG\" && value == \"1033\")\n"
        "            })\n"
        "    }\n\n"
        "    pub(crate) fn cairn_phase_environment(&self) -> impl Iterator<Item = (&OsStr, &OsStr)> + '_ {\n"
        "        self.env.iter().enumerate().filter(move |(index, (key, _))| !self.env[*index + 1..].iter().any(|(later, _)| later == key)).map(|(_, (key, value))| (key.as_os_str(), value.as_os_str()))\n"
        "    }\n\n"
        "    pub(crate) fn command(&self) -> process::Command {\n");
    join(output, argv[4], command);
    save(output, source);
    free(source);
    const char *driver = "compiler/rustc_driver_impl/src/lib.rs";
    join(original, argv[1], driver);
    source = read_source(original);
    replace_once(&source, "pub fn main() -> ! {\n",
        "mod cairn_phase_env;\n\n"
        "pub fn main() -> ! {\n"
        "    if let Err(error) = cairn_phase_env::load() {\n"
        "        eprintln!(\"Cairn compiler environment: {error}\");\n"
        "        std::process::exit(1);\n"
        "    }\n");
    if (argc == 6) {
        replace_once(&source, "mod cairn_phase_env;\n", "mod cairn_phase_env;\nmod cairn_source_overlay;\n");
        replace_once(&source, "        file_loader: None,\n", "        file_loader: cairn_source_overlay::loader(),\n");
    }
    join(output, argv[4], driver);
    save(output, source);
    free(source);
    join(output, argv[4], "compiler/rustc_driver_impl/src/cairn_phase_env.rs");
    copy_file(argv[3], output, 0644);
    if (argc == 6) {
        join(output, argv[4], "compiler/rustc_driver_impl/src/cairn_source_overlay.rs");
        copy_file(argv[5], output, 0644);
    }
    return 0;
}
