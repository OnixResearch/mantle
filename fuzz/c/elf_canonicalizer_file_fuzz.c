#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#define main stagex_elf_canonicalizer_main
#include "../../bootstrap/stagex-elf-local-symbol-canonicalizer.c"

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    char dir_tmpl[] = "/dev/shm/mantle-cfile-fuzz-XXXXXX";
    char file_path[PATH_BYTES_MAX];
    char tmp_path[PATH_BYTES_MAX];
    unsigned char *mem_copy = NULL;
    uint64_t rewrites = 0U;
    int mem_ok = 0;
    int fd;
    int rc;

    if (size > 1048576U) return 0;
    if (mkdtemp(dir_tmpl) == NULL) return 0;
    if (snprintf(file_path, sizeof(file_path), "%s/in.o", dir_tmpl) <= 0) {
        rmdir(dir_tmpl);
        return 0;
    }
    if (snprintf(tmp_path, sizeof(tmp_path), "%s/in.o.mantle-canonical.tmp", dir_tmpl) <= 0) {
        rmdir(dir_tmpl);
        return 0;
    }

    fd = open(file_path, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, 0644);
    if (fd >= 0) {
        if (size == 0U || write_exact(fd, data, (uint64_t)size)) {
            close(fd);
            mem_copy = (unsigned char *)malloc(size ? size : 1U);
            if (mem_copy == NULL) __builtin_trap();
            if (size > 0U) memcpy(mem_copy, data, size);
            mem_ok = canonicalize_elf(mem_copy, (uint64_t)size, &rewrites);

            rc = canonicalize_file(file_path);
            if (size == 0U) {
                if (rc != EXIT_INPUT) __builtin_trap();
            } else if (mem_ok == 1) {
                if (rc != 0) __builtin_trap();
                int out_fd = open(file_path, O_RDONLY | O_NOFOLLOW);
                if (out_fd < 0) __builtin_trap();
                unsigned char *disk_bytes = (unsigned char *)malloc(size);
                if (disk_bytes == NULL) __builtin_trap();
                if (!read_exact(out_fd, disk_bytes, (uint64_t)size)) __builtin_trap();
                close(out_fd);
                if (memcmp(disk_bytes, mem_copy, size) != 0) __builtin_trap();
                free(disk_bytes);
            } else {
                if (rc != EXIT_FORMAT) __builtin_trap();
            }
            free(mem_copy);
        } else {
            close(fd);
        }
    }

    unlink(tmp_path);
    unlink(file_path);
    rmdir(dir_tmpl);
    return 0;
}
