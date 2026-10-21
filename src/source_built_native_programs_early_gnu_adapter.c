/* Source-owned native compatibility adapter. Verify original installed members;
 * explicit phase mode also verifies its real shared runtime variant before
 * copying that variant. No executable discovery or child process creation. */
#define _GNU_SOURCE 1
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static void fail(const char *message)
{
    fprintf(stderr, "native StageX adapter: %s\n", message);
    exit(1);
}

static int member_open(const char *absolute)
{
    int directory, next;
    const char *part, *end;
    char component[256];
    size_t size;
    if (*absolute != '/' || !absolute[1]) fail("invalid typed member path");
    directory = open("/", O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC);
    if (directory < 0) fail("opening original transition root");
    part = absolute + 1;
    for (;;) {
        end = strchr(part, '/');
        size = end ? (size_t)(end - part) : strlen(part);
        if (!size || size >= sizeof(component)) fail("invalid member component");
        memcpy(component, part, size);
        component[size] = 0;
        if (!strcmp(component, ".") || !strcmp(component, "..")) fail("member escapes original transition");
        next = openat(directory, component, O_RDONLY | O_NOFOLLOW | O_CLOEXEC | (end ? O_DIRECTORY : 0));
        close(directory);
        if (next < 0) fail("opening original transition member");
        if (!end) return next;
        directory = next;
        part = end + 1;
    }
}

static ssize_t read_chunk(int fd, unsigned char *bytes, size_t capacity)
{
    size_t used = 0;
    ssize_t count;
    while (used < capacity) {
        count = read(fd, bytes + used, capacity - used);
        if (count < 0 && errno == EINTR) continue;
        if (count < 0) fail("reading installed member");
        if (!count) break;
        used += (size_t)count;
    }
    return (ssize_t)used;
}

static void write_chunk(int fd, const unsigned char *bytes, size_t size)
{
    ssize_t count;
    while (size) {
        count = write(fd, bytes, size);
        if (count < 0 && errno == EINTR) continue;
        if (count <= 0) fail("writing installed member");
        bytes += count;
        size -= (size_t)count;
    }
}

static void verify_raw_executable(const char *member, const char *canonical,
                                  unsigned char *first, unsigned char *second, size_t capacity)
{
    int source = member_open(member), original = member_open(canonical);
    struct stat a, b;
    ssize_t n, m;
    if (fstat(source, &a) || fstat(original, &b) || !S_ISREG(a.st_mode) || !S_ISREG(b.st_mode)
        || a.st_size != b.st_size || !(a.st_mode & 0111) || !(b.st_mode & 0111))
        fail("raw transition executable mismatch");
    do {
        n = read_chunk(source, first, capacity);
        m = read_chunk(original, second, capacity);
        if (n != m || memcmp(first, second, (size_t)n)) fail("raw transition executable bytes mismatch");
    } while (n);
    if (close(source) || close(original)) fail("closing raw transition executable");
}

int main(int argc, char **argv)
{
    int source, original, destination, executable;
    struct stat a, b;
    unsigned char first[16384], second[16384];
    ssize_t n, m;
    if (argc == 7 && !strcmp(argv[4], "phase-executable")) {
        verify_raw_executable(argv[5], argv[6], first, second, sizeof(first));
        executable = 1;
    } else {
        if (argc != 5) fail("expected member, canonical member, output, mode");
        executable = !strcmp(argv[4], "executable");
        if (!executable && strcmp(argv[4], "data")) fail("invalid install mode");
    }
    source = member_open(argv[1]);
    original = member_open(argv[2]);
    if (source < 0 || fstat(source, &a) || fstat(original, &b)) fail("opening member inputs");
    if (!S_ISREG(a.st_mode) || !S_ISREG(b.st_mode) || a.st_size != b.st_size) fail("original transition member mismatch");
    if (executable && (!(a.st_mode & 0111) || !(b.st_mode & 0111))) fail("non-executable original tool");
    destination = open(argv[3], O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC, 0600);
    if (destination < 0) fail("creating installed output");
    for (;;) {
        n = read_chunk(source, first, sizeof(first));
        m = read_chunk(original, second, sizeof(second));
        if (n != m || memcmp(first, second, (size_t)n)) fail("original transition bytes mismatch");
        if (!n) break;
        write_chunk(destination, first, (size_t)n);
    }
    if (fchmod(destination, executable ? 0755 : 0644) || close(destination) || close(source) || close(original)) fail("finishing installed output");
    return 0;
}
