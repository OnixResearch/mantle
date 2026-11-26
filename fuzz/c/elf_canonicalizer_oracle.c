#define main stagex_elf_canonicalizer_main
#include "../../bootstrap/stagex-elf-local-symbol-canonicalizer.c"

int fuzz_c_canonicalize_elf(unsigned char *bytes, uint64_t file_size, uint64_t *rewrite_count)
{
    return canonicalize_elf(bytes, file_size, rewrite_count);
}

int fuzz_c_canonicalize_file(const char *path)
{
    return canonicalize_file(path);
}
