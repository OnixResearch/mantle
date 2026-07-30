#include <elf.h>
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#define FILE_MEBIBYTES_MAX 64U
#define MEBIBYTE_BYTES (1024U * 1024U)
#define FILE_BYTES_MAX ((uint64_t)FILE_MEBIBYTES_MAX * MEBIBYTE_BYTES)
#define SECTION_COUNT_MAX 4096U
#define SYMBOL_COUNT_MAX 1048576U
#define PATH_BYTES_MAX 4096U
#define IO_OPERATION_COUNT_MAX FILE_BYTES_MAX
#define EXIT_USAGE 64
#define EXIT_INPUT 65
#define EXIT_FORMAT 66
#define EXIT_OUTPUT 67
#define TEMP_SUFFIX_BYTES 22U
#define LOCAL_SYMBOL_PREFIX_BYTES 2U
#define LOCAL_SYMBOL_NAME_BYTES_MIN 3U
#define DECIMAL_RADIX 10U
#define PARENT_COMPONENT_BYTES 3U
#define PARENT_SECOND_DOT_OFFSET 2U
#define REQUIRED_ARGUMENT_COUNT 2
#define FILE_MODE_PERMISSION_MASK 0777U

static int range_valid(uint64_t offset, uint64_t length, uint64_t size)
{
    return offset <= size && length <= size - offset;
}

static int decimal_name(const unsigned char *name, uint64_t length)
{
    uint64_t index;
    if (length < LOCAL_SYMBOL_NAME_BYTES_MIN || name[0] != 'L' || name[1] != '.') return 0;
    for (index = LOCAL_SYMBOL_PREFIX_BYTES; index < length; index++) {
        if (name[index] < '0' || name[index] > '9') return 0;
    }
    return 1;
}

static uint64_t bounded_string_length(const unsigned char *bytes, uint64_t bytes_max)
{
    uint64_t length = 0U;
    while (length < bytes_max && bytes[length] != 0U) length++;
    return length;
}

static int write_decimal_index(unsigned char *name, uint64_t length, uint64_t symbol_index)
{
    uint64_t cursor;
    uint64_t value = symbol_index;
    uint64_t digit_count = length - LOCAL_SYMBOL_PREFIX_BYTES;
    if (digit_count == 0U) return 0;
    for (cursor = 0U; cursor < digit_count; cursor++) name[LOCAL_SYMBOL_PREFIX_BYTES + cursor] = '0';
    cursor = length;
    do {
        if (cursor <= LOCAL_SYMBOL_PREFIX_BYTES) return 0;
        cursor--;
        name[cursor] = (unsigned char)('0' + (value % DECIMAL_RADIX));
        value /= DECIMAL_RADIX;
    } while (value != 0U);
    return 1;
}

static int read_section_header(
    const unsigned char *bytes,
    uint64_t file_size,
    uint64_t section_offset,
    uint64_t section_index,
    Elf64_Shdr *section)
{
    uint64_t relative_offset;
    uint64_t offset;
    if (section_index > SECTION_COUNT_MAX) return 0;
    relative_offset = section_index * sizeof(Elf64_Shdr);
    if (section_offset > UINT64_MAX - relative_offset) return 0;
    offset = section_offset + relative_offset;
    if (!range_valid(offset, sizeof(Elf64_Shdr), file_size)) return 0;
    memcpy(section, bytes + offset, sizeof(Elf64_Shdr));
    return 1;
}

static int canonicalize_symbol_table(
    unsigned char *bytes,
    uint64_t file_size,
    uint64_t section_offset,
    uint64_t section_count,
    const Elf64_Shdr *symbols,
    uint64_t *rewrite_count)
{
    Elf64_Shdr strings;
    unsigned char *string_bytes;
    uint64_t symbol_count;
    uint64_t symbol_index;
    if (symbols->sh_link >= section_count) return 0;
    if (!read_section_header(bytes, file_size, section_offset, symbols->sh_link, &strings)) return 0;
    if (strings.sh_type != SHT_STRTAB) return 0;
    if (symbols->sh_entsize != sizeof(Elf64_Sym) || symbols->sh_size % sizeof(Elf64_Sym) != 0U) return 0;
    if (!range_valid(symbols->sh_offset, symbols->sh_size, file_size)) return 0;
    if (!range_valid(strings.sh_offset, strings.sh_size, file_size)) return 0;
    symbol_count = symbols->sh_size / sizeof(Elf64_Sym);
    if (symbol_count > SYMBOL_COUNT_MAX) return 0;
    string_bytes = bytes + strings.sh_offset;
    for (symbol_index = 0U; symbol_index < symbol_count; symbol_index++) {
        Elf64_Sym symbol;
        unsigned char *name;
        uint64_t name_length;
        memcpy(&symbol, bytes + symbols->sh_offset + symbol_index * sizeof(Elf64_Sym), sizeof(Elf64_Sym));
        if (ELF64_ST_BIND(symbol.st_info) != STB_LOCAL || symbol.st_name >= strings.sh_size) continue;
        name = string_bytes + symbol.st_name;
        name_length = bounded_string_length(name, strings.sh_size - symbol.st_name);
        if (name_length == strings.sh_size - symbol.st_name) return 0;
        if (!decimal_name(name, name_length)) continue;
        if (!write_decimal_index(name, name_length, symbol_index)) return 0;
        if (*rewrite_count == UINT64_MAX) return 0;
        (*rewrite_count)++;
    }
    return 1;
}

static int canonicalize_elf(unsigned char *bytes, uint64_t file_size, uint64_t *rewrite_count)
{
    Elf64_Ehdr header;
    uint64_t section_bytes;
    uint64_t section_index;
    if (file_size < sizeof(Elf64_Ehdr)) return 0;
    memcpy(&header, bytes, sizeof(Elf64_Ehdr));
    if (header.e_ident[EI_MAG0] != ELFMAG0 || header.e_ident[EI_MAG1] != ELFMAG1) return 0;
    if (header.e_ident[EI_MAG2] != ELFMAG2 || header.e_ident[EI_MAG3] != ELFMAG3) return 0;
    if (header.e_ident[EI_CLASS] != ELFCLASS64 || header.e_ident[EI_DATA] != ELFDATA2LSB) return 0;
    if (header.e_shentsize != sizeof(Elf64_Shdr) || header.e_shnum == 0U) return 0;
    if (header.e_shnum > SECTION_COUNT_MAX) return 0;
    section_bytes = (uint64_t)header.e_shnum * sizeof(Elf64_Shdr);
    if (!range_valid(header.e_shoff, section_bytes, file_size)) return 0;
    *rewrite_count = 0U;
    for (section_index = 0U; section_index < header.e_shnum; section_index++) {
        Elf64_Shdr section;
        if (!read_section_header(bytes, file_size, header.e_shoff, section_index, &section)) return 0;
        if (section.sh_type != SHT_SYMTAB) continue;
        if (!canonicalize_symbol_table(
                bytes,
                file_size,
                header.e_shoff,
                header.e_shnum,
                &section,
                rewrite_count)) return 0;
    }
    return 1;
}

static int read_exact(int fd, unsigned char *bytes, uint64_t size)
{
    uint64_t offset = 0U;
    uint64_t operation_count = 0U;
    while (offset < size && operation_count < IO_OPERATION_COUNT_MAX) {
        ssize_t result = read(fd, bytes + offset, (size_t)(size - offset));
        if (result < 0 && errno == EINTR) continue;
        if (result <= 0) return 0;
        offset += (uint64_t)result;
        operation_count++;
    }
    return offset == size;
}

static int write_exact(int fd, const unsigned char *bytes, uint64_t size)
{
    uint64_t offset = 0U;
    uint64_t operation_count = 0U;
    while (offset < size && operation_count < IO_OPERATION_COUNT_MAX) {
        ssize_t result = write(fd, bytes + offset, (size_t)(size - offset));
        if (result < 0 && errno == EINTR) continue;
        if (result <= 0) return 0;
        offset += (uint64_t)result;
        operation_count++;
    }
    return offset == size;
}

static uint64_t path_length(const char *path)
{
    uint64_t length = 0U;
    while (length < PATH_BYTES_MAX && path[length] != 0) length++;
    return length;
}

static int path_is_bounded_absolute(const char *path, uint64_t length)
{
    uint64_t index;
    if (length == 0U || length >= PATH_BYTES_MAX || path[0] != '/') return 0;
    for (index = 0U; index + PARENT_COMPONENT_BYTES < length; index++) {
        if (path[index] == '/' && path[index + 1U] == '.' && path[index + PARENT_SECOND_DOT_OFFSET] == '.' && path[index + PARENT_COMPONENT_BYTES] == '/') return 0;
    }
    if (length >= PARENT_COMPONENT_BYTES && path[length - PARENT_COMPONENT_BYTES] == '/' && path[length - PARENT_SECOND_DOT_OFFSET] == '.' && path[length - 1U] == '.') return 0;
    return 1;
}

static int temp_path(char *output, const char *path, uint64_t path_bytes)
{
    static const char suffix[TEMP_SUFFIX_BYTES] = {
        '.', 'm', 'a', 'n', 't', 'l', 'e', '-', 'c', 'a', 'n', 'o', 'n', 'i', 'c', 'a', 'l', '.', 't', 'm', 'p', 0
    };
    uint64_t index;
    if (path_bytes + TEMP_SUFFIX_BYTES > PATH_BYTES_MAX) return 0;
    for (index = 0U; index < path_bytes; index++) output[index] = path[index];
    for (index = 0U; index < TEMP_SUFFIX_BYTES; index++) output[path_bytes + index] = suffix[index];
    return 1;
}

static int canonicalize_file(const char *path)
{
    struct stat metadata;
    struct stat opened_metadata;
    unsigned char *bytes = NULL;
    char staged_path[PATH_BYTES_MAX] = {0};
    uint64_t path_bytes = path_length(path);
    uint64_t rewrite_count = 0U;
    int input_fd = -1;
    int output_fd = -1;
    int result = EXIT_INPUT;
    if (!path_is_bounded_absolute(path, path_bytes)) return EXIT_USAGE;
    if (lstat(path, &metadata) != 0 || !S_ISREG(metadata.st_mode)) return EXIT_INPUT;
    if (metadata.st_size <= 0 || (uint64_t)metadata.st_size > FILE_BYTES_MAX) return EXIT_INPUT;
    bytes = (unsigned char *)malloc((size_t)metadata.st_size);
    if (bytes == NULL) return EXIT_INPUT;
    input_fd = open(path, O_RDONLY | O_NOFOLLOW);
    if (input_fd < 0 || fstat(input_fd, &opened_metadata) != 0) goto cleanup;
    if (!S_ISREG(opened_metadata.st_mode)) goto cleanup;
    if (opened_metadata.st_dev != metadata.st_dev || opened_metadata.st_ino != metadata.st_ino) goto cleanup;
    if (opened_metadata.st_size != metadata.st_size) goto cleanup;
    if (!read_exact(input_fd, bytes, (uint64_t)metadata.st_size)) goto cleanup;
    if (close(input_fd) != 0) goto cleanup;
    input_fd = -1;
    if (!canonicalize_elf(bytes, (uint64_t)metadata.st_size, &rewrite_count)) {
        result = EXIT_FORMAT;
        goto cleanup;
    }
    if (!temp_path(staged_path, path, path_bytes)) {
        result = EXIT_OUTPUT;
        goto cleanup;
    }
    output_fd = open(staged_path, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, metadata.st_mode & FILE_MODE_PERMISSION_MASK);
    if (output_fd < 0) {
        result = EXIT_OUTPUT;
        goto cleanup;
    }
    if (!write_exact(output_fd, bytes, (uint64_t)metadata.st_size)) {
        result = EXIT_OUTPUT;
        goto cleanup;
    }
    if (fchmod(output_fd, metadata.st_mode & FILE_MODE_PERMISSION_MASK) != 0 || fsync(output_fd) != 0) {
        result = EXIT_OUTPUT;
        goto cleanup;
    }
    if (close(output_fd) != 0) {
        output_fd = -1;
        result = EXIT_OUTPUT;
        goto cleanup;
    }
    output_fd = -1;
    if (rename(staged_path, path) != 0) {
        result = EXIT_OUTPUT;
        goto cleanup;
    }
    result = 0;
cleanup:
    if (input_fd >= 0) close(input_fd);
    if (output_fd >= 0) close(output_fd);
    if (result != 0 && staged_path[0] != 0) unlink(staged_path);
    free(bytes);
    return result;
}

int main(int argc, char **argv)
{
    if (argc != REQUIRED_ARGUMENT_COUNT || argv == NULL || argv[1] == NULL) return EXIT_USAGE;
    return canonicalize_file(argv[1]);
}
