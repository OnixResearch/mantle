#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define main stagex_elf_canonicalizer_main
#include "../../bootstrap/stagex-elf-local-symbol-canonicalizer.c"

static int has_disjoint_symtab_and_strtab(const unsigned char *bytes, uint64_t file_size)
{
    Elf64_Ehdr header;
    uint64_t i;
    if (file_size < sizeof(Elf64_Ehdr)) return 0;
    memcpy(&header, bytes, sizeof(Elf64_Ehdr));
    if (header.e_shnum == 0U || header.e_shnum > SECTION_COUNT_MAX) return 0;
    if (header.e_shentsize != sizeof(Elf64_Shdr)) return 0;
    if (!range_valid(header.e_shoff, (uint64_t)header.e_shnum * sizeof(Elf64_Shdr), file_size)) return 0;
    for (i = 0U; i < header.e_shnum; i++) {
        Elf64_Shdr sym_sec;
        Elf64_Shdr str_sec;
        uint64_t sym_end;
        uint64_t str_end;
        uint64_t sh_start = header.e_shoff;
        uint64_t sh_end = header.e_shoff + (uint64_t)header.e_shnum * sizeof(Elf64_Shdr);
        if (!read_section_header(bytes, file_size, header.e_shoff, i, &sym_sec)) return 0;
        if (sym_sec.sh_type != SHT_SYMTAB) continue;
        if (!read_section_header(bytes, file_size, header.e_shoff, sym_sec.sh_link, &str_sec)) return 0;
        sym_end = sym_sec.sh_offset + sym_sec.sh_size;
        str_end = str_sec.sh_offset + str_sec.sh_size;
        if (sym_sec.sh_offset < str_end && str_sec.sh_offset < sym_end) return 0;
        if (str_sec.sh_offset < sh_end && sh_start < str_end) return 0;
    }
    return 1;
}

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    unsigned char *buf1;
    unsigned char *buf2;
    uint64_t rewrites1 = 0U;
    uint64_t rewrites2 = 0U;
    int disjoint;
    int ok1;
    int ok2;

    if (size > FILE_BYTES_MAX) return 0;
    buf1 = (unsigned char *)malloc(size ? size : 1U);
    if (buf1 == NULL) __builtin_trap();
    if (size > 0U) memcpy(buf1, data, size);

    disjoint = has_disjoint_symtab_and_strtab(buf1, (uint64_t)size);
    ok1 = canonicalize_elf(buf1, (uint64_t)size, &rewrites1);
    if (ok1 == 1) {
        buf2 = (unsigned char *)malloc(size ? size : 1U);
        if (buf2 == NULL) __builtin_trap();
        if (size > 0U) memcpy(buf2, buf1, size);
        ok2 = canonicalize_elf(buf2, (uint64_t)size, &rewrites2);
        if (ok2 != 1) __builtin_trap();
        if (disjoint && (rewrites2 != rewrites1 || (size > 0U && memcmp(buf1, buf2, size) != 0))) {
            __builtin_trap();
        }
        free(buf2);
    }
    free(buf1);
    return 0;
}
